//! Model calibration pipeline.
//!
//! The pipeline has two halves:
//!
//! 1. **Quote sourcing** — callers load vanilla option quotes and convert
//!    them into [`MarketQuote`] values. In this platform, `pricing-api`
//!    reads the `options_data` TimescaleDB table and computes mid prices.
//! 2. **Parameter fitting** — a calibrator (e.g.
//!    [`heston::HestonCalibrator`]) minimises the pricing error against
//!    those quotes and returns fitted model parameters.
//!
//! Keeping quote sourcing out of this crate means calibration stays pure and
//! synchronous; database access (and its async runtime) lives in the API
//! service.

pub mod heston;

use crate::instruments::option::OptionType;
use rust_decimal::Decimal;
use std::cmp::Ordering;

/// A vanilla option market quote used as a calibration target.
#[derive(Debug, Clone)]
pub struct MarketQuote {
    /// Strike price.
    pub strike: Decimal,
    /// Time to expiry in years.
    pub time_to_expiry: f64,
    /// Call or put.
    pub option_type: OptionType,
    /// Observed market price (typically the bid/ask mid).
    pub market_price: Decimal,
}

/// Outcome of a derivative-free minimisation run.
#[derive(Debug, Clone)]
pub(crate) struct OptimizationOutcome {
    pub best: Vec<f64>,
    pub best_value: f64,
    pub iterations: usize,
    pub converged: bool,
}

/// Minimise `f` with the Nelder-Mead simplex method.
///
/// `x0` is the starting point and `step` the per-dimension perturbation used
/// to build the initial simplex. Convergence is declared when the spread of
/// objective values across the simplex falls below `tolerance` (relative to
/// the best value).
pub(crate) fn nelder_mead<F>(
    f: F,
    x0: &[f64],
    step: &[f64],
    max_iterations: usize,
    tolerance: f64,
) -> OptimizationOutcome
where
    F: Fn(&[f64]) -> f64,
{
    let n = x0.len();
    let (alpha, gamma, beta, delta) = (1.0, 2.0, 0.5, 0.5);

    let mut simplex: Vec<Vec<f64>> = vec![x0.to_vec()];
    for i in 0..n {
        let mut vertex = x0.to_vec();
        vertex[i] += step[i];
        simplex.push(vertex);
    }
    let mut values: Vec<f64> = simplex.iter().map(|x| f(x)).collect();

    let mut iterations = 0;
    let mut converged = false;

    while iterations < max_iterations {
        iterations += 1;

        let mut order: Vec<usize> = (0..=n).collect();
        order.sort_by(|&a, &b| values[a].partial_cmp(&values[b]).unwrap_or(Ordering::Equal));
        simplex = order.iter().map(|&i| simplex[i].clone()).collect();
        values = order.iter().map(|&i| values[i]).collect();

        if (values[n] - values[0]).abs() <= tolerance * (1.0 + values[0].abs()) {
            converged = true;
            break;
        }

        // Centroid of all vertices except the worst.
        let mut centroid = vec![0.0; n];
        for vertex in &simplex[..n] {
            for j in 0..n {
                centroid[j] += vertex[j] / n as f64;
            }
        }

        let reflected: Vec<f64> = (0..n)
            .map(|j| centroid[j] + alpha * (centroid[j] - simplex[n][j]))
            .collect();
        let f_reflected = f(&reflected);

        if f_reflected < values[0] {
            let expanded: Vec<f64> = (0..n)
                .map(|j| centroid[j] + gamma * (reflected[j] - centroid[j]))
                .collect();
            let f_expanded = f(&expanded);
            if f_expanded < f_reflected {
                simplex[n] = expanded;
                values[n] = f_expanded;
            } else {
                simplex[n] = reflected;
                values[n] = f_reflected;
            }
        } else if f_reflected < values[n - 1] {
            simplex[n] = reflected;
            values[n] = f_reflected;
        } else {
            let contracted: Vec<f64> = (0..n)
                .map(|j| centroid[j] + beta * (simplex[n][j] - centroid[j]))
                .collect();
            let f_contracted = f(&contracted);
            if f_contracted < values[n] {
                simplex[n] = contracted;
                values[n] = f_contracted;
            } else {
                // Shrink the whole simplex towards the best vertex.
                let (best, rest) = simplex.split_at_mut(1);
                for (vertex, value) in rest.iter_mut().zip(values.iter_mut().skip(1)) {
                    for (coord, &anchor) in vertex.iter_mut().zip(best[0].iter()) {
                        *coord = anchor + delta * (*coord - anchor);
                    }
                    *value = f(vertex);
                }
            }
        }
    }

    let best_index = values
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    OptimizationOutcome {
        best: simplex[best_index].clone(),
        best_value: values[best_index],
        iterations,
        converged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nelder_mead_minimises_quadratic() {
        // f(x, y) = (x - 3)² + (y + 1)², minimum at (3, -1)
        let f = |x: &[f64]| (x[0] - 3.0).powi(2) + (x[1] + 1.0).powi(2);
        let outcome = nelder_mead(f, &[0.0, 0.0], &[0.5, 0.5], 500, 1e-12);

        assert!(outcome.converged);
        assert!((outcome.best[0] - 3.0).abs() < 1e-4);
        assert!((outcome.best[1] + 1.0).abs() < 1e-4);
        assert!(outcome.best_value < 1e-8);
    }
}

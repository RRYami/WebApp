//! Heston model calibration against vanilla option quotes.
//!
//! Minimises the mean squared error between model and market prices with a
//! bounded Nelder-Mead search over (v₀, κ, θ, σ, ρ).

use super::{nelder_mead, MarketQuote};
use crate::core::error::{Error, Result};
use crate::pricing::heston::{Heston, HestonParams};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Box bounds for (v0, kappa, theta, sigma, rho).
const BOUNDS: [(f64, f64); 5] = [
    (1e-4, 2.0),     // v0
    (1e-3, 20.0),    // kappa
    (1e-4, 2.0),     // theta
    (1e-3, 5.0),     // sigma
    (-0.999, 0.999), // rho
];

/// Objective value returned for out-of-bounds or failing parameter sets,
/// steering the simplex back into the feasible region.
const PENALTY: f64 = 1e8;

/// Configuration for a Heston calibration run.
#[derive(Debug, Clone)]
pub struct CalibrationConfig {
    /// Starting point for the optimiser.
    pub initial_guess: HestonParams,
    /// Maximum Nelder-Mead iterations.
    pub max_iterations: usize,
    /// Relative convergence tolerance on the objective spread.
    pub tolerance: f64,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            initial_guess: HestonParams {
                v0: 0.04,
                kappa: 1.5,
                theta: 0.04,
                sigma: 0.5,
                rho: -0.5,
            },
            max_iterations: 1000,
            tolerance: 1e-10,
        }
    }
}

/// Result of a Heston calibration run.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CalibrationResult {
    /// Fitted model parameters.
    pub params: HestonParams,
    /// Root mean squared pricing error across the quotes.
    pub rmse: f64,
    /// Optimiser iterations used.
    pub iterations: usize,
    /// Whether the optimiser converged within tolerance.
    pub converged: bool,
}

/// Calibrates [`HestonParams`] to a set of vanilla option quotes sharing one
/// underlying (single spot and risk-free rate).
#[derive(Debug, Clone)]
pub struct HestonCalibrator {
    spot: Decimal,
    rate: Decimal,
    config: CalibrationConfig,
}

impl HestonCalibrator {
    /// Create a calibrator with the default configuration.
    pub fn new(spot: Decimal, rate: Decimal) -> Self {
        Self::with_config(spot, rate, CalibrationConfig::default())
    }

    /// Create a calibrator with a custom configuration.
    pub fn with_config(spot: Decimal, rate: Decimal, config: CalibrationConfig) -> Self {
        Self { spot, rate, config }
    }

    /// Fit Heston parameters to the given quotes.
    pub fn calibrate(&self, quotes: &[MarketQuote]) -> Result<CalibrationResult> {
        if quotes.len() < 5 {
            return Err(Error::invalid_input(format!(
                "Heston calibration needs at least 5 quotes (one per parameter), got {}",
                quotes.len()
            )));
        }
        self.config.initial_guess.validate()?;

        let targets = quotes
            .iter()
            .map(|q| {
                let price = q
                    .market_price
                    .to_f64()
                    .ok_or_else(|| Error::arithmetic("Invalid market price"))?;
                if price <= 0.0 || q.time_to_expiry <= 0.0 {
                    return Err(Error::invalid_input(
                        "Quotes must have positive price and time to expiry",
                    ));
                }
                Ok((q.strike, q.time_to_expiry, q.option_type, price))
            })
            .collect::<Result<Vec<_>>>()?;

        let spot = self.spot;
        let rate = self.rate;
        let objective = |x: &[f64]| -> f64 {
            let mut violation = 0.0;
            for (value, (lo, hi)) in x.iter().zip(BOUNDS.iter()) {
                if value < lo {
                    violation += lo - value;
                }
                if value > hi {
                    violation += value - hi;
                }
            }
            if violation > 0.0 {
                return PENALTY * (1.0 + violation);
            }

            let params = HestonParams {
                v0: x[0],
                kappa: x[1],
                theta: x[2],
                sigma: x[3],
                rho: x[4],
            };
            let mut sum_squared = 0.0;
            for &(strike, time, option_type, target) in &targets {
                match Heston::price(spot, strike, rate, &params, time, option_type) {
                    Ok(price) => {
                        let error = price.to_f64().unwrap_or(f64::MAX) - target;
                        sum_squared += error * error;
                    }
                    Err(_) => return PENALTY,
                }
            }
            sum_squared / targets.len() as f64
        };

        let guess = self.config.initial_guess;
        let x0 = [guess.v0, guess.kappa, guess.theta, guess.sigma, guess.rho];
        let step = [0.02, 0.5, 0.02, 0.1, 0.1];
        let outcome = nelder_mead(
            objective,
            &x0,
            &step,
            self.config.max_iterations,
            self.config.tolerance,
        );

        let params = HestonParams::new(
            outcome.best[0],
            outcome.best[1],
            outcome.best[2],
            outcome.best[3],
            outcome.best[4],
        )?;

        Ok(CalibrationResult {
            params,
            rmse: outcome.best_value.max(0.0).sqrt(),
            iterations: outcome.iterations,
            converged: outcome.converged,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruments::option::OptionType;
    use rust_decimal_macros::dec;

    fn synthetic_quotes(spot: Decimal, rate: Decimal, params: &HestonParams) -> Vec<MarketQuote> {
        let strikes = [dec!(80), dec!(90), dec!(100), dec!(110), dec!(120)];
        let maturities = [0.5, 1.0];
        let mut quotes = Vec::new();
        for &time in &maturities {
            for &strike in &strikes {
                let option_type = if strike <= spot {
                    OptionType::Put
                } else {
                    OptionType::Call
                };
                let price = Heston::price(spot, strike, rate, params, time, option_type).unwrap();
                quotes.push(MarketQuote {
                    strike,
                    time_to_expiry: time,
                    option_type,
                    market_price: price,
                });
            }
        }
        quotes
    }

    #[test]
    fn test_calibration_fits_synthetic_quotes() {
        let spot = dec!(100);
        let rate = dec!(0.03);
        let true_params = HestonParams::new(0.05, 2.0, 0.05, 0.4, -0.6).unwrap();
        let quotes = synthetic_quotes(spot, rate, &true_params);

        let config = CalibrationConfig {
            max_iterations: 400,
            ..Default::default()
        };
        let calibrator = HestonCalibrator::with_config(spot, rate, config);
        let result = calibrator.calibrate(&quotes).unwrap();

        // The fitted parameters must reprice the quotes accurately.
        assert!(
            result.rmse < 0.05,
            "RMSE too high: {} (params {:?})",
            result.rmse,
            result.params
        );
        assert!(result.params.validate().is_ok());
    }

    #[test]
    fn test_calibration_rejects_too_few_quotes() {
        let calibrator = HestonCalibrator::new(dec!(100), dec!(0.03));
        let quotes = vec![MarketQuote {
            strike: dec!(100),
            time_to_expiry: 1.0,
            option_type: OptionType::Call,
            market_price: dec!(10),
        }];
        assert!(calibrator.calibrate(&quotes).is_err());
    }

    #[test]
    fn test_calibration_rejects_invalid_quotes() {
        let calibrator = HestonCalibrator::new(dec!(100), dec!(0.03));
        let quotes: Vec<MarketQuote> = (0..5)
            .map(|i| MarketQuote {
                strike: dec!(100),
                time_to_expiry: 1.0,
                option_type: OptionType::Call,
                market_price: if i == 0 { dec!(-1) } else { dec!(10) },
            })
            .collect();
        assert!(calibrator.calibrate(&quotes).is_err());
    }
}

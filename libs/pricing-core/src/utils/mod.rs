//! Utility functions and helpers.

use crate::core::error::{Error, Result};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Approximate equality for floating-point comparisons.
pub fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

/// Approximate equality for Decimal comparisons.
pub fn approx_eq_decimal(a: Decimal, b: Decimal, tolerance: Decimal) -> bool {
    (a - b).abs() < tolerance
}

/// Calculate the natural logarithm of a Decimal.
pub fn ln(x: Decimal) -> Result<f64> {
    if x <= Decimal::ZERO {
        return Err(Error::arithmetic("Cannot take ln of non-positive number"));
    }
    x.to_f64()
        .map(|f| f.ln())
        .ok_or_else(|| Error::arithmetic("Failed to convert to f64 for ln"))
}

/// Calculate the exponential of a Decimal.
pub fn exp(x: Decimal) -> Result<Decimal> {
    x.to_f64()
        .map(|f| f.exp())
        .and_then(|f| Decimal::try_from(f).ok())
        .ok_or_else(|| Error::arithmetic("Failed to calculate exp"))
}

/// Calculate the square root of a Decimal.
pub fn sqrt(x: Decimal) -> Result<Decimal> {
    if x < Decimal::ZERO {
        return Err(Error::arithmetic("Cannot take sqrt of negative number"));
    }
    x.to_f64()
        .map(|f| f.sqrt())
        .and_then(|f| Decimal::try_from(f).ok())
        .ok_or_else(|| Error::arithmetic("Failed to calculate sqrt"))
}

/// Convert an annual rate to a periodic rate.
pub fn annual_to_periodic_rate(annual_rate: Decimal, periods_per_year: u32) -> Decimal {
    annual_rate / Decimal::from(periods_per_year)
}

/// Convert a periodic rate to an annual rate.
pub fn periodic_to_annual_rate(periodic_rate: Decimal, periods_per_year: u32) -> Decimal {
    periodic_rate * Decimal::from(periods_per_year)
}

/// Calculate the continuously compounded rate from a simple rate.
pub fn simple_to_continuous_rate(simple_rate: Decimal, time: Decimal) -> Result<Decimal> {
    if time <= Decimal::ZERO {
        return Err(Error::invalid_input("Time must be positive"));
    }
    let factor = Decimal::ONE + simple_rate * time;
    factor
        .to_f64()
        .map(|f| f.ln() / time.to_f64().unwrap_or(1.0))
        .and_then(|f| Decimal::try_from(f).ok())
        .ok_or_else(|| Error::arithmetic("Failed to convert rate"))
}

/// Calculate the simple rate from a continuously compounded rate.
pub fn continuous_to_simple_rate(continuous_rate: Decimal, time: Decimal) -> Result<Decimal> {
    if time <= Decimal::ZERO {
        return Err(Error::invalid_input("Time must be positive"));
    }
    continuous_rate
        .to_f64()
        .map(|r| (r * time.to_f64().unwrap_or(0.0)).exp())
        .and_then(|f| Decimal::try_from(f - 1.0).ok())
        .map(|df| df / time)
        .ok_or_else(|| Error::arithmetic("Failed to convert rate"))
}

/// Generate a range of values.
pub fn linspace(start: f64, end: f64, num_points: usize) -> Vec<f64> {
    if num_points < 2 {
        return vec![start];
    }

    let step = (end - start) / (num_points - 1) as f64;
    (0..num_points).map(|i| start + step * i as f64).collect()
}

/// Linear interpolation between two points.
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Linear interpolation for Decimal values.
pub fn lerp_decimal(a: Decimal, b: Decimal, t: Decimal) -> Decimal {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(1.0, 1.0000001, 1e-6));
        assert!(!approx_eq(1.0, 1.1, 1e-6));
    }

    #[test]
    fn test_ln() {
        let result = ln(dec!(2.718281828459045)).unwrap();
        assert!(approx_eq(result, 1.0, 1e-10));
    }

    #[test]
    fn test_ln_error() {
        assert!(ln(dec!(0)).is_err());
        assert!(ln(dec!(-1)).is_err());
    }

    #[test]
    fn test_sqrt() {
        let result = sqrt(dec!(16)).unwrap();
        assert_eq!(result, dec!(4));
    }

    #[test]
    fn test_sqrt_error() {
        assert!(sqrt(dec!(-1)).is_err());
    }

    #[test]
    fn test_linspace() {
        let space = linspace(0.0, 1.0, 5);
        assert_eq!(space.len(), 5);
        assert!(approx_eq(space[0], 0.0, 1e-10));
        assert!(approx_eq(space[4], 1.0, 1e-10));
    }

    #[test]
    fn test_lerp() {
        assert!(approx_eq(lerp(0.0, 10.0, 0.5), 5.0, 1e-10));
        assert!(approx_eq(lerp(0.0, 10.0, 0.0), 0.0, 1e-10));
        assert!(approx_eq(lerp(0.0, 10.0, 1.0), 10.0, 1e-10));
    }

    #[test]
    fn test_rate_conversions() {
        let simple = dec!(0.05);
        let time = dec!(1);

        let continuous = simple_to_continuous_rate(simple, time).unwrap();
        let back_to_simple = continuous_to_simple_rate(continuous, time).unwrap();

        assert!(approx_eq_decimal(simple, back_to_simple, dec!(0.0001)));
    }
}

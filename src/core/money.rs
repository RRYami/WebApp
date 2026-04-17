//! Money type for precise financial calculations.

use crate::core::currency::CurrencyCode;
use crate::core::error::{Error, Result};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// A monetary value with an associated currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    amount: Decimal,
    currency: CurrencyCode,
}

impl Money {
    /// Create a new money value.
    ///
    /// # Examples
    ///
    /// ```
    /// use pricing_lib::prelude::*;
    /// use rust_decimal_macros::dec;
    ///
    /// let amount = Money::new(dec!(100.50), CurrencyCode::USD);
    /// ```
    pub fn new(amount: Decimal, currency: CurrencyCode) -> Self {
        Self { amount, currency }
    }

    /// Create a new money value with zero amount.
    pub fn zero(currency: CurrencyCode) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    /// Get the monetary amount.
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// Get the currency code.
    pub fn currency(&self) -> CurrencyCode {
        self.currency
    }

    /// Check if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    /// Check if the amount is positive.
    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive() && !self.amount.is_zero()
    }

    /// Check if the amount is negative.
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative()
    }

    /// Get the absolute value.
    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            currency: self.currency,
        }
    }

    /// Round to the given number of decimal places.
    pub fn round(&self, decimal_places: u32) -> Self {
        Self {
            amount: self.amount.round_dp(decimal_places),
            currency: self.currency,
        }
    }

    /// Convert to a different currency using the given exchange rate.
    ///
    /// The rate is defined as units of target currency per unit of source currency.
    pub fn convert(&self, target_currency: CurrencyCode, rate: Decimal) -> Self {
        Self {
            amount: self.amount * rate,
            currency: target_currency,
        }
    }

    /// Add two money values of the same currency.
    ///
    /// # Errors
    ///
    /// Returns an error if the currencies don't match.
    pub fn checked_add(&self, other: &Self) -> Result<Self> {
        if self.currency != other.currency {
            return Err(Error::currency_mismatch(
                self.currency.as_str(),
                other.currency.as_str(),
            ));
        }
        Ok(Self {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }

    /// Subtract two money values of the same currency.
    ///
    /// # Errors
    ///
    /// Returns an error if the currencies don't match.
    pub fn checked_sub(&self, other: &Self) -> Result<Self> {
        if self.currency != other.currency {
            return Err(Error::currency_mismatch(
                self.currency.as_str(),
                other.currency.as_str(),
            ));
        }
        Ok(Self {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }

    /// Multiply by a scalar.
    pub fn mul_scalar(&self, scalar: Decimal) -> Self {
        Self {
            amount: self.amount * scalar,
            currency: self.currency,
        }
    }

    /// Divide by a scalar.
    ///
    /// # Errors
    ///
    /// Returns an error if dividing by zero.
    pub fn div_scalar(&self, scalar: Decimal) -> Result<Self> {
        if scalar.is_zero() {
            return Err(Error::arithmetic("Division by zero"));
        }
        Ok(Self {
            amount: self.amount / scalar,
            currency: self.currency,
        })
    }

    /// Calculate the present value using continuous compounding.
    pub fn present_value_continuous(&self, rate: Decimal, time: Decimal) -> Result<Self> {
        let df = (-rate * time).exp();
        Ok(Self {
            amount: self.amount * df,
            currency: self.currency,
        })
    }

    /// Calculate the future value using continuous compounding.
    pub fn future_value_continuous(&self, rate: Decimal, time: Decimal) -> Result<Self> {
        let fv_factor = (rate * time).exp();
        Ok(Self {
            amount: self.amount * fv_factor,
            currency: self.currency,
        })
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.currency, self.amount)
    }
}

// Implement Add for Money (panics on currency mismatch)
impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        self.checked_add(&other)
            .expect("Currency mismatch in Money addition")
    }
}

// Implement Sub for Money (panics on currency mismatch)
impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        self.checked_sub(&other)
            .expect("Currency mismatch in Money subtraction")
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        assert_eq!(
            self.currency, other.currency,
            "Currency mismatch in Money +="
        );
        self.amount += other.amount;
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Self) {
        assert_eq!(
            self.currency, other.currency,
            "Currency mismatch in Money -="
        );
        self.amount -= other.amount;
    }
}

impl Neg for Money {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            amount: -self.amount,
            currency: self.currency,
        }
    }
}

impl Mul<Decimal> for Money {
    type Output = Self;

    fn mul(self, scalar: Decimal) -> Self::Output {
        self.mul_scalar(scalar)
    }
}

impl Div<Decimal> for Money {
    type Output = Result<Self>;

    fn div(self, scalar: Decimal) -> Self::Output {
        self.div_scalar(scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_money_creation() {
        let money = Money::new(dec!(100.50), CurrencyCode::USD);
        assert_eq!(money.amount(), dec!(100.50));
        assert_eq!(money.currency(), CurrencyCode::USD);
    }

    #[test]
    fn test_money_zero() {
        let money = Money::zero(CurrencyCode::EUR);
        assert!(money.is_zero());
        assert!(!money.is_positive());
        assert!(!money.is_negative());
    }

    #[test]
    fn test_money_addition() {
        let m1 = Money::new(dec!(100), CurrencyCode::USD);
        let m2 = Money::new(dec!(50), CurrencyCode::USD);
        let result = m1 + m2;
        assert_eq!(result.amount(), dec!(150));
    }

    #[test]
    fn test_money_subtraction() {
        let m1 = Money::new(dec!(100), CurrencyCode::USD);
        let m2 = Money::new(dec!(30), CurrencyCode::USD);
        let result = m1 - m2;
        assert_eq!(result.amount(), dec!(70));
    }

    #[test]
    fn test_money_scalar_ops() {
        let m = Money::new(dec!(100), CurrencyCode::USD);
        let multiplied = m * dec!(1.5);
        assert_eq!(multiplied.amount(), dec!(150));

        let divided = m.div_scalar(dec!(4)).unwrap();
        assert_eq!(divided.amount(), dec!(25));
    }

    #[test]
    fn test_money_display() {
        let m = Money::new(dec!(1234.56), CurrencyCode::USD);
        assert_eq!(format!("{}", m), "USD 1234.56");
    }

    #[test]
    fn test_money_checked_add_currency_mismatch() {
        let usd = Money::new(dec!(100), CurrencyCode::USD);
        let eur = Money::new(dec!(100), CurrencyCode::EUR);
        assert!(usd.checked_add(&eur).is_err());
    }

    #[test]
    fn test_money_rounding() {
        let m = Money::new(dec!(100.5678), CurrencyCode::USD);
        let rounded = m.round(2);
        assert_eq!(rounded.amount(), dec!(100.57));
    }
}

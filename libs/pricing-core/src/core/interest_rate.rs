//! Interest rate calculations and compounding methods.

use crate::core::day_count::DayCountConvention;
use crate::core::error::{Error, Result};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use std::fmt;

/// Interest rate compounding method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compounding {
    /// Simple interest (no compounding).
    /// Formula: r * t
    Simple,

    /// Compounded k times per year.
    /// Formula: (1 + r/k)^(k*t) - 1
    Compounded(u32),

    /// Continuous compounding.
    /// Formula: e^(r*t) - 1
    Continuous,

    /// Simple compounding for short-term (< 1 year).
    /// Formula: 1 / (1 - r * t) - 1
    SimpleThenCompounded,
}

impl Compounding {
    /// Get the number of compounding periods per year.
    pub fn periods_per_year(&self) -> Option<u32> {
        match self {
            Compounding::Compounded(k) => Some(*k),
            _ => None,
        }
    }

    /// Check if this is continuous compounding.
    pub fn is_continuous(&self) -> bool {
        matches!(self, Compounding::Continuous)
    }

    /// Check if this is simple compounding.
    pub fn is_simple(&self) -> bool {
        matches!(self, Compounding::Simple)
    }
}

impl fmt::Display for Compounding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Compounding::Simple => write!(f, "Simple"),
            Compounding::Compounded(k) => write!(f, "Compounded({})", k),
            Compounding::Continuous => write!(f, "Continuous"),
            Compounding::SimpleThenCompounded => write!(f, "SimpleThenCompounded"),
        }
    }
}

/// Interest rate with associated compounding and day count convention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterestRate {
    rate: Decimal,
    compounding: Compounding,
    day_count: DayCountConvention,
}

impl InterestRate {
    /// Create a new interest rate.
    ///
    /// # Arguments
    ///
    /// * `rate` - The annual interest rate as a decimal (e.g., 0.05 for 5%).
    /// * `compounding` - The compounding method.
    /// * `day_count` - The day count convention.
    ///
    /// # Examples
    ///
    /// ```
    /// use pricing_core::prelude::*;
    /// use rust_decimal_macros::dec;
    ///
    /// let rate = InterestRate::new(
    ///     dec!(0.05),
    ///     Compounding::Continuous,
    ///     DayCountConvention::Act360,
    /// );
    /// ```
    pub fn new(rate: Decimal, compounding: Compounding, day_count: DayCountConvention) -> Self {
        Self {
            rate,
            compounding,
            day_count,
        }
    }

    /// Create a continuous rate with ACT/360 convention.
    pub fn continuous(rate: Decimal) -> Self {
        Self::new(rate, Compounding::Continuous, DayCountConvention::Act360)
    }

    /// Create a simple rate with ACT/360 convention.
    pub fn simple(rate: Decimal) -> Self {
        Self::new(rate, Compounding::Simple, DayCountConvention::Act360)
    }

    /// Create an annually compounded rate with ACT/ACT convention.
    pub fn annual(rate: Decimal) -> Self {
        Self::new(rate, Compounding::Compounded(1), DayCountConvention::ActAct)
    }

    /// Get the rate value.
    pub fn rate(&self) -> Decimal {
        self.rate
    }

    /// Get the compounding method.
    pub fn compounding(&self) -> Compounding {
        self.compounding
    }

    /// Get the day count convention.
    pub fn day_count(&self) -> DayCountConvention {
        self.day_count
    }

    /// Calculate the discount factor for a given time period.
    ///
    /// # Arguments
    ///
    /// * `time` - Time in years.
    ///
    /// # Returns
    ///
    /// The discount factor.
    pub fn discount_factor(&self, time: Decimal) -> Result<Decimal> {
        if time < Decimal::ZERO {
            return Err(Error::invalid_input("Time cannot be negative"));
        }

        let df = match self.compounding {
            Compounding::Simple => {
                let denominator = Decimal::ONE + self.rate * time;
                if denominator.is_zero() {
                    return Err(Error::arithmetic("Division by zero in discount factor"));
                }
                Decimal::ONE / denominator
            }
            Compounding::Compounded(k) => {
                let k_dec = Decimal::from(k);
                let base = Decimal::ONE + self.rate / k_dec;
                base.powd(-k_dec * time)
            }
            Compounding::Continuous => (-self.rate * time).exp(),
            Compounding::SimpleThenCompounded => {
                // Use simple for t <= 1, compounded otherwise
                if time <= Decimal::ONE {
                    Decimal::ONE / (Decimal::ONE + self.rate * time)
                } else {
                    let base = Decimal::ONE + self.rate;
                    base.powd(-time)
                }
            }
        };

        Ok(df)
    }

    /// Calculate the compound factor for a given time period.
    ///
    /// This is the inverse of the discount factor.
    pub fn compound_factor(&self, time: Decimal) -> Result<Decimal> {
        if time < Decimal::ZERO {
            return Err(Error::invalid_input("Time cannot be negative"));
        }

        let cf = match self.compounding {
            Compounding::Simple => Decimal::ONE + self.rate * time,
            Compounding::Compounded(k) => {
                let k_dec = Decimal::from(k);
                let base = Decimal::ONE + self.rate / k_dec;
                base.powd(k_dec * time)
            }
            Compounding::Continuous => (self.rate * time).exp(),
            Compounding::SimpleThenCompounded => {
                if time <= Decimal::ONE {
                    Decimal::ONE + self.rate * time
                } else {
                    let base = Decimal::ONE + self.rate;
                    base.powd(time)
                }
            }
        };

        Ok(cf)
    }

    /// Convert this rate to a different compounding method.
    ///
    /// The conversion preserves the equivalent rate for the given time period.
    pub fn to_compounding(&self, new_compounding: Compounding, time: Decimal) -> Result<Self> {
        if time <= Decimal::ZERO {
            return Err(Error::invalid_input("Time must be positive for conversion"));
        }

        // Calculate the compound factor with current compounding
        let cf = self.compound_factor(time)?;

        // Calculate the equivalent rate with new compounding
        let new_rate = match new_compounding {
            Compounding::Simple => (cf - Decimal::ONE) / time,
            Compounding::Compounded(k) => {
                let k_dec = Decimal::from(k);
                let base = cf.powd(Decimal::ONE / (k_dec * time));
                (base - Decimal::ONE) * k_dec
            }
            Compounding::Continuous => cf.ln() / time,
            Compounding::SimpleThenCompounded => {
                if time <= Decimal::ONE {
                    (cf - Decimal::ONE) / time
                } else {
                    cf.powd(Decimal::ONE / time) - Decimal::ONE
                }
            }
        };

        Ok(Self {
            rate: new_rate,
            compounding: new_compounding,
            day_count: self.day_count,
        })
    }

    /// Calculate the forward rate between two time points.
    pub fn forward_rate(&self, t1: Decimal, t2: Decimal) -> Result<Decimal> {
        if t2 <= t1 {
            return Err(Error::invalid_input("t2 must be greater than t1"));
        }

        let df1 = self.discount_factor(t1)?;
        let df2 = self.discount_factor(t2)?;

        let forward = (df1 / df2 - Decimal::ONE) / (t2 - t1);
        Ok(forward)
    }

    /// Calculate the implied rate from a discount factor.
    ///
    /// # Arguments
    ///
    /// * `df` - The discount factor.
    /// * `time` - Time in years.
    /// * `compounding` - The compounding method to use.
    pub fn from_discount_factor(
        df: Decimal,
        time: Decimal,
        compounding: Compounding,
    ) -> Result<Self> {
        if df <= Decimal::ZERO {
            return Err(Error::invalid_input("Discount factor must be positive"));
        }
        if time <= Decimal::ZERO {
            return Err(Error::invalid_input("Time must be positive"));
        }

        let rate = match compounding {
            Compounding::Simple => (Decimal::ONE / df - Decimal::ONE) / time,
            Compounding::Compounded(k) => {
                let k_dec = Decimal::from(k);
                let base = df.powd(-Decimal::ONE / (k_dec * time));
                (base - Decimal::ONE) * k_dec
            }
            Compounding::Continuous => -(df.ln()) / time,
            Compounding::SimpleThenCompounded => {
                if time <= Decimal::ONE {
                    (Decimal::ONE / df - Decimal::ONE) / time
                } else {
                    df.powd(-Decimal::ONE / time) - Decimal::ONE
                }
            }
        };

        Ok(Self::new(rate, compounding, DayCountConvention::Act360))
    }
}

impl fmt::Display for InterestRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.4}% {} {:?}",
            self.rate * Decimal::from(100),
            self.compounding,
            self.day_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_rate_creation() {
        let rate = InterestRate::continuous(dec!(0.05));
        assert_eq!(rate.rate(), dec!(0.05));
        assert!(rate.compounding().is_continuous());
    }

    #[test]
    fn test_discount_factor_continuous() {
        let rate = InterestRate::continuous(dec!(0.05));
        let df = rate.discount_factor(dec!(1)).unwrap();
        // e^(-0.05) ≈ 0.951229
        assert!(df > dec!(0.95) && df < dec!(0.96));
    }

    #[test]
    fn test_discount_factor_simple() {
        let rate = InterestRate::simple(dec!(0.10));
        let df = rate.discount_factor(dec!(1)).unwrap();
        assert_eq!(df, dec!(0.9090909090909090909090909091));
    }

    #[test]
    fn test_compound_factor() {
        let rate = InterestRate::continuous(dec!(0.05));
        let cf = rate.compound_factor(dec!(1)).unwrap();
        let df = rate.discount_factor(dec!(1)).unwrap();
        assert!((cf * df - Decimal::ONE).abs() < dec!(0.0001));
    }

    #[test]
    fn test_conversion_to_continuous() {
        let annual_rate = InterestRate::annual(dec!(0.05));
        let continuous_rate = annual_rate
            .to_compounding(Compounding::Continuous, dec!(1))
            .unwrap();
        assert!(continuous_rate.rate() > dec!(0.04) && continuous_rate.rate() < dec!(0.06));
    }

    #[test]
    fn test_from_discount_factor() {
        let df = dec!(0.95);
        let rate =
            InterestRate::from_discount_factor(df, dec!(1), Compounding::Continuous).unwrap();
        let calculated_df = rate.discount_factor(dec!(1)).unwrap();
        assert!((calculated_df - df).abs() < dec!(0.0001));
    }

    #[test]
    fn test_forward_rate() {
        let rate = InterestRate::continuous(dec!(0.05));
        let forward = rate.forward_rate(dec!(1), dec!(2)).unwrap();
        assert!(forward > Decimal::ZERO);
    }
}

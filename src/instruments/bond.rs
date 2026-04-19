//! Bond instruments.

use crate::core::day_count::DayCountConvention;
use crate::core::error::{Error, Result};
use crate::core::interest_rate::InterestRate;
use crate::core::money::Money;
use crate::core::traits::{AsAny, CashFlowGenerating, HasYield, Instrument};
use chrono::{Datelike, NaiveDate};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use std::fmt;

/// A generic bond trait.
pub trait Bond: Instrument + CashFlowGenerating + HasYield {
    /// Get the face value of the bond.
    fn face_value(&self) -> Money;

    /// Get the coupon rate.
    fn coupon_rate(&self) -> Decimal;

    /// Get the maturity date.
    fn maturity_date(&self) -> NaiveDate;

    /// Get the issue date.
    fn issue_date(&self) -> NaiveDate;

    /// Check if the bond has coupon payments.
    fn has_coupons(&self) -> bool;

    /// Calculate the accrued interest.
    fn accrued_interest(&self, settlement_date: NaiveDate) -> Result<Money>;

    /// Calculate the clean price from dirty price.
    fn clean_price(&self, dirty_price: Money, settlement_date: NaiveDate) -> Result<Money>;

    /// Calculate the dirty price from clean price.
    fn dirty_price(&self, clean_price: Money, settlement_date: NaiveDate) -> Result<Money>;
}

/// Zero-coupon bond.
#[derive(Debug, Clone, PartialEq)]
pub struct ZeroCouponBond {
    face_value: Money,
    maturity_date: NaiveDate,
    issue_date: NaiveDate,
    day_count: DayCountConvention,
}

impl ZeroCouponBond {
    /// Create a new zero-coupon bond.
    ///
    /// # Arguments
    ///
    /// * `face_value` - The face value to be paid at maturity.
    /// * `issue_date` - The issue date.
    /// * `maturity_date` - The maturity date.
    /// * `day_count` - The day count convention.
    ///
    /// # Errors
    ///
    /// Returns an error if maturity is not after issue date.
    pub fn new(
        face_value: Money,
        issue_date: NaiveDate,
        maturity_date: NaiveDate,
        day_count: DayCountConvention,
    ) -> Result<Self> {
        if maturity_date <= issue_date {
            return Err(Error::invalid_input("Maturity must be after issue date"));
        }

        Ok(Self {
            face_value,
            maturity_date,
            issue_date,
            day_count,
        })
    }

    /// Price the bond using a given yield.
    ///
    /// # Arguments
    ///
    /// * `yield_rate` - The yield to use for discounting.
    /// * `pricing_date` - The date at which to price.
    pub fn price_with_yield(
        &self,
        yield_rate: &InterestRate,
        pricing_date: NaiveDate,
    ) -> Result<Money> {
        if pricing_date >= self.maturity_date {
            return Ok(Money::zero(self.face_value.currency()));
        }

        let time_to_maturity = self
            .day_count
            .year_fraction(pricing_date, self.maturity_date);

        let df = yield_rate.discount_factor(time_to_maturity)?;
        let price = self.face_value.amount() * df;

        Ok(Money::new(price, self.face_value.currency()))
    }

    /// Get the day count convention.
    pub fn day_count_convention(&self) -> DayCountConvention {
        self.day_count
    }
}

impl AsAny for ZeroCouponBond {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Instrument for ZeroCouponBond {
    fn notional(&self) -> Money {
        self.face_value
    }

    fn maturity(&self) -> Option<NaiveDate> {
        Some(self.maturity_date)
    }

    fn instrument_type(&self) -> &'static str {
        "ZeroCouponBond"
    }
}

impl CashFlowGenerating for ZeroCouponBond {
    fn cash_flows(&self) -> Vec<(NaiveDate, Money)> {
        vec![(self.maturity_date, self.face_value)]
    }

    fn next_cash_flow_date(&self, after: NaiveDate) -> Option<NaiveDate> {
        if after < self.maturity_date {
            Some(self.maturity_date)
        } else {
            None
        }
    }
}

impl HasYield for ZeroCouponBond {
    fn yield_to_maturity(&self, market_price: Money, _guess: Option<f64>) -> Result<f64> {
        if market_price.is_zero() {
            return Err(Error::pricing("Market price cannot be zero"));
        }

        if market_price.currency() != self.face_value.currency() {
            return Err(Error::currency_mismatch(
                self.face_value.currency().as_str(),
                market_price.currency().as_str(),
            ));
        }

        // For zero coupon: price = face_value * exp(-y * t)
        // y = -ln(price/face_value) / t
        let t = self
            .day_count
            .year_fraction(self.issue_date, self.maturity_date);

        if t.is_zero() {
            return Err(Error::pricing("Time to maturity is zero"));
        }

        let price_ratio = market_price.amount() / self.face_value.amount();
        let yield_rate = -(price_ratio.ln()) / t;

        yield_rate
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Failed to convert yield to f64"))
    }

    fn current_yield(&self, market_price: Money) -> Result<f64> {
        // Zero coupon bonds have no current yield
        if market_price.is_zero() {
            return Err(Error::pricing("Market price cannot be zero"));
        }

        Ok(0.0)
    }
}

impl Bond for ZeroCouponBond {
    fn face_value(&self) -> Money {
        self.face_value
    }

    fn coupon_rate(&self) -> Decimal {
        Decimal::ZERO
    }

    fn maturity_date(&self) -> NaiveDate {
        self.maturity_date
    }

    fn issue_date(&self) -> NaiveDate {
        self.issue_date
    }

    fn has_coupons(&self) -> bool {
        false
    }

    fn accrued_interest(&self, _settlement_date: NaiveDate) -> Result<Money> {
        // No coupons, so no accrued interest
        Ok(Money::zero(self.face_value.currency()))
    }

    fn clean_price(&self, dirty_price: Money, _settlement_date: NaiveDate) -> Result<Money> {
        // Same as dirty price for zero coupon
        Ok(dirty_price)
    }

    fn dirty_price(&self, clean_price: Money, _settlement_date: NaiveDate) -> Result<Money> {
        // Same as clean price for zero coupon
        Ok(clean_price)
    }
}

impl fmt::Display for ZeroCouponBond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ZeroCouponBond({} {}, maturity: {})",
            self.face_value.currency(),
            self.face_value.amount(),
            self.maturity_date
        )
    }
}

/// Coupon-bearing bond with fixed coupons.
#[derive(Debug, Clone, PartialEq)]
pub struct CouponBond {
    face_value: Money,
    coupon_rate: Decimal,
    maturity_date: NaiveDate,
    issue_date: NaiveDate,
    coupon_frequency: u8, // coupons per year
    day_count: DayCountConvention,
}

impl CouponBond {
    /// Create a new coupon bond.
    ///
    /// # Arguments
    ///
    /// * `face_value` - The face value.
    /// * `coupon_rate` - Annual coupon rate as decimal.
    /// * `issue_date` - Issue date.
    /// * `maturity_date` - Maturity date.
    /// * `coupon_frequency` - Coupons per year (1=annual, 2=semi-annual, 4=quarterly).
    /// * `day_count` - Day count convention.
    pub fn new(
        face_value: Money,
        coupon_rate: Decimal,
        issue_date: NaiveDate,
        maturity_date: NaiveDate,
        coupon_frequency: u8,
        day_count: DayCountConvention,
    ) -> Result<Self> {
        if maturity_date <= issue_date {
            return Err(Error::invalid_input("Maturity must be after issue date"));
        }

        if ![1, 2, 4, 12].contains(&coupon_frequency) {
            return Err(Error::invalid_input(
                "Coupon frequency must be 1, 2, 4, or 12",
            ));
        }

        Ok(Self {
            face_value,
            coupon_rate,
            maturity_date,
            issue_date,
            coupon_frequency,
            day_count,
        })
    }

    /// Get the coupon frequency.
    pub fn coupon_frequency(&self) -> u8 {
        self.coupon_frequency
    }

    /// Calculate the coupon amount per period.
    pub fn coupon_amount(&self) -> Money {
        let amount =
            self.face_value.amount() * self.coupon_rate / Decimal::from(self.coupon_frequency);
        Money::new(amount, self.face_value.currency())
    }

    /// Price the bond using a given yield.
    pub fn price_with_yield(
        &self,
        yield_rate: &InterestRate,
        pricing_date: NaiveDate,
    ) -> Result<Money> {
        if pricing_date >= self.maturity_date {
            return Ok(Money::zero(self.face_value.currency()));
        }

        let cash_flows = self.cash_flows();
        let mut pv = Decimal::ZERO;

        for (date, amount) in cash_flows {
            if date <= pricing_date {
                continue;
            }

            let t = self.day_count.year_fraction(pricing_date, date);
            let df = yield_rate.discount_factor(t)?;
            pv += amount.amount() * df;
        }

        Ok(Money::new(pv, self.face_value.currency()))
    }

    /// Calculate Macaulay duration.
    pub fn macaulay_duration(
        &self,
        yield_rate: &InterestRate,
        pricing_date: NaiveDate,
    ) -> Result<Decimal> {
        let price = self.price_with_yield(yield_rate, pricing_date)?;

        if price.is_zero() {
            return Err(Error::arithmetic(
                "Price is zero, cannot calculate duration",
            ));
        }

        let cash_flows = self.cash_flows();
        let mut weighted_time = Decimal::ZERO;

        for (date, amount) in cash_flows {
            if date <= pricing_date {
                continue;
            }

            let t = self.day_count.year_fraction(pricing_date, date);
            let df = yield_rate.discount_factor(t)?;
            weighted_time += t * amount.amount() * df;
        }

        Ok(weighted_time / price.amount())
    }

    /// Calculate modified duration.
    pub fn modified_duration(
        &self,
        yield_rate: &InterestRate,
        pricing_date: NaiveDate,
    ) -> Result<Decimal> {
        let mac_duration = self.macaulay_duration(yield_rate, pricing_date)?;

        // For continuous compounding, modified = macaulay
        // Otherwise: modified = macaulay / (1 + y/k)
        let adjustment = match yield_rate.compounding() {
            crate::core::interest_rate::Compounding::Continuous => Decimal::ONE,
            crate::core::interest_rate::Compounding::Compounded(k) => {
                Decimal::ONE + yield_rate.rate() / Decimal::from(k)
            }
            _ => Decimal::ONE + yield_rate.rate(),
        };

        Ok(mac_duration / adjustment)
    }

    /// Get the day count convention.
    pub fn day_count_convention(&self) -> DayCountConvention {
        self.day_count
    }

    /// Generate coupon payment dates.
    fn generate_coupon_dates(&self) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let months_between = 12 / self.coupon_frequency as i32;

        // Work backwards from maturity
        let mut current = self.maturity_date;
        while current > self.issue_date {
            // Handle month boundaries properly
            let year = current.year();
            let month = current.month() as i32 - months_between;

            let (new_year, new_month) = if month <= 0 {
                (year - 1, (month + 12) as u32)
            } else {
                (year, month as u32)
            };

            current = match NaiveDate::from_ymd_opt(new_year, new_month, current.day()) {
                Some(d) => d,
                None => {
                    // Handle end of month
                    NaiveDate::from_ymd_opt(new_year, new_month + 1, 1)
                        .unwrap()
                        .pred_opt()
                        .unwrap()
                }
            };

            if current >= self.issue_date {
                dates.push(current);
            }
        }

        dates.reverse();
        dates
    }
}

impl AsAny for CouponBond {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Instrument for CouponBond {
    fn notional(&self) -> Money {
        self.face_value
    }

    fn maturity(&self) -> Option<NaiveDate> {
        Some(self.maturity_date)
    }

    fn instrument_type(&self) -> &'static str {
        "CouponBond"
    }
}

impl CashFlowGenerating for CouponBond {
    fn cash_flows(&self) -> Vec<(NaiveDate, Money)> {
        let mut flows = Vec::new();
        let coupon_dates = self.generate_coupon_dates();
        let coupon_amount = self.coupon_amount();

        for date in coupon_dates {
            flows.push((date, coupon_amount));
        }

        // Add principal at maturity
        flows.push((self.maturity_date, self.face_value));

        flows
    }

    fn next_cash_flow_date(&self, after: NaiveDate) -> Option<NaiveDate> {
        let coupon_dates = self.generate_coupon_dates();
        coupon_dates.into_iter().find(|&d| d > after)
    }
}

impl HasYield for CouponBond {
    fn yield_to_maturity(&self, market_price: Money, guess: Option<f64>) -> Result<f64> {
        // Newton-Raphson method to find YTM
        let mut y = guess.unwrap_or(self.coupon_rate.to_f64().unwrap_or(0.05));
        let tolerance = 1e-10;
        let max_iterations = 100;

        for _ in 0..max_iterations {
            let rate = InterestRate::new(
                Decimal::try_from(y).map_err(|_| Error::arithmetic("Invalid rate"))?,
                crate::core::interest_rate::Compounding::Continuous,
                self.day_count,
            );

            let price = self.price_with_yield(&rate, self.issue_date)?;
            let error = price.amount() - market_price.amount();

            if error.abs() < Decimal::from_f64(tolerance).unwrap() {
                return Ok(y);
            }

            // Calculate numerical derivative
            let dy = 1e-7;
            let rate_dy = InterestRate::new(
                Decimal::try_from(y + dy).map_err(|_| Error::arithmetic("Invalid rate"))?,
                crate::core::interest_rate::Compounding::Continuous,
                self.day_count,
            );
            let price_dy = self.price_with_yield(&rate_dy, self.issue_date)?;
            let derivative = (price_dy.amount() - price.amount()) / Decimal::try_from(dy).unwrap();

            if derivative.abs() < Decimal::from_f64(1e-15).unwrap() {
                return Err(Error::arithmetic("Derivative too small in YTM calculation"));
            }

            y -= (error / derivative).to_f64().unwrap_or(0.0);

            if y < -1.0 || y > 2.0 {
                return Err(Error::pricing("YTM calculation diverged"));
            }
        }

        Err(Error::pricing("YTM calculation did not converge"))
    }

    fn current_yield(&self, market_price: Money) -> Result<f64> {
        if market_price.is_zero() {
            return Err(Error::pricing("Market price cannot be zero"));
        }

        let annual_coupon = self.coupon_rate * self.face_value.amount();
        let current_yield = annual_coupon / market_price.amount();

        current_yield
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Failed to convert yield to f64"))
    }
}

impl Bond for CouponBond {
    fn face_value(&self) -> Money {
        self.face_value
    }

    fn coupon_rate(&self) -> Decimal {
        self.coupon_rate
    }

    fn maturity_date(&self) -> NaiveDate {
        self.maturity_date
    }

    fn issue_date(&self) -> NaiveDate {
        self.issue_date
    }

    fn has_coupons(&self) -> bool {
        true
    }

    fn accrued_interest(&self, settlement_date: NaiveDate) -> Result<Money> {
        let coupon_dates = self.generate_coupon_dates();

        // Find the previous coupon date
        let prev_coupon = coupon_dates
            .iter()
            .filter(|&&d| d <= settlement_date)
            .last()
            .copied()
            .unwrap_or(self.issue_date);

        let days_accrued = self.day_count.day_count(prev_coupon, settlement_date);
        let days_in_period = self.day_count.day_count(
            prev_coupon,
            self.next_cash_flow_date(prev_coupon)
                .unwrap_or(self.maturity_date),
        );

        if days_in_period == 0 {
            return Ok(Money::zero(self.face_value.currency()));
        }

        let accrued = self.coupon_amount().amount() * Decimal::from(days_accrued)
            / Decimal::from(days_in_period);

        Ok(Money::new(accrued, self.face_value.currency()))
    }

    fn clean_price(&self, dirty_price: Money, settlement_date: NaiveDate) -> Result<Money> {
        let accrued = self.accrued_interest(settlement_date)?;
        dirty_price.checked_sub(&accrued)
    }

    fn dirty_price(&self, clean_price: Money, settlement_date: NaiveDate) -> Result<Money> {
        let accrued = self.accrued_interest(settlement_date)?;
        clean_price.checked_add(&accrued)
    }
}

impl fmt::Display for CouponBond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CouponBond({} {}, coupon: {}%, maturity: {})",
            self.face_value.currency(),
            self.face_value.amount(),
            self.coupon_rate * Decimal::from(100),
            self.maturity_date
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::currency::CurrencyCode;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_zero_coupon_bond_creation() {
        let bond = ZeroCouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            date(2024, 1, 1),
            date(2025, 1, 1),
            DayCountConvention::Act360,
        );
        assert!(bond.is_ok());
    }

    #[test]
    fn test_zero_coupon_bond_invalid_dates() {
        let bond = ZeroCouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            date(2025, 1, 1),
            date(2024, 1, 1),
            DayCountConvention::Act360,
        );
        assert!(bond.is_err());
    }

    #[test]
    fn test_zero_coupon_pricing() {
        let bond = ZeroCouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            date(2024, 1, 1),
            date(2025, 1, 1),
            DayCountConvention::Act360,
        )
        .unwrap();

        let yield_rate = InterestRate::continuous(dec!(0.05));
        let price = bond
            .price_with_yield(&yield_rate, date(2024, 1, 1))
            .unwrap();

        // 1000 * exp(-0.05) ≈ 951.23
        assert!(price.amount() > dec!(950) && price.amount() < dec!(952));
    }

    #[test]
    fn test_coupon_bond_creation() {
        let bond = CouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            dec!(0.05),
            date(2024, 1, 1),
            date(2029, 1, 1),
            2,
            DayCountConvention::Act360,
        );
        assert!(bond.is_ok());
    }

    #[test]
    fn test_coupon_amount() {
        let bond = CouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            dec!(0.06),
            date(2024, 1, 1),
            date(2029, 1, 1),
            2,
            DayCountConvention::Act360,
        )
        .unwrap();

        let coupon = bond.coupon_amount();
        assert_eq!(coupon.amount(), dec!(30)); // 1000 * 0.06 / 2
    }

    #[test]
    fn test_coupon_bond_cash_flows() {
        let bond = CouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            dec!(0.06),
            date(2024, 1, 1),
            date(2026, 1, 1),
            2,
            DayCountConvention::Act360,
        )
        .unwrap();

        let flows = bond.cash_flows();
        assert_eq!(flows.len(), 5); // 4 coupons + principal
    }

    #[test]
    fn test_coupon_bond_ytm() {
        let bond = CouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            dec!(0.05),
            date(2024, 1, 1),
            date(2029, 1, 1),
            2,
            DayCountConvention::Act360,
        )
        .unwrap();

        // Price at par should give YTM ≈ coupon rate
        // Note: Using continuous compounding means YTM won't exactly equal coupon rate
        let price = Money::new(dec!(1000), CurrencyCode::USD);
        let ytm = bond.yield_to_maturity(price, None).unwrap();

        assert!((ytm - 0.05).abs() < 0.01, "YTM was {}", ytm);
    }

    #[test]
    fn test_accrued_interest() {
        let bond = CouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            dec!(0.06),
            date(2024, 1, 1),
            date(2029, 1, 1),
            2,
            DayCountConvention::Thirty360,
        )
        .unwrap();

        // Halfway through coupon period
        let accrued = bond.accrued_interest(date(2024, 4, 1)).unwrap();
        assert!(accrued.amount() > Decimal::ZERO);
    }
}

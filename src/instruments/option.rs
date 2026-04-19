//! Option instruments and types.

use crate::core::currency::CurrencyCode;
use crate::core::error::Result;
use crate::core::money::Money;
use crate::core::traits::{Instrument, Optionable};
use chrono::NaiveDate;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::fmt;

/// Option type: Call or Put.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionType {
    /// Call option - right to buy.
    Call,
    /// Put option - right to sell.
    Put,
}

impl OptionType {
    /// Check if this is a call option.
    pub fn is_call(&self) -> bool {
        matches!(self, OptionType::Call)
    }

    /// Check if this is a put option.
    pub fn is_put(&self) -> bool {
        matches!(self, OptionType::Put)
    }

    /// Get the opposite option type.
    pub fn opposite(&self) -> Self {
        match self {
            OptionType::Call => OptionType::Put,
            OptionType::Put => OptionType::Call,
        }
    }
}

impl fmt::Display for OptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionType::Call => write!(f, "Call"),
            OptionType::Put => write!(f, "Put"),
        }
    }
}

/// Exercise style for options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExerciseStyle {
    /// European style - can only exercise at expiration.
    European,
    /// American style - can exercise any time before expiration.
    American,
    /// Bermudan style - can exercise at specific dates.
    Bermudan,
}

impl ExerciseStyle {
    /// Check if this is European style.
    pub fn is_european(&self) -> bool {
        matches!(self, ExerciseStyle::European)
    }

    /// Check if this is American style.
    pub fn is_american(&self) -> bool {
        matches!(self, ExerciseStyle::American)
    }
}

/// European option (can only be exercised at expiration).
#[derive(Debug, Clone, PartialEq)]
pub struct EuropeanOption {
    strike: Decimal,
    spot: Decimal,
    risk_free_rate: Decimal,
    volatility: Decimal,
    time_to_expiry: f64,
    option_type: OptionType,
    underlying_currency: CurrencyCode,
}

impl EuropeanOption {
    /// Create a new European option.
    ///
    /// # Arguments
    ///
    /// * `strike` - The strike price.
    /// * `spot` - The current spot price of the underlying.
    /// * `risk_free_rate` - The annual risk-free rate as a decimal.
    /// * `volatility` - The annual volatility as a decimal.
    /// * `time_to_expiry` - Time to expiration in years.
    /// * `option_type` - Call or Put.
    pub fn new(
        strike: Decimal,
        spot: Decimal,
        risk_free_rate: Decimal,
        volatility: Decimal,
        time_to_expiry: f64,
        option_type: OptionType,
    ) -> Self {
        Self {
            strike,
            spot,
            risk_free_rate,
            volatility,
            time_to_expiry,
            option_type,
            underlying_currency: CurrencyCode::USD,
        }
    }

    /// Create a new European option with a specific underlying currency.
    pub fn new_with_currency(
        strike: Decimal,
        spot: Decimal,
        risk_free_rate: Decimal,
        volatility: Decimal,
        time_to_expiry: f64,
        option_type: OptionType,
        currency: CurrencyCode,
    ) -> Self {
        Self {
            strike,
            spot,
            risk_free_rate,
            volatility,
            time_to_expiry,
            option_type,
            underlying_currency: currency,
        }
    }

    /// Get the strike price.
    pub fn strike(&self) -> Decimal {
        self.strike
    }

    /// Get the spot price.
    pub fn spot(&self) -> Decimal {
        self.spot
    }

    /// Get the risk-free rate.
    pub fn risk_free_rate(&self) -> Decimal {
        self.risk_free_rate
    }

    /// Get the volatility.
    pub fn volatility(&self) -> Decimal {
        self.volatility
    }

    /// Get the time to expiry.
    pub fn time_to_expiry(&self) -> f64 {
        self.time_to_expiry
    }

    /// Get the option type.
    pub fn option_type(&self) -> OptionType {
        self.option_type
    }

    /// Get the underlying currency.
    pub fn underlying_currency(&self) -> CurrencyCode {
        self.underlying_currency
    }

    /// Calculate moneyness (spot / strike).
    pub fn moneyness(&self) -> Decimal {
        self.spot / self.strike
    }

    /// Check if the option is in-the-money.
    pub fn is_in_the_money(&self) -> bool {
        match self.option_type {
            OptionType::Call => self.spot > self.strike,
            OptionType::Put => self.spot < self.strike,
        }
    }

    /// Check if the option is out-of-the-money.
    pub fn is_out_of_the_money(&self) -> bool {
        match self.option_type {
            OptionType::Call => self.spot < self.strike,
            OptionType::Put => self.spot > self.strike,
        }
    }

    /// Check if the option is at-the-money.
    pub fn is_at_the_money(&self) -> bool {
        (self.spot - self.strike).abs() < Decimal::from_f64(1e-10).unwrap()
    }

    /// Calculate the intrinsic value.
    pub fn intrinsic_value(&self) -> Money {
        let value = match self.option_type {
            OptionType::Call => (self.spot - self.strike).max(Decimal::ZERO),
            OptionType::Put => (self.strike - self.spot).max(Decimal::ZERO),
        };
        Money::new(value, self.underlying_currency)
    }

    /// Calculate the time value (price - intrinsic value).
    pub fn time_value(&self, option_price: Money) -> Result<Money> {
        let intrinsic = self.intrinsic_value();
        option_price.checked_sub(&intrinsic)
    }

    /// Update the spot price and return a new option.
    pub fn with_spot(&self, spot: Decimal) -> Self {
        Self {
            spot,
            ..self.clone()
        }
    }

    /// Update the volatility and return a new option.
    pub fn with_volatility(&self, volatility: Decimal) -> Self {
        Self {
            volatility,
            ..self.clone()
        }
    }

    /// Update the time to expiry and return a new option.
    pub fn with_time_to_expiry(&self, time_to_expiry: f64) -> Self {
        Self {
            time_to_expiry,
            ..self.clone()
        }
    }
}

impl Instrument for EuropeanOption {
    fn notional(&self) -> Money {
        // Options typically have a notional of 1 unit of underlying
        Money::new(Decimal::ONE, self.underlying_currency)
    }

    fn maturity(&self) -> Option<NaiveDate> {
        // We don't have a calendar date, just time to expiry
        None
    }

    fn instrument_type(&self) -> &'static str {
        "EuropeanOption"
    }
}

impl Optionable for EuropeanOption {
    fn is_exercisable(&self, _date: NaiveDate) -> bool {
        // European options can only be exercised at expiry
        // Without a specific calendar date, we assume only at maturity
        false
    }

    fn intrinsic_value(&self, underlying_price: Money) -> Money {
        let spot = underlying_price.amount();
        let value = match self.option_type {
            OptionType::Call => (spot - self.strike).max(Decimal::ZERO),
            OptionType::Put => (self.strike - spot).max(Decimal::ZERO),
        };
        Money::new(value, underlying_price.currency())
    }

    fn is_in_the_money(&self, underlying_price: Money) -> bool {
        match self.option_type {
            OptionType::Call => underlying_price.amount() > self.strike,
            OptionType::Put => underlying_price.amount() < self.strike,
        }
    }
}

impl fmt::Display for EuropeanOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "European {} Option: strike={}, spot={}, T={:.2}y, vol={:.1}%",
            self.option_type,
            self.strike,
            self.spot,
            self.time_to_expiry,
            self.volatility * Decimal::from(100)
        )
    }
}

/// American option (can be exercised any time before expiration).
#[derive(Debug, Clone, PartialEq)]
pub struct AmericanOption {
    strike: Decimal,
    spot: Decimal,
    risk_free_rate: Decimal,
    volatility: Decimal,
    time_to_expiry: f64,
    dividend_yield: Decimal,
    option_type: OptionType,
    underlying_currency: CurrencyCode,
}

impl AmericanOption {
    /// Create a new American option without dividends.
    pub fn new(
        strike: Decimal,
        spot: Decimal,
        risk_free_rate: Decimal,
        volatility: Decimal,
        time_to_expiry: f64,
        option_type: OptionType,
    ) -> Self {
        Self {
            strike,
            spot,
            risk_free_rate,
            volatility,
            time_to_expiry,
            dividend_yield: Decimal::ZERO,
            option_type,
            underlying_currency: CurrencyCode::USD,
        }
    }

    /// Create a new American option with dividend yield.
    pub fn new_with_dividends(
        strike: Decimal,
        spot: Decimal,
        risk_free_rate: Decimal,
        volatility: Decimal,
        time_to_expiry: f64,
        dividend_yield: Decimal,
        option_type: OptionType,
    ) -> Self {
        Self {
            strike,
            spot,
            risk_free_rate,
            volatility,
            time_to_expiry,
            dividend_yield,
            option_type,
            underlying_currency: CurrencyCode::USD,
        }
    }

    /// Get the strike price.
    pub fn strike(&self) -> Decimal {
        self.strike
    }

    /// Get the spot price.
    pub fn spot(&self) -> Decimal {
        self.spot
    }

    /// Get the risk-free rate.
    pub fn risk_free_rate(&self) -> Decimal {
        self.risk_free_rate
    }

    /// Get the volatility.
    pub fn volatility(&self) -> Decimal {
        self.volatility
    }

    /// Get the time to expiry.
    pub fn time_to_expiry(&self) -> f64 {
        self.time_to_expiry
    }

    /// Get the dividend yield.
    pub fn dividend_yield(&self) -> Decimal {
        self.dividend_yield
    }

    /// Get the option type.
    pub fn option_type(&self) -> OptionType {
        self.option_type
    }

    /// Calculate intrinsic value.
    pub fn intrinsic_value(&self) -> Money {
        let value = match self.option_type {
            OptionType::Call => (self.spot - self.strike).max(Decimal::ZERO),
            OptionType::Put => (self.strike - self.spot).max(Decimal::ZERO),
        };
        Money::new(value, self.underlying_currency)
    }

    /// Get the underlying currency.
    pub fn underlying_currency(&self) -> CurrencyCode {
        self.underlying_currency
    }

    /// Calculate cost of carry (b = r - q).
    pub fn cost_of_carry(&self) -> Decimal {
        self.risk_free_rate - self.dividend_yield
    }

    /// Update the spot price and return a new option.
    pub fn with_spot(&self, spot: Decimal) -> Self {
        Self {
            spot,
            ..self.clone()
        }
    }

    /// Update the volatility and return a new option.
    pub fn with_volatility(&self, volatility: Decimal) -> Self {
        Self {
            volatility,
            ..self.clone()
        }
    }

    /// Update the time to expiry and return a new option.
    pub fn with_time_to_expiry(&self, time_to_expiry: f64) -> Self {
        Self {
            time_to_expiry,
            ..self.clone()
        }
    }
}

impl Instrument for AmericanOption {
    fn notional(&self) -> Money {
        Money::new(Decimal::ONE, self.underlying_currency)
    }

    fn maturity(&self) -> Option<NaiveDate> {
        None
    }

    fn instrument_type(&self) -> &'static str {
        "AmericanOption"
    }
}

impl Optionable for AmericanOption {
    fn is_exercisable(&self, _date: NaiveDate) -> bool {
        // American options can always be exercised before expiry
        true
    }

    fn intrinsic_value(&self, underlying_price: Money) -> Money {
        let spot = underlying_price.amount();
        let value = match self.option_type {
            OptionType::Call => (spot - self.strike).max(Decimal::ZERO),
            OptionType::Put => (self.strike - spot).max(Decimal::ZERO),
        };
        Money::new(value, underlying_price.currency())
    }

    fn is_in_the_money(&self, underlying_price: Money) -> bool {
        match self.option_type {
            OptionType::Call => underlying_price.amount() > self.strike,
            OptionType::Put => underlying_price.amount() < self.strike,
        }
    }
}

/// Option payoff at expiration.
#[derive(Debug, Clone, PartialEq)]
pub struct OptionPayoff {
    /// The payoff amount.
    pub amount: Money,
    /// Whether the option expired in-the-money.
    pub in_the_money: bool,
}

/// Calculate the payoff of an option at expiration.
pub fn calculate_payoff(
    spot: Decimal,
    strike: Decimal,
    option_type: OptionType,
    notional: Decimal,
    currency: CurrencyCode,
) -> OptionPayoff {
    let payoff_amount = match option_type {
        OptionType::Call => (spot - strike).max(Decimal::ZERO),
        OptionType::Put => (strike - spot).max(Decimal::ZERO),
    } * notional;

    OptionPayoff {
        amount: Money::new(payoff_amount, currency),
        in_the_money: payoff_amount > Decimal::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_european_option_creation() {
        let option = EuropeanOption::new(
            dec!(100),  // strike
            dec!(105),  // spot
            dec!(0.05), // risk-free rate
            dec!(0.2),  // volatility
            1.0,        // time to expiry
            OptionType::Call,
        );

        assert_eq!(option.strike(), dec!(100));
        assert_eq!(option.spot(), dec!(105));
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_option_type() {
        let call = OptionType::Call;
        let put = OptionType::Put;

        assert!(call.is_call());
        assert!(!call.is_put());
        assert!(put.is_put());
        assert!(!put.is_call());

        assert_eq!(call.opposite(), OptionType::Put);
        assert_eq!(put.opposite(), OptionType::Call);
    }

    #[test]
    fn test_intrinsic_value() {
        let call_itm = EuropeanOption::new(
            dec!(100),
            dec!(110),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );
        assert_eq!(call_itm.intrinsic_value().amount(), dec!(10));

        let call_otm = EuropeanOption::new(
            dec!(100),
            dec!(90),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );
        assert_eq!(call_otm.intrinsic_value().amount(), dec!(0));

        let put_itm = EuropeanOption::new(
            dec!(100),
            dec!(90),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Put,
        );
        assert_eq!(put_itm.intrinsic_value().amount(), dec!(10));
    }

    #[test]
    fn test_moneyness() {
        let option = EuropeanOption::new(
            dec!(100),
            dec!(110),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );
        assert_eq!(option.moneyness(), dec!(1.1));
    }

    #[test]
    fn test_option_with_spot() {
        let option = EuropeanOption::new(
            dec!(100),
            dec!(105),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );
        let new_option = option.with_spot(dec!(95));
        assert_eq!(new_option.spot(), dec!(95));
        assert!(new_option.is_out_of_the_money());
    }

    #[test]
    fn test_calculate_payoff() {
        let payoff = calculate_payoff(
            dec!(110),
            dec!(100),
            OptionType::Call,
            dec!(1),
            CurrencyCode::USD,
        );
        assert_eq!(payoff.amount.amount(), dec!(10));
        assert!(payoff.in_the_money);

        let payoff = calculate_payoff(
            dec!(90),
            dec!(100),
            OptionType::Call,
            dec!(1),
            CurrencyCode::USD,
        );
        assert_eq!(payoff.amount.amount(), dec!(0));
        assert!(!payoff.in_the_money);
    }

    #[test]
    fn test_american_option() {
        let option = AmericanOption::new(
            dec!(100),
            dec!(105),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );
        assert_eq!(option.intrinsic_value().amount(), dec!(5));
    }
}

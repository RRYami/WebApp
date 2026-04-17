//! Core traits for pricing and risk calculations.

use crate::core::error::Result;
use crate::core::money::Money;
use crate::risk::greeks::Greeks;

/// Trait for types that can be priced.
///
/// Implement this trait for any financial instrument that has a theoretical price.
pub trait Pricable {
    /// Calculate the price of the instrument.
    ///
    /// # Returns
    ///
    /// The price as a Money value, or an error if pricing fails.
    fn price(&self) -> Result<Money>;

    /// Calculate the price with a specific pricing engine.
    ///
    /// # Type Parameters
    ///
    /// * `E` - The pricing engine type.
    ///
    /// # Returns
    ///
    /// The price as a Money value, or an error if pricing fails.
    fn price_with<E: PricingEngine>(&self, engine: &E) -> Result<Money>;
}

/// Trait for types that have Greeks (risk sensitivities).
///
/// This is typically implemented by options and other derivatives.
pub trait HasGreeks {
    /// Calculate all Greeks at once.
    ///
    /// # Returns
    ///
    /// A Greeks struct containing all sensitivities.
    fn greeks(&self) -> Result<Greeks>;

    /// Calculate delta (sensitivity to underlying price).
    fn delta(&self) -> Result<f64>;

    /// Calculate gamma (sensitivity of delta to underlying price).
    fn gamma(&self) -> Result<f64>;

    /// Calculate theta (sensitivity to time).
    fn theta(&self) -> Result<f64>;

    /// Calculate vega (sensitivity to volatility).
    fn vega(&self) -> Result<f64>;

    /// Calculate rho (sensitivity to interest rate).
    fn rho(&self) -> Result<f64>;
}

/// Trait for pricing engines.
///
/// Different pricing models (Black-Scholes, Binomial, Monte Carlo) implement this trait.
pub trait PricingEngine {
    /// Price a given instrument.
    ///
    /// # Type Parameters
    ///
    /// * `I` - The instrument type to price.
    fn price<I: Instrument + 'static>(&self, instrument: &I) -> Result<Money>;
}

/// Trait for financial instruments.
///
/// This is the base trait that all financial instruments must implement.
pub trait Instrument {
    /// Get the instrument's notional amount.
    fn notional(&self) -> Money;

    /// Get the instrument's currency.
    fn currency(&self) -> crate::core::currency::CurrencyCode {
        self.notional().currency()
    }

    /// Get the instrument's maturity date if applicable.
    fn maturity(&self) -> Option<chrono::NaiveDate>;

    /// Get the instrument type name.
    fn instrument_type(&self) -> &'static str;
}

/// Trait for instruments that pay coupons or cash flows.
pub trait CashFlowGenerating: Instrument {
    /// Get all future cash flows.
    ///
    /// # Returns
    ///
    /// A vector of (date, amount) pairs representing cash flows.
    fn cash_flows(&self) -> Vec<(chrono::NaiveDate, Money)>;

    /// Get the next cash flow date after the given date.
    fn next_cash_flow_date(&self, after: chrono::NaiveDate) -> Option<chrono::NaiveDate>;
}

/// Trait for instruments that have a yield or internal rate of return.
pub trait HasYield: Instrument {
    /// Calculate the yield to maturity.
    ///
    /// # Arguments
    ///
    /// * `market_price` - The current market price of the instrument.
    /// * `guess` - Initial guess for the yield (optional).
    fn yield_to_maturity(&self, market_price: Money, guess: Option<f64>) -> Result<f64>;

    /// Calculate the current yield (annual coupon / price).
    fn current_yield(&self, market_price: Money) -> Result<f64>;
}

/// Trait for instruments with embedded options.
pub trait Optionable: Instrument {
    /// Check if the option is exercisable at the given date.
    fn is_exercisable(&self, date: chrono::NaiveDate) -> bool;

    /// Calculate the intrinsic value.
    fn intrinsic_value(&self, underlying_price: Money) -> Money;

    /// Check if the option is in-the-money.
    fn is_in_the_money(&self, underlying_price: Money) -> bool;
}

//! Black-Scholes option pricing model.

use crate::core::error::{Error, Result};
use crate::core::money::Money;
use crate::core::traits::{HasGreeks, Instrument, Pricable, PricingEngine};
use crate::instruments::option::{EuropeanOption, OptionType};
use crate::risk::greeks::Greeks;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use std::f64::consts::PI;

/// Standard normal cumulative distribution function.
///
/// Uses the Hart approximation for accuracy.
pub fn ndf(x: f64) -> f64 {
    if x < -10.0 {
        return 0.0;
    }
    if x > 10.0 {
        return 1.0;
    }

    // Hart approximation
    let b1 = 0.319381530;
    let b2 = -0.356563782;
    let b3 = 1.781477937;
    let b4 = -1.821255978;
    let b5 = 1.330274429;
    let p = 0.2316419;
    let c = 0.39894228;

    let ax = x.abs();
    let t = 1.0 / (1.0 + p * ax);

    let phi = c * (-ax * ax / 2.0).exp();
    let poly = b1 * t + b2 * t * t + b3 * t * t * t + b4 * t * t * t * t + b5 * t * t * t * t * t;

    let result = 1.0 - phi * poly;

    if x >= 0.0 {
        result
    } else {
        1.0 - result
    }
}

/// Standard normal probability density function.
pub fn npdf(x: f64) -> f64 {
    (-x * x / 2.0).exp() / (2.0 * PI).sqrt()
}

/// Black-Scholes pricing model.
#[derive(Debug, Clone)]
pub struct BlackScholes;

impl BlackScholes {
    /// Create a new Black-Scholes model.
    pub fn new() -> Self {
        Self
    }

    /// Calculate d1 for Black-Scholes formula.
    ///
    /// d1 = (ln(S/K) + (r + σ²/2) * T) / (σ * √T)
    pub fn d1(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        time: f64,
    ) -> Result<f64> {
        if volatility.is_zero() {
            return Err(Error::invalid_input("Volatility cannot be zero"));
        }
        if time <= 0.0 {
            return Err(Error::invalid_input("Time to expiry must be positive"));
        }
        if spot <= Decimal::ZERO || strike <= Decimal::ZERO {
            return Err(Error::invalid_input("Spot and strike must be positive"));
        }

        let spot_f = spot
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid spot"))?;
        let strike_f = strike
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid strike"))?;
        let rate_f = rate
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid rate"))?;
        let vol_f = volatility
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid volatility"))?;

        let ln_sk = (spot_f / strike_f).ln();
        let numerator = ln_sk + (rate_f + vol_f * vol_f / 2.0) * time;
        let denominator = vol_f * time.sqrt();

        Ok(numerator / denominator)
    }

    /// Calculate d2 for Black-Scholes formula.
    ///
    /// d2 = d1 - σ * √T
    pub fn d2(d1: f64, volatility: Decimal, time: f64) -> Result<f64> {
        let vol_f = volatility
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid volatility"))?;
        Ok(d1 - vol_f * time.sqrt())
    }

    /// Price a European call option.
    ///
    /// Call = S * N(d1) - K * e^(-rT) * N(d2)
    pub fn price_call(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        time: f64,
    ) -> Result<Decimal> {
        let d1_val = Self::d1(spot, strike, rate, volatility, time)?;
        let d2_val = Self::d2(d1_val, volatility, time)?;

        let nd1 = ndf(d1_val);
        let nd2 = ndf(d2_val);

        let spot_f = spot
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid spot"))?;
        let strike_f = strike
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid strike"))?;
        let rate_f = rate
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid rate"))?;

        let price = spot_f * nd1 - strike_f * (-rate_f * time).exp() * nd2;

        Decimal::try_from(price)
            .map_err(|_| Error::arithmetic("Failed to convert price to Decimal"))
    }

    /// Price a European put option.
    ///
    /// Put = K * e^(-rT) * N(-d2) - S * N(-d1)
    pub fn price_put(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        time: f64,
    ) -> Result<Decimal> {
        let d1_val = Self::d1(spot, strike, rate, volatility, time)?;
        let d2_val = Self::d2(d1_val, volatility, time)?;

        let n_neg_d1 = ndf(-d1_val);
        let n_neg_d2 = ndf(-d2_val);

        let spot_f = spot
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid spot"))?;
        let strike_f = strike
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid strike"))?;
        let rate_f = rate
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid rate"))?;

        let price = strike_f * (-rate_f * time).exp() * n_neg_d2 - spot_f * n_neg_d1;

        Decimal::try_from(price)
            .map_err(|_| Error::arithmetic("Failed to convert price to Decimal"))
    }

    /// Price a European option.
    pub fn price(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Decimal> {
        match option_type {
            OptionType::Call => Self::price_call(spot, strike, rate, volatility, time),
            OptionType::Put => Self::price_put(spot, strike, rate, volatility, time),
        }
    }

    /// Calculate Greeks for a European option.
    pub fn greeks(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Greeks> {
        let d1_val = Self::d1(spot, strike, rate, volatility, time)?;
        let d2_val = Self::d2(d1_val, volatility, time)?;

        let nd1 = ndf(d1_val);
        let nd2 = ndf(d2_val);
        let n_prime_d1 = npdf(d1_val);

        let spot_f = spot
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid spot"))?;
        let strike_f = strike
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid strike"))?;
        let rate_f = rate
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid rate"))?;
        let vol_f = volatility
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid volatility"))?;

        // Delta
        let delta = match option_type {
            OptionType::Call => nd1,
            OptionType::Put => nd1 - 1.0,
        };

        // Gamma (same for calls and puts)
        let gamma = n_prime_d1 / (spot_f * vol_f * time.sqrt());

        // Theta
        let term1 = -spot_f * n_prime_d1 * vol_f / (2.0 * time.sqrt());
        let term2 = match option_type {
            OptionType::Call => -rate_f * strike_f * (-rate_f * time).exp() * nd2,
            OptionType::Put => rate_f * strike_f * (-rate_f * time).exp() * ndf(-d2_val),
        };
        let theta = (term1 + term2) / 365.0; // Daily theta

        // Vega (same for calls and puts)
        let vega = spot_f * n_prime_d1 * time.sqrt() / 100.0; // For 1% change

        // Rho
        let rho = match option_type {
            OptionType::Call => strike_f * time * (-rate_f * time).exp() * nd2 / 100.0,
            OptionType::Put => -strike_f * time * (-rate_f * time).exp() * ndf(-d2_val) / 100.0,
        };

        Ok(Greeks {
            delta,
            gamma,
            theta,
            vega,
            rho,
            phi: -time
                * Self::price_call(spot, strike, rate, volatility, time)?
                    .to_f64()
                    .unwrap_or(0.0)
                / 365.0,
        })
    }

    /// Calculate implied volatility from market price.
    ///
    /// Uses Newton-Raphson method.
    pub fn implied_volatility(
        market_price: Decimal,
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        time: f64,
        option_type: OptionType,
        guess: Option<f64>,
    ) -> Result<f64> {
        let mut vol = guess.unwrap_or(0.2);
        let target_price = market_price
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid market price"))?;
        let tolerance = 1e-10;
        let max_iterations = 100;

        for _i in 0..max_iterations {
            let price = Self::price(
                spot,
                strike,
                rate,
                Decimal::from_f64(vol).unwrap(),
                time,
                option_type,
            )?;
            let price_f = price
                .to_f64()
                .ok_or_else(|| Error::arithmetic("Invalid price"))?;

            let diff = price_f - target_price;
            if diff.abs() < tolerance {
                return Ok(vol);
            }

            // Calculate vega for derivative
            let vega = Self::calculate_vega(spot, strike, rate, vol, time)?;

            if vega.abs() < 1e-10 {
                return Err(Error::pricing("Vega too small, cannot converge"));
            }

            vol -= diff / vega;

            if vol < 0.0 || vol > 5.0 {
                return Err(Error::pricing("Implied volatility calculation diverged"));
            }
        }

        Err(Error::pricing(format!(
            "Implied volatility did not converge after {} iterations",
            max_iterations
        )))
    }

    /// Calculate vega (used in implied vol calculation).
    fn calculate_vega(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: f64,
        time: f64,
    ) -> Result<f64> {
        let vol_dec =
            Decimal::from_f64(volatility).ok_or_else(|| Error::arithmetic("Invalid vol"))?;
        let d1_val = Self::d1(spot, strike, rate, vol_dec, time)?;
        let n_prime_d1 = npdf(d1_val);

        let spot_f = spot
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid spot"))?;

        Ok(spot_f * n_prime_d1 * time.sqrt())
    }
}

impl Default for BlackScholes {
    fn default() -> Self {
        Self::new()
    }
}

impl PricingEngine for BlackScholes {
    fn price<I: Instrument + 'static>(&self, instrument: &I) -> Result<Money> {
        // Try to downcast to EuropeanOption
        if let Some(option) = (instrument as &dyn std::any::Any).downcast_ref::<EuropeanOption>() {
            let price = Self::price(
                option.spot(),
                option.strike(),
                option.risk_free_rate(),
                option.volatility(),
                option.time_to_expiry(),
                option.option_type(),
            )?;
            Ok(Money::new(price, option.underlying_currency()))
        } else {
            Err(Error::pricing(format!(
                "BlackScholes engine does not support instrument type: {}",
                instrument.instrument_type()
            )))
        }
    }
    // Note: supports() method not overridden - using trait default
}

impl Pricable for EuropeanOption {
    fn price(&self) -> Result<Money> {
        let price = BlackScholes::price(
            self.spot(),
            self.strike(),
            self.risk_free_rate(),
            self.volatility(),
            self.time_to_expiry(),
            self.option_type(),
        )?;
        Ok(Money::new(price, self.underlying_currency()))
    }

    fn price_with<E: PricingEngine>(&self, engine: &E) -> Result<Money> {
        engine.price(self)
    }
}

impl HasGreeks for EuropeanOption {
    fn greeks(&self) -> Result<Greeks> {
        BlackScholes::greeks(
            self.spot(),
            self.strike(),
            self.risk_free_rate(),
            self.volatility(),
            self.time_to_expiry(),
            self.option_type(),
        )
    }

    fn delta(&self) -> Result<f64> {
        let d1_val = BlackScholes::d1(
            self.spot(),
            self.strike(),
            self.risk_free_rate(),
            self.volatility(),
            self.time_to_expiry(),
        )?;

        let nd1 = ndf(d1_val);
        Ok(match self.option_type() {
            OptionType::Call => nd1,
            OptionType::Put => nd1 - 1.0,
        })
    }

    fn gamma(&self) -> Result<f64> {
        let greeks = self.greeks()?;
        Ok(greeks.gamma)
    }

    fn theta(&self) -> Result<f64> {
        let greeks = self.greeks()?;
        Ok(greeks.theta)
    }

    fn vega(&self) -> Result<f64> {
        let greeks = self.greeks()?;
        Ok(greeks.vega)
    }

    fn rho(&self) -> Result<f64> {
        let greeks = self.greeks()?;
        Ok(greeks.rho)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_ndf() {
        // Standard normal CDF
        assert!((ndf(0.0) - 0.5).abs() < 1e-6); // Relaxed tolerance for Hart approximation
        assert!((ndf(1.96) - 0.975).abs() < 0.01);
        assert!((ndf(-1.96) - 0.025).abs() < 0.01);
    }

    #[test]
    fn test_npdf() {
        // Standard normal PDF
        assert!((npdf(0.0) - 0.39894228).abs() < 1e-6);
    }

    #[test]
    fn test_call_price() {
        // Standard Black-Scholes test case
        let price = BlackScholes::price_call(
            dec!(100),  // spot
            dec!(100),  // strike
            dec!(0.05), // rate
            dec!(0.2),  // volatility
            1.0,        // time
        )
        .unwrap();

        // Expected price is approximately 10.45
        let price_f = price.to_f64().unwrap();
        assert!((price_f - 10.45).abs() < 0.1);
    }

    #[test]
    fn test_put_price() {
        let price = BlackScholes::price_put(
            dec!(100),  // spot
            dec!(100),  // strike
            dec!(0.05), // rate
            dec!(0.2),  // volatility
            1.0,        // time
        )
        .unwrap();

        // Expected price is approximately 5.57
        let price_f = price.to_f64().unwrap();
        assert!((price_f - 5.57).abs() < 0.1);
    }

    #[test]
    fn test_put_call_parity() {
        let spot = dec!(100);
        let strike = dec!(100);
        let rate = dec!(0.05);
        let vol = dec!(0.2);
        let time = 1.0;

        let call = BlackScholes::price_call(spot, strike, rate, vol, time).unwrap();
        let put = BlackScholes::price_put(spot, strike, rate, vol, time).unwrap();

        // Put-Call Parity: C - P = S - K * e^(-rT)
        let lhs = call - put;
        let rhs = spot - strike * Decimal::from_f64((-0.05_f64).exp()).unwrap();

        assert!((lhs - rhs).abs() < dec!(0.01));
    }

    #[test]
    fn test_greeks() {
        let greeks = BlackScholes::greeks(
            dec!(100),  // spot
            dec!(100),  // strike
            dec!(0.05), // rate
            dec!(0.2),  // volatility
            1.0,        // time
            OptionType::Call,
        )
        .unwrap();

        // ATM call delta should be around 0.6 (not exactly 0.5 due to drift from risk-free rate)
        assert!(
            greeks.delta > 0.5 && greeks.delta < 0.7,
            "Delta was {}",
            greeks.delta
        );

        // Gamma should be positive
        assert!(greeks.gamma > 0.0);

        // Vega should be positive
        assert!(greeks.vega > 0.0);
    }

    #[test]
    fn test_implied_volatility() {
        // Calculate a price with known vol
        let target_vol = 0.25;
        let price = BlackScholes::price_call(
            dec!(100),
            dec!(100),
            dec!(0.05),
            Decimal::from_f64(target_vol).unwrap(),
            1.0,
        )
        .unwrap();

        // Recover the volatility
        let implied = BlackScholes::implied_volatility(
            price,
            dec!(100),
            dec!(100),
            dec!(0.05),
            1.0,
            OptionType::Call,
            None,
        )
        .unwrap();

        assert!((implied - target_vol).abs() < 1e-6);
    }

    #[test]
    fn test_european_option_pricable() {
        let option = EuropeanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );

        let price = option.price().unwrap();
        assert!(price.amount() > dec!(0));
    }

    #[test]
    fn test_european_option_has_greeks() {
        let option = EuropeanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );

        let greeks = option.greeks().unwrap();
        // ATM call delta should be around 0.6 (not exactly 0.5 due to drift from risk-free rate)
        assert!(
            greeks.delta > 0.5 && greeks.delta < 0.7,
            "Delta was {}",
            greeks.delta
        );
    }

    #[test]
    fn test_zero_volatility_error() {
        let result = BlackScholes::price_call(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0), // zero volatility
            1.0,
        );
        assert!(result.is_err());
    }
}

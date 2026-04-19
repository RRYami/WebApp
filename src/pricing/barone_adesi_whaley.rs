//! Barone-Adesi Whaley (BAW) American option pricing model.
//!
//! BAW is a quadratic approximation method for pricing American options.
//! It's much faster than binomial trees while maintaining good accuracy.
//!
//! The model decomposes the American option price into:
//! - European component (Black-Scholes price)
//! - Early exercise premium
//!
//! # Formulas
//!
//! For American calls:
//! C(S) = C_bs(S) + A2 * (S/S*)^q2    when S < S*
//! C(S) = S - K                       when S >= S*
//!
//! For American puts:
//! P(S) = P_bs(S) + A1 * (S/S*)^q1    when S > S*
//! P(S) = K - S                       when S <= S*
//!
//! Where S* is the critical stock price found by solving a nonlinear equation.

use crate::core::error::{Error, Result};
use crate::core::money::Money;
use crate::core::traits::{HasGreeks, Instrument, Pricable, PricingEngine};
use crate::instruments::option::{AmericanOption, OptionType};
use crate::pricing::black_scholes::{ndf, npdf, BlackScholes};
use crate::risk::greeks::Greeks;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

/// Barone-Adesi Whaley American option pricing model.
#[derive(Debug, Clone)]
pub struct BaroneAdesiWhaley;

impl BaroneAdesiWhaley {
    /// Create a new BAW model.
    pub fn new() -> Self {
        Self
    }

    /// Price an American option using BAW approximation.
    ///
    /// # Arguments
    ///
    /// * `spot` - Current spot price
    /// * `strike` - Strike price
    /// * `rate` - Risk-free rate (continuous)
    /// * `volatility` - Volatility
    /// * `dividend_yield` - Dividend yield (q)
    /// * `time` - Time to expiry in years
    /// * `option_type` - Call or Put
    pub fn price(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        dividend_yield: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Decimal> {
        // Edge cases
        if time <= 0.0 {
            // Expired - return intrinsic value
            return Ok(match option_type {
                OptionType::Call => (spot - strike).max(Decimal::ZERO),
                OptionType::Put => (strike - spot).max(Decimal::ZERO),
            });
        }

        if volatility.is_zero() {
            // Zero volatility - return discounted intrinsic
            let b = rate - dividend_yield;
            let growth = (b * Decimal::from_f64(time).unwrap()).exp();
            let forward = spot * growth;
            let df = (-rate * Decimal::from_f64(time).unwrap()).exp();
            return Ok(match option_type {
                OptionType::Call => ((forward - strike) * df).max(Decimal::ZERO),
                OptionType::Put => ((strike - forward) * df).max(Decimal::ZERO),
            });
        }

        // Check for immediate exercise (deep ITM)
        match option_type {
            OptionType::Call => {
                // For calls, if spot is very high relative to strike, exercise immediately
                // S* is the critical price - if S > S*, exercise
                if spot > strike * Decimal::from_f64(10.0).unwrap() {
                    return Ok(spot - strike);
                }
            }
            OptionType::Put => {
                // For puts, if spot is very low relative to strike, exercise immediately
                if spot < strike / Decimal::from_f64(10.0).unwrap() {
                    return Ok(strike - spot);
                }
            }
        }

        match option_type {
            OptionType::Call => {
                Self::price_call(spot, strike, rate, volatility, dividend_yield, time)
            }
            OptionType::Put => {
                Self::price_put(spot, strike, rate, volatility, dividend_yield, time)
            }
        }
    }

    /// Price an American call option using BAW.
    fn price_call(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        dividend_yield: Decimal,
        time: f64,
    ) -> Result<Decimal> {
        let b = rate - dividend_yield; // Cost of carry
        let b_f = b.to_f64().unwrap();

        // If b >= r (i.e., q <= 0), no early exercise premium for calls
        // Unless there are negative dividends (unlikely), calls behave like European
        if b_f >= rate.to_f64().unwrap() {
            return BlackScholes::price_call(spot, strike, rate, volatility, time);
        }

        let spot_f = spot.to_f64().unwrap();
        let strike_f = strike.to_f64().unwrap();
        let rate_f = rate.to_f64().unwrap();
        let vol_f = volatility.to_f64().unwrap();
        let time_f = time;

        // Calculate coefficients
        let n = 2.0 * b_f / (vol_f * vol_f);
        let m = 2.0 * rate_f / (vol_f * vol_f);
        let k = 1.0 - (-rate_f * time_f).exp();

        let q2 = (-(n - 1.0) + ((n - 1.0) * (n - 1.0) + 4.0 * m / k).sqrt()) / 2.0;

        // Find critical price S*
        let s_star =
            Self::find_critical_price_call(spot_f, strike_f, rate_f, b_f, vol_f, time_f, q2, n, k)?;

        // If spot >= S*, exercise immediately
        if spot_f >= s_star {
            return Ok(spot - strike);
        }

        // Calculate European price
        let european_price = BlackScholes::price_call(spot, strike, rate, volatility, time)?;

        // Calculate early exercise premium
        let d1 = Self::calculate_d1(s_star, strike_f, b_f, vol_f, time_f);
        let nd1 = ndf(d1);

        let a2 = s_star / q2 * (1.0 - ((b_f - rate_f) * time_f).exp() * nd1);

        let premium = a2 * (spot_f / s_star).powf(q2);

        let total_price = european_price.to_f64().unwrap() + premium;
        Decimal::try_from(total_price).map_err(|_| Error::arithmetic("Failed to convert BAW price"))
    }

    /// Price an American put option using BAW.
    fn price_put(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        dividend_yield: Decimal,
        time: f64,
    ) -> Result<Decimal> {
        let b = rate - dividend_yield;
        let b_f = b.to_f64().unwrap();
        let spot_f = spot.to_f64().unwrap();
        let strike_f = strike.to_f64().unwrap();
        let rate_f = rate.to_f64().unwrap();
        let vol_f = volatility.to_f64().unwrap();
        let time_f = time;

        // Calculate coefficients
        let n = 2.0 * b_f / (vol_f * vol_f);
        let m = 2.0 * rate_f / (vol_f * vol_f);
        let k = 1.0 - (-rate_f * time_f).exp();

        let q1 = (-(n - 1.0) - ((n - 1.0) * (n - 1.0) + 4.0 * m / k).sqrt()) / 2.0;

        // Find critical price S*
        let s_star =
            Self::find_critical_price_put(spot_f, strike_f, rate_f, b_f, vol_f, time_f, q1, n, k)?;

        // If spot <= S*, exercise immediately
        if spot_f <= s_star {
            return Ok(strike - spot);
        }

        // Calculate European price
        let european_price = BlackScholes::price_put(spot, strike, rate, volatility, time)?;

        // Calculate early exercise premium
        let d1 = Self::calculate_d1(s_star, strike_f, b_f, vol_f, time_f);
        let nd1 = ndf(d1);

        let a1 = -s_star / q1 * (1.0 - ((b_f - rate_f) * time_f).exp() * nd1);

        let premium = a1 * (spot_f / s_star).powf(q1);

        let total_price = european_price.to_f64().unwrap() + premium;
        Decimal::try_from(total_price).map_err(|_| Error::arithmetic("Failed to convert BAW price"))
    }

    /// Find critical price S* for American calls using Newton-Raphson.
    fn find_critical_price_call(
        _spot: f64,
        strike: f64,
        rate: f64,
        b: f64,
        vol: f64,
        time: f64,
        q2: f64,
        _n: f64,
        _k: f64,
    ) -> Result<f64> {
        // Initial guess
        let mut s_star = strike;
        let tolerance = 1e-6;
        let max_iterations = 100;

        for _ in 0..max_iterations {
            // Calculate European call at S*
            let european_at_s = Self::bs_call_at_point(s_star, strike, rate, b, vol, time)?;

            // Calculate d1 at S*
            let d1 = Self::calculate_d1(s_star, strike, b, vol, time);
            let nd1 = ndf(d1);

            // Function to solve: f(S*) = S* - K - C_bs(S*) - [1 - e^((b-r)T)N(d1)] * S* / q2 = 0
            let f = s_star
                - strike
                - european_at_s
                - (1.0 - ((b - rate) * time).exp() * nd1) * s_star / q2;

            // Derivative (simplified)
            let nd1_derivative = npdf(d1);
            let df = 1.0
                - (1.0 - ((b - rate) * time).exp() * nd1) / q2
                - ((b - rate) * time).exp() * nd1_derivative / (q2 * vol * time.sqrt());

            if df.abs() < 1e-10 {
                return Ok(s_star);
            }

            let new_s_star = s_star - f / df;

            if (new_s_star - s_star).abs() < tolerance {
                return Ok(new_s_star);
            }

            s_star = new_s_star.max(strike); // S* must be >= strike for calls
        }

        Ok(s_star)
    }

    /// Find critical price S* for American puts using Newton-Raphson.
    fn find_critical_price_put(
        _spot: f64,
        strike: f64,
        rate: f64,
        b: f64,
        vol: f64,
        time: f64,
        q1: f64,
        _n: f64,
        _k: f64,
    ) -> Result<f64> {
        // Initial guess
        let mut s_star = strike;
        let tolerance = 1e-6;
        let max_iterations = 100;

        for _ in 0..max_iterations {
            // Calculate European put at S*
            let european_at_s = Self::bs_put_at_point(s_star, strike, rate, b, vol, time)?;

            // Calculate d1 at S*
            let d1 = Self::calculate_d1(s_star, strike, b, vol, time);
            let nd1 = ndf(d1);

            // Function to solve: f(S*) = K - S* - P_bs(S*) + [1 - e^((b-r)T)N(d1)] * S* / q1 = 0
            let f = strike - s_star - european_at_s
                + (1.0 - ((b - rate) * time).exp() * nd1) * s_star / q1;

            // Derivative
            let nd1_derivative = npdf(d1);
            let df = -1.0
                + (1.0 - ((b - rate) * time).exp() * nd1) / q1
                + ((b - rate) * time).exp() * nd1_derivative / (q1 * vol * time.sqrt());

            if df.abs() < 1e-10 {
                return Ok(s_star);
            }

            let new_s_star = s_star - f / df;

            if (new_s_star - s_star).abs() < tolerance {
                return Ok(new_s_star);
            }

            s_star = new_s_star.min(strike); // S* must be <= strike for puts
        }

        Ok(s_star)
    }

    /// Calculate d1 for Black-Scholes with cost of carry.
    fn calculate_d1(spot: f64, strike: f64, b: f64, vol: f64, time: f64) -> f64 {
        let ln_sk = (spot / strike).ln();
        let numerator = ln_sk + (b + vol * vol / 2.0) * time;
        let denominator = vol * time.sqrt();
        numerator / denominator
    }

    /// Calculate European call price at a specific point (for internal use).
    fn bs_call_at_point(
        spot: f64,
        strike: f64,
        rate: f64,
        _b: f64,
        vol: f64,
        time: f64,
    ) -> Result<f64> {
        let spot_dec = Decimal::from_f64(spot).unwrap();
        let strike_dec = Decimal::from_f64(strike).unwrap();
        let rate_dec = Decimal::from_f64(rate).unwrap();
        let vol_dec = Decimal::from_f64(vol).unwrap();

        let price = BlackScholes::price_call(spot_dec, strike_dec, rate_dec, vol_dec, time)?;

        price
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Failed to convert price"))
    }

    /// Calculate European put price at a specific point (for internal use).
    fn bs_put_at_point(
        spot: f64,
        strike: f64,
        rate: f64,
        _b: f64,
        vol: f64,
        time: f64,
    ) -> Result<f64> {
        let spot_dec = Decimal::from_f64(spot).unwrap();
        let strike_dec = Decimal::from_f64(strike).unwrap();
        let rate_dec = Decimal::from_f64(rate).unwrap();
        let vol_dec = Decimal::from_f64(vol).unwrap();

        let price = BlackScholes::price_put(spot_dec, strike_dec, rate_dec, vol_dec, time)?;

        price
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Failed to convert price"))
    }

    /// Calculate Greeks for American options using numerical differentiation.
    ///
    /// This is the fastest method - uses central differences with small perturbations.
    pub fn greeks(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        dividend_yield: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Greeks> {
        let epsilon_spot = spot * Decimal::from_f64(0.0001).unwrap(); // 0.01% bump
        let epsilon_vol = Decimal::from_f64(0.0001).unwrap(); // 0.01% bump
        let epsilon_rate = Decimal::from_f64(0.0001).unwrap(); // 0.01% bump
        let epsilon_time = 0.0001; // Small time bump

        let epsilon_spot_f = epsilon_spot.to_f64().unwrap();
        let price_f = Self::price(
            spot,
            strike,
            rate,
            volatility,
            dividend_yield,
            time,
            option_type,
        )?
        .to_f64()
        .unwrap();

        // Delta (sensitivity to spot)
        let price_up_spot = Self::price(
            spot + epsilon_spot,
            strike,
            rate,
            volatility,
            dividend_yield,
            time,
            option_type,
        )?
        .to_f64()
        .unwrap();
        let price_down_spot = Self::price(
            spot - epsilon_spot,
            strike,
            rate,
            volatility,
            dividend_yield,
            time,
            option_type,
        )?
        .to_f64()
        .unwrap();
        let delta = (price_up_spot - price_down_spot) / (2.0 * epsilon_spot_f);

        // Gamma (second derivative)
        let gamma =
            (price_up_spot - 2.0 * price_f + price_down_spot) / (epsilon_spot_f * epsilon_spot_f);

        // Theta (sensitivity to time)
        let price_up_time = Self::price(
            spot,
            strike,
            rate,
            volatility,
            dividend_yield,
            time + epsilon_time,
            option_type,
        )?
        .to_f64()
        .unwrap();
        let price_down_time = Self::price(
            spot,
            strike,
            rate,
            volatility,
            dividend_yield,
            time - epsilon_time,
            option_type,
        )?
        .to_f64()
        .unwrap();
        let theta = (price_up_time - price_down_time) / (2.0 * epsilon_time) / 365.0; // Daily theta

        // Vega (sensitivity to volatility)
        let price_up_vol = Self::price(
            spot,
            strike,
            rate,
            volatility + epsilon_vol,
            dividend_yield,
            time,
            option_type,
        )?
        .to_f64()
        .unwrap();
        let price_down_vol = Self::price(
            spot,
            strike,
            rate,
            volatility - epsilon_vol,
            dividend_yield,
            time,
            option_type,
        )?
        .to_f64()
        .unwrap();
        let vega = (price_up_vol - price_down_vol) / (2.0 * epsilon_vol.to_f64().unwrap()) / 100.0; // Per 1%

        // Rho (sensitivity to rate)
        let price_up_rate = Self::price(
            spot,
            strike,
            rate + epsilon_rate,
            volatility,
            dividend_yield,
            time,
            option_type,
        )?
        .to_f64()
        .unwrap();
        let price_down_rate = Self::price(
            spot,
            strike,
            rate - epsilon_rate,
            volatility,
            dividend_yield,
            time,
            option_type,
        )?
        .to_f64()
        .unwrap();
        let rho =
            (price_up_rate - price_down_rate) / (2.0 * epsilon_rate.to_f64().unwrap()) / 100.0; // Per 1%

        Ok(Greeks {
            delta,
            gamma,
            theta,
            vega,
            rho,
            phi: 0.0,
        })
    }

    /// Calculate early exercise premium (American - European).
    pub fn early_exercise_premium(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        dividend_yield: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Decimal> {
        let american = Self::price(
            spot,
            strike,
            rate,
            volatility,
            dividend_yield,
            time,
            option_type,
        )?;
        let european = match option_type {
            OptionType::Call => BlackScholes::price_call(spot, strike, rate, volatility, time),
            OptionType::Put => BlackScholes::price_put(spot, strike, rate, volatility, time),
        }?;
        Ok(american - european)
    }
}

impl Default for BaroneAdesiWhaley {
    fn default() -> Self {
        Self::new()
    }
}

impl PricingEngine for BaroneAdesiWhaley {
    fn price(&self, instrument: &dyn Instrument) -> Result<Money> {
        if let Some(option) = instrument.as_any().downcast_ref::<AmericanOption>() {
            let price = Self::price(
                option.spot(),
                option.strike(),
                option.risk_free_rate(),
                option.volatility(),
                option.dividend_yield(),
                option.time_to_expiry(),
                option.option_type(),
            )?;
            Ok(Money::new(price, option.underlying_currency()))
        } else {
            Err(Error::pricing(format!(
                "BAW engine does not support instrument type: {}",
                instrument.instrument_type()
            )))
        }
    }

    fn supports(&self, instrument: &dyn Instrument) -> bool {
        instrument.as_any().is::<AmericanOption>()
    }

    fn name(&self) -> &'static str {
        "BaroneAdesiWhaley"
    }
}

impl Pricable for AmericanOption {
    fn price(&self) -> Result<Money> {
        let price = BaroneAdesiWhaley::price(
            self.spot(),
            self.strike(),
            self.risk_free_rate(),
            self.volatility(),
            self.dividend_yield(),
            self.time_to_expiry(),
            self.option_type(),
        )?;
        Ok(Money::new(price, self.underlying_currency()))
    }

    fn price_with_dyn(&self, engine: &dyn PricingEngine) -> Result<Money> {
        engine.price(self)
    }
}

impl HasGreeks for AmericanOption {
    fn greeks(&self) -> Result<Greeks> {
        BaroneAdesiWhaley::greeks(
            self.spot(),
            self.strike(),
            self.risk_free_rate(),
            self.volatility(),
            self.dividend_yield(),
            self.time_to_expiry(),
            self.option_type(),
        )
    }

    fn delta(&self) -> Result<f64> {
        self.greeks().map(|g| g.delta)
    }

    fn gamma(&self) -> Result<f64> {
        self.greeks().map(|g| g.gamma)
    }

    fn theta(&self) -> Result<f64> {
        self.greeks().map(|g| g.theta)
    }

    fn vega(&self) -> Result<f64> {
        self.greeks().map(|g| g.vega)
    }

    fn rho(&self) -> Result<f64> {
        self.greeks().map(|g| g.rho)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_baw_call_no_dividend() {
        // Without dividends, American call = European call
        let spot = dec!(100);
        let strike = dec!(100);
        let rate = dec!(0.05);
        let vol = dec!(0.2);
        let dividend = Decimal::ZERO;
        let time = 1.0;

        let baw_price =
            BaroneAdesiWhaley::price(spot, strike, rate, vol, dividend, time, OptionType::Call)
                .unwrap();

        let bs_price = BlackScholes::price_call(spot, strike, rate, vol, time).unwrap();

        // Should be very close when no dividends
        let diff = (baw_price - bs_price).abs() / bs_price;
        assert!(diff < dec!(0.001), "BAW should equal BS when no dividends");
    }

    #[test]
    fn test_baw_put_early_exercise() {
        // Deep ITM put should have early exercise premium
        let spot = dec!(50);
        let strike = dec!(100);
        let rate = dec!(0.05);
        let vol = dec!(0.2);
        let dividend = Decimal::ZERO;
        let time = 1.0;

        let baw_price =
            BaroneAdesiWhaley::price(spot, strike, rate, vol, dividend, time, OptionType::Put)
                .unwrap();

        let bs_price = BlackScholes::price_put(spot, strike, rate, vol, time).unwrap();

        // American put should be worth more than European
        assert!(
            baw_price > bs_price,
            "American put should have early exercise premium"
        );
    }

    #[test]
    fn test_baw_dividend_effect_on_calls() {
        // Higher dividend = lower call price (more likely to exercise early)
        let spot = dec!(100);
        let strike = dec!(100);
        let rate = dec!(0.05);
        let vol = dec!(0.2);
        let time = 1.0;

        let no_div = BaroneAdesiWhaley::price(
            spot,
            strike,
            rate,
            vol,
            Decimal::ZERO,
            time,
            OptionType::Call,
        )
        .unwrap();

        let with_div =
            BaroneAdesiWhaley::price(spot, strike, rate, vol, dec!(0.03), time, OptionType::Call)
                .unwrap();

        assert!(with_div != no_div, "Dividends should affect call price");
    }

    #[test]
    fn test_baw_zero_volatility() {
        // Zero vol should return discounted intrinsic
        let spot = dec!(110);
        let strike = dec!(100);
        let rate = dec!(0.05);
        let vol = Decimal::ZERO;
        let dividend = Decimal::ZERO;
        let time = 1.0;

        let price =
            BaroneAdesiWhaley::price(spot, strike, rate, vol, dividend, time, OptionType::Call)
                .unwrap();

        // Should be approximately spot - strike (discounted)
        assert!(price > dec!(0), "Should have positive value");
    }

    #[test]
    fn test_baw_expired() {
        // Expired option should return intrinsic value
        let spot = dec!(110);
        let strike = dec!(100);
        let rate = dec!(0.05);
        let vol = dec!(0.2);
        let dividend = Decimal::ZERO;
        let time = 0.0;

        let price =
            BaroneAdesiWhaley::price(spot, strike, rate, vol, dividend, time, OptionType::Call)
                .unwrap();

        assert_eq!(price, dec!(10), "Expired call should equal intrinsic");
    }

    #[test]
    fn test_baw_greeks() {
        let option = AmericanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );

        let greeks = option.greeks().unwrap();

        // Delta should be positive for calls
        assert!(greeks.delta > 0.0, "Call delta should be positive");

        // Gamma should be positive
        assert!(greeks.gamma > 0.0, "Gamma should be positive");

        // Vega should be positive
        assert!(greeks.vega > 0.0, "Vega should be positive");
    }

    #[test]
    fn test_early_exercise_premium() {
        let spot = dec!(80);
        let strike = dec!(100);
        let rate = dec!(0.05);
        let vol = dec!(0.2);
        let dividend = Decimal::ZERO;
        let time = 1.0;

        let premium = BaroneAdesiWhaley::early_exercise_premium(
            spot,
            strike,
            rate,
            vol,
            dividend,
            time,
            OptionType::Put,
        )
        .unwrap();

        assert!(
            premium > Decimal::ZERO,
            "Should have early exercise premium"
        );
    }
}

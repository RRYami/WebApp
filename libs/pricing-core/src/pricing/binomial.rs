//! Binomial tree option pricing model.
//!
//! Memory-efficient implementation using O(N) space with in-place backward induction.
//! Supports CRR, Jarrow-Rudd, and Tian models.

use crate::core::error::{Error, Result};
use crate::core::money::Money;
use crate::core::traits::{Instrument, PricingEngine};
use crate::instruments::option::{AmericanOption, EuropeanOption, OptionType};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

/// Binomial model type for tree parameter calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinomialModel {
    /// Cox-Ross-Rubinstein: u = e^(σ√Δt), d = 1/u
    CRR,
    /// Jarrow-Rudd: risk-neutral drift
    JR,
    /// Tian: matches first three moments
    Tian,
}

impl BinomialModel {
    /// Get model name.
    pub fn name(&self) -> &'static str {
        match self {
            BinomialModel::CRR => "CRR",
            BinomialModel::JR => "Jarrow-Rudd",
            BinomialModel::Tian => "Tian",
        }
    }
}

/// Binomial tree option pricing engine.
///
/// Efficient implementation using single Vec with in-place backward sweep.
/// Memory usage: O(N) where N = number of steps.
#[derive(Debug, Clone)]
pub struct BinomialTree {
    steps: usize,
    model: BinomialModel,
}

impl BinomialTree {
    /// Create new binomial tree pricer.
    pub fn new(steps: usize, model: BinomialModel) -> Self {
        Self { steps, model }
    }

    /// Create CRR tree (most common).
    pub fn crr(steps: usize) -> Self {
        Self::new(steps, BinomialModel::CRR)
    }

    /// Get number of steps.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// Get model type.
    pub fn model(&self) -> BinomialModel {
        self.model
    }

    /// Price American option with early exercise.
    pub fn price_american(
        &self,
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        dividend_yield: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Decimal> {
        self.price_internal(
            spot,
            strike,
            rate,
            volatility,
            dividend_yield,
            time,
            option_type,
            true,
        )
    }

    /// Price European option.
    pub fn price_european(
        &self,
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        dividend_yield: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Decimal> {
        self.price_internal(
            spot,
            strike,
            rate,
            volatility,
            dividend_yield,
            time,
            option_type,
            false,
        )
    }

    /// Core pricing implementation.
    fn price_internal(
        &self,
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        dividend_yield: Decimal,
        time: f64,
        option_type: OptionType,
        american: bool,
    ) -> Result<Decimal> {
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
            .ok_or_else(|| Error::arithmetic("Invalid vol"))?;
        let div_f = dividend_yield
            .to_f64()
            .ok_or_else(|| Error::arithmetic("Invalid div"))?;

        if vol_f <= 0.0 {
            return Err(Error::invalid_input("Volatility must be positive"));
        }
        if time <= 0.0 {
            let payoff = Self::payoff(spot_f, strike_f, option_type);
            return Decimal::from_f64(payoff).ok_or_else(|| Error::arithmetic("Failed to convert"));
        }

        let n = self.steps;
        let dt = time / n as f64;

        // Calculate tree parameters
        let (u, d, p) = match self.model {
            BinomialModel::CRR => Self::crr_params(vol_f, dt, rate_f, div_f),
            BinomialModel::JR => Self::jr_params(vol_f, dt, rate_f, div_f),
            BinomialModel::Tian => Self::tian_params(vol_f, dt, rate_f, div_f),
        };

        let disc = (-rate_f * dt).exp();
        let ud_ratio = u / d;

        // Step 1: Terminal payoffs - single vector, size N+1
        let mut v: Vec<f64> = Vec::with_capacity(n + 1);

        // S_{N,0} = S * d^N (only powi call for stock prices)
        let mut s_j = spot_f * d.powi(n as i32);

        for _ in 0..=n {
            v.push(Self::payoff(s_j, strike_f, option_type));
            s_j *= ud_ratio; // S_{N,j+1} = S_{N,j} * (u/d)
        }

        // Step 2: Backward induction (in-place, left-to-right sweep)
        for i in (0..n).rev() {
            // S_{i,0} = S * d^i
            let mut s_j = spot_f * d.powi(i as i32);

            for j in 0..=i {
                // Continuation value
                let cont = disc * (p * v[j + 1] + (1.0 - p) * v[j]);

                if american {
                    // American: max(continuation, intrinsic)
                    let intrinsic = Self::payoff(s_j, strike_f, option_type);
                    v[j] = cont.max(intrinsic);
                } else {
                    // European: continuation only
                    v[j] = cont;
                }

                s_j *= ud_ratio;
            }
        }

        Decimal::from_f64(v[0]).ok_or_else(|| Error::arithmetic("Failed to convert price"))
    }

    /// CRR parameters: u = e^(σ√Δt), d = 1/u, p = (e^((r-q)Δt) - d)/(u - d)
    fn crr_params(vol: f64, dt: f64, rate: f64, div: f64) -> (f64, f64, f64) {
        let sqrt_dt = dt.sqrt();
        let u = (vol * sqrt_dt).exp();
        let d = 1.0 / u;
        let growth = ((rate - div) * dt).exp();
        let p = (growth - d) / (u - d);
        (u, d, p)
    }

    /// Jarrow-Rudd parameters.
    fn jr_params(vol: f64, dt: f64, rate: f64, div: f64) -> (f64, f64, f64) {
        let sqrt_dt = dt.sqrt();
        let drift = (rate - div - 0.5 * vol * vol) * dt;
        let u = (drift + vol * sqrt_dt).exp();
        let d = (drift - vol * sqrt_dt).exp();
        (u, d, 0.5)
    }

    /// Tian parameters.
    fn tian_params(vol: f64, dt: f64, rate: f64, div: f64) -> (f64, f64, f64) {
        let v = ((rate - div) * dt).exp();
        let vv = v * v;
        let ud = (vol * vol * dt).exp();
        let d = v * (ud + 1.0 - (vv * ud * ud + 2.0 * ud + 1.0 - 4.0 * v * ud).sqrt()) / (ud + 1.0);
        let u = v * v / d;
        let p = (v - d) / (u - d);
        (u, d, p)
    }

    /// Option payoff.
    fn payoff(spot: f64, strike: f64, option_type: OptionType) -> f64 {
        match option_type {
            OptionType::Call => (spot - strike).max(0.0),
            OptionType::Put => (strike - spot).max(0.0),
        }
    }
}

impl Default for BinomialTree {
    fn default() -> Self {
        Self::crr(1000)
    }
}

impl PricingEngine for BinomialTree {
    fn price(&self, instrument: &dyn Instrument) -> Result<Money> {
        if let Some(option) = instrument.as_any().downcast_ref::<AmericanOption>() {
            let price = self.price_american(
                option.spot(),
                option.strike(),
                option.risk_free_rate(),
                option.volatility(),
                option.dividend_yield(),
                option.time_to_expiry(),
                option.option_type(),
            )?;
            Ok(Money::new(price, option.underlying_currency()))
        } else if let Some(option) = instrument.as_any().downcast_ref::<EuropeanOption>() {
            let price = self.price_european(
                option.spot(),
                option.strike(),
                option.risk_free_rate(),
                option.volatility(),
                Decimal::ZERO,
                option.time_to_expiry(),
                option.option_type(),
            )?;
            Ok(Money::new(price, option.underlying_currency()))
        } else {
            Err(Error::pricing(format!(
                "BinomialTree doesn't support {}",
                instrument.instrument_type()
            )))
        }
    }

    fn supports(&self, instrument: &dyn Instrument) -> bool {
        instrument.as_any().is::<AmericanOption>() || instrument.as_any().is::<EuropeanOption>()
    }

    fn name(&self) -> &'static str {
        "BinomialTree"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_european_call() {
        let tree = BinomialTree::crr(1000);
        let price = tree
            .price_european(
                dec!(100),
                dec!(100),
                dec!(0.05),
                dec!(0.2),
                Decimal::ZERO,
                1.0,
                OptionType::Call,
            )
            .unwrap();
        assert!(price > dec!(10) && price < dec!(11));
    }

    #[test]
    fn test_american_put_premium() {
        let tree = BinomialTree::crr(1000);
        let european = tree
            .price_european(
                dec!(80),
                dec!(100),
                dec!(0.05),
                dec!(0.2),
                Decimal::ZERO,
                1.0,
                OptionType::Put,
            )
            .unwrap();
        let american = tree
            .price_american(
                dec!(80),
                dec!(100),
                dec!(0.05),
                dec!(0.2),
                Decimal::ZERO,
                1.0,
                OptionType::Put,
            )
            .unwrap();
        assert!(american > european);
    }

    #[test]
    fn test_crr_params() {
        let (u, d, p) = BinomialTree::crr_params(0.2, 0.001, 0.05, 0.0);
        assert!(u > 1.0 && d < 1.0);
        assert!((d - 1.0 / u).abs() < 1e-10);
        assert!(p > 0.0 && p < 1.0);
    }

    #[test]
    fn test_expired_option() {
        let tree = BinomialTree::crr(100);
        let price = tree
            .price_european(
                dec!(110),
                dec!(100),
                dec!(0.05),
                dec!(0.2),
                Decimal::ZERO,
                0.0,
                OptionType::Call,
            )
            .unwrap();
        assert_eq!(price, dec!(10));
    }

    #[test]
    fn test_as_engine() {
        let tree = BinomialTree::crr(500);
        let option = AmericanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );
        let price = tree.price(&option).unwrap();
        assert!(price.amount() > dec!(0));
    }
}

//! Heston stochastic volatility model for European vanilla options.
//!
//! Prices calls and puts semi-analytically from the Heston characteristic
//! function using the "little Heston trap" formulation (Albrecher et al.,
//! 2007), with the two risk-neutral probabilities computed by Gauss-Legendre
//! quadrature over a truncated domain.
//!
//! Under Heston dynamics the variance follows a CIR process:
//!
//! dS = r S dt + √v S dW₁
//! dv = κ(θ - v) dt + σ √v dW₂,   d⟨W₁, W₂⟩ = ρ dt

use crate::core::error::{Error, Result};
use crate::core::money::Money;
use crate::core::traits::{Instrument, PricingEngine};
use crate::instruments::option::{EuropeanOption, OptionType};
use num_complex::Complex64;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::f64::consts::PI;
use std::sync::OnceLock;

/// Number of Gauss-Legendre quadrature nodes.
const QUAD_NODES: usize = 128;

/// Upper truncation of the semi-infinite characteristic function integral.
/// The integrand decays exponentially, so mass beyond this point is negligible
/// for realistic parameters.
const INTEGRATION_LIMIT: f64 = 200.0;

/// Parameters of the Heston stochastic volatility model.
///
/// These are model parameters, not monetary values, so `f64` is used
/// (consistent with how Greeks are represented).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HestonParams {
    /// Initial (spot) variance, v₀.
    pub v0: f64,
    /// Mean-reversion speed of the variance, κ.
    pub kappa: f64,
    /// Long-run variance, θ.
    pub theta: f64,
    /// Volatility of variance ("vol of vol"), σ.
    pub sigma: f64,
    /// Correlation between the asset and variance Brownian motions, ρ.
    pub rho: f64,
}

impl HestonParams {
    /// Create a new validated parameter set.
    pub fn new(v0: f64, kappa: f64, theta: f64, sigma: f64, rho: f64) -> Result<Self> {
        let params = Self {
            v0,
            kappa,
            theta,
            sigma,
            rho,
        };
        params.validate()?;
        Ok(params)
    }

    /// Validate parameter ranges.
    pub fn validate(&self) -> Result<()> {
        if !self.v0.is_finite() || self.v0 <= 0.0 {
            return Err(Error::invalid_input("Heston v0 must be positive"));
        }
        if !self.kappa.is_finite() || self.kappa <= 0.0 {
            return Err(Error::invalid_input("Heston kappa must be positive"));
        }
        if !self.theta.is_finite() || self.theta <= 0.0 {
            return Err(Error::invalid_input("Heston theta must be positive"));
        }
        if !self.sigma.is_finite() || self.sigma <= 0.0 {
            return Err(Error::invalid_input("Heston sigma must be positive"));
        }
        if !self.rho.is_finite() || self.rho <= -1.0 || self.rho >= 1.0 {
            return Err(Error::invalid_input(
                "Heston rho must be strictly between -1 and 1",
            ));
        }
        Ok(())
    }

    /// Check the Feller condition 2κθ ≥ σ², which guarantees the variance
    /// process stays strictly positive.
    pub fn feller_satisfied(&self) -> bool {
        2.0 * self.kappa * self.theta >= self.sigma * self.sigma
    }
}

/// Heston pricing model.
///
/// As a [`PricingEngine`] it supports [`EuropeanOption`], using the option's
/// spot, strike, rate, expiry, and type; the option's flat Black-Scholes
/// volatility field is ignored in favour of the engine's Heston parameters.
#[derive(Debug, Clone)]
pub struct Heston {
    params: HestonParams,
}

impl Heston {
    /// Create a new Heston engine with validated parameters.
    pub fn new(params: HestonParams) -> Result<Self> {
        params.validate()?;
        Ok(Self { params })
    }

    /// Get the model parameters.
    pub fn params(&self) -> &HestonParams {
        &self.params
    }

    /// Price a European option under the Heston model.
    pub fn price(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        params: &HestonParams,
        time: f64,
        option_type: OptionType,
    ) -> Result<Decimal> {
        match option_type {
            OptionType::Call => Self::price_call(spot, strike, rate, params, time),
            OptionType::Put => Self::price_put(spot, strike, rate, params, time),
        }
    }

    /// Price a European call option under the Heston model.
    ///
    /// Call = S·P₁ - K·e^(-rT)·P₂
    pub fn price_call(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        params: &HestonParams,
        time: f64,
    ) -> Result<Decimal> {
        let (spot_f, strike_f, rate_f) = Self::validate_inputs(spot, strike, rate, params, time)?;
        let price = Self::call_f64(spot_f, strike_f, rate_f, params, time);
        Decimal::try_from(price.max(0.0))
            .map_err(|_| Error::arithmetic("Failed to convert price to Decimal"))
    }

    /// Price a European put option under the Heston model.
    ///
    /// Computed from the call via put-call parity:
    /// Put = Call - S + K·e^(-rT)
    pub fn price_put(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        params: &HestonParams,
        time: f64,
    ) -> Result<Decimal> {
        let (spot_f, strike_f, rate_f) = Self::validate_inputs(spot, strike, rate, params, time)?;
        let call = Self::call_f64(spot_f, strike_f, rate_f, params, time);
        let put = call - spot_f + strike_f * (-rate_f * time).exp();
        Decimal::try_from(put.max(0.0))
            .map_err(|_| Error::arithmetic("Failed to convert price to Decimal"))
    }

    fn validate_inputs(
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        params: &HestonParams,
        time: f64,
    ) -> Result<(f64, f64, f64)> {
        params.validate()?;
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
        Ok((spot_f, strike_f, rate_f))
    }

    fn call_f64(spot: f64, strike: f64, rate: f64, params: &HestonParams, time: f64) -> f64 {
        let x = spot.ln();
        let ln_strike = strike.ln();
        let p1 = Self::probability(1, x, ln_strike, rate, time, params);
        let p2 = Self::probability(2, x, ln_strike, rate, time, params);
        spot * p1 - strike * (-rate * time).exp() * p2
    }

    /// Risk-neutral exercise probability Pⱼ (j = 1, 2):
    ///
    /// Pⱼ = 1/2 + (1/π) ∫₀^∞ Re[ e^(-iφ·lnK) · fⱼ(φ) / (iφ) ] dφ
    fn probability(
        j: u8,
        x: f64,
        ln_strike: f64,
        rate: f64,
        time: f64,
        params: &HestonParams,
    ) -> f64 {
        let half_limit = INTEGRATION_LIMIT / 2.0;
        let mut integral = 0.0;
        for &(node, weight) in quad_nodes() {
            // Map Gauss-Legendre nodes from [-1, 1] to [0, INTEGRATION_LIMIT].
            let phi = half_limit * (node + 1.0);
            let f = Self::characteristic_fn(j, phi, x, rate, time, params);
            let value =
                (Complex64::new(0.0, -phi * ln_strike).exp() * f / Complex64::new(0.0, phi)).re;
            integral += weight * value;
        }
        integral *= half_limit;
        (0.5 + integral / PI).clamp(0.0, 1.0)
    }

    /// Heston characteristic function fⱼ in the "little trap" formulation,
    /// which is numerically stable for long maturities.
    fn characteristic_fn(
        j: u8,
        phi: f64,
        x: f64,
        rate: f64,
        time: f64,
        params: &HestonParams,
    ) -> Complex64 {
        let i = Complex64::new(0.0, 1.0);
        let (u, b) = if j == 1 {
            (0.5, params.kappa - params.rho * params.sigma)
        } else {
            (-0.5, params.kappa)
        };
        let sigma2 = params.sigma * params.sigma;
        let iphi = i * phi;

        let beta = b - params.rho * params.sigma * iphi;
        let d = (beta * beta - sigma2 * (2.0 * u * iphi - phi * phi)).sqrt();
        let g = (beta - d) / (beta + d);
        let exp_dt = (-d * time).exp();

        let big_c = rate * iphi * time
            + (params.kappa * params.theta / sigma2)
                * ((beta - d) * time - 2.0 * ((1.0 - g * exp_dt) / (1.0 - g)).ln());
        let big_d = ((beta - d) / sigma2) * ((1.0 - exp_dt) / (1.0 - g * exp_dt));

        (big_c + big_d * params.v0 + iphi * x).exp()
    }
}

impl PricingEngine for Heston {
    fn price(&self, instrument: &dyn Instrument) -> Result<Money> {
        if let Some(option) = instrument.as_any().downcast_ref::<EuropeanOption>() {
            let price = Self::price(
                option.spot(),
                option.strike(),
                option.risk_free_rate(),
                &self.params,
                option.time_to_expiry(),
                option.option_type(),
            )?;
            Ok(Money::new(price, option.underlying_currency()))
        } else {
            Err(Error::pricing(format!(
                "Heston engine does not support instrument type: {}",
                instrument.instrument_type()
            )))
        }
    }

    fn supports(&self, instrument: &dyn Instrument) -> bool {
        instrument.as_any().is::<EuropeanOption>()
    }

    fn name(&self) -> &'static str {
        "Heston"
    }
}

/// Gauss-Legendre nodes and weights on [-1, 1], computed once.
fn quad_nodes() -> &'static [(f64, f64)] {
    static NODES: OnceLock<Vec<(f64, f64)>> = OnceLock::new();
    NODES.get_or_init(|| gauss_legendre(QUAD_NODES))
}

/// Compute n-point Gauss-Legendre nodes and weights via Newton iteration on
/// the Legendre polynomial recurrence.
fn gauss_legendre(n: usize) -> Vec<(f64, f64)> {
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        // Initial guess for the i-th root of Pₙ.
        let mut x = (PI * (i as f64 + 0.75) / (n as f64 + 0.5)).cos();
        let mut dp = 0.0;
        for _ in 0..100 {
            let mut p0 = 1.0;
            let mut p1 = x;
            for k in 2..=n {
                let kf = k as f64;
                let p2 = ((2.0 * kf - 1.0) * x * p1 - (kf - 1.0) * p0) / kf;
                p0 = p1;
                p1 = p2;
            }
            dp = n as f64 * (x * p1 - p0) / (x * x - 1.0);
            let dx = p1 / dp;
            x -= dx;
            if dx.abs() < 1e-15 {
                break;
            }
        }
        let weight = 2.0 / ((1.0 - x * x) * dp * dp);
        nodes.push((x, weight));
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::black_scholes::BlackScholes;
    use rust_decimal_macros::dec;

    fn test_params() -> HestonParams {
        HestonParams::new(0.04, 2.0, 0.04, 0.3, -0.7).unwrap()
    }

    #[test]
    fn test_gauss_legendre_integrates_polynomial() {
        // ∫₋₁¹ x² dx = 2/3
        let integral: f64 = gauss_legendre(16).iter().map(|&(x, w)| w * x * x).sum();
        assert!((integral - 2.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_params_validation() {
        assert!(HestonParams::new(0.04, 2.0, 0.04, 0.3, -0.7).is_ok());
        assert!(HestonParams::new(-0.04, 2.0, 0.04, 0.3, -0.7).is_err());
        assert!(HestonParams::new(0.04, 0.0, 0.04, 0.3, -0.7).is_err());
        assert!(HestonParams::new(0.04, 2.0, -0.04, 0.3, -0.7).is_err());
        assert!(HestonParams::new(0.04, 2.0, 0.04, 0.0, -0.7).is_err());
        assert!(HestonParams::new(0.04, 2.0, 0.04, 0.3, -1.0).is_err());
        assert!(HestonParams::new(0.04, 2.0, 0.04, 0.3, 1.5).is_err());
    }

    #[test]
    fn test_feller_condition() {
        // 2 * 2.0 * 0.04 = 0.16 >= 0.09
        assert!(test_params().feller_satisfied());
        // 2 * 0.5 * 0.04 = 0.04 < 1.0
        let violating = HestonParams::new(0.04, 0.5, 0.04, 1.0, -0.7).unwrap();
        assert!(!violating.feller_satisfied());
    }

    #[test]
    fn test_converges_to_black_scholes_for_small_vol_of_vol() {
        // With v0 = theta and vanishing vol of vol, Heston degenerates to
        // Black-Scholes with volatility sqrt(v0).
        let params = HestonParams::new(0.04, 2.0, 0.04, 0.001, 0.0).unwrap();
        let spot = dec!(100);
        let strike = dec!(100);
        let rate = dec!(0.05);
        let time = 1.0;

        let heston_call = Heston::price_call(spot, strike, rate, &params, time).unwrap();
        let bs_call = BlackScholes::price_call(spot, strike, rate, dec!(0.2), time).unwrap();
        let diff = (heston_call - bs_call).to_f64().unwrap().abs();
        assert!(diff < 0.01, "Heston {} vs BS {}", heston_call, bs_call);

        let heston_put = Heston::price_put(spot, strike, rate, &params, time).unwrap();
        let bs_put = BlackScholes::price_put(spot, strike, rate, dec!(0.2), time).unwrap();
        let diff = (heston_put - bs_put).to_f64().unwrap().abs();
        assert!(diff < 0.01, "Heston {} vs BS {}", heston_put, bs_put);
    }

    #[test]
    fn test_put_call_parity() {
        let params = test_params();
        let spot = dec!(100);
        let strike = dec!(95);
        let rate = dec!(0.03);
        let time = 0.75;

        let call = Heston::price_call(spot, strike, rate, &params, time).unwrap();
        let put = Heston::price_put(spot, strike, rate, &params, time).unwrap();

        // C - P = S - K * e^(-rT)
        let lhs = (call - put).to_f64().unwrap();
        let rhs = 100.0 - 95.0 * (-0.03_f64 * 0.75).exp();
        assert!((lhs - rhs).abs() < 1e-6);
    }

    #[test]
    fn test_call_respects_lower_bound() {
        let params = test_params();
        // Deep ITM call must be worth at least S - K * e^(-rT).
        let call = Heston::price_call(dec!(150), dec!(100), dec!(0.05), &params, 1.0)
            .unwrap()
            .to_f64()
            .unwrap();
        let lower_bound = 150.0 - 100.0 * (-0.05_f64).exp();
        assert!(call >= lower_bound - 1e-6);
    }

    #[test]
    fn test_price_increases_with_initial_variance() {
        let low = HestonParams::new(0.02, 2.0, 0.04, 0.3, -0.7).unwrap();
        let high = HestonParams::new(0.09, 2.0, 0.04, 0.3, -0.7).unwrap();
        let cheap = Heston::price_call(dec!(100), dec!(100), dec!(0.05), &low, 1.0).unwrap();
        let rich = Heston::price_call(dec!(100), dec!(100), dec!(0.05), &high, 1.0).unwrap();
        assert!(rich > cheap);
    }

    #[test]
    fn test_invalid_inputs_rejected() {
        let params = test_params();
        assert!(Heston::price_call(dec!(0), dec!(100), dec!(0.05), &params, 1.0).is_err());
        assert!(Heston::price_call(dec!(100), dec!(0), dec!(0.05), &params, 1.0).is_err());
        assert!(Heston::price_call(dec!(100), dec!(100), dec!(0.05), &params, 0.0).is_err());
    }

    #[test]
    fn test_pricing_engine_for_european_option() {
        let engine = Heston::new(test_params()).unwrap();
        let option = EuropeanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2), // ignored by the Heston engine
            1.0,
            OptionType::Call,
        );

        assert!(engine.supports(&option));
        let price = PricingEngine::price(&engine, &option).unwrap();
        assert!(price.amount() > Decimal::ZERO);
        assert_eq!(engine.name(), "Heston");
    }
}

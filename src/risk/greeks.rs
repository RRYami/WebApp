//! Option Greeks (risk sensitivities).

use std::fmt;

/// Greeks represent the sensitivities of option prices to various factors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Greeks {
    /// Delta: sensitivity to underlying price (first derivative).
    /// Range: 0 to 1 for calls, -1 to 0 for puts.
    pub delta: f64,

    /// Gamma: sensitivity of delta to underlying price (second derivative).
    /// Always positive.
    pub gamma: f64,

    /// Theta: sensitivity to time decay (per day).
    /// Typically negative for long options.
    pub theta: f64,

    /// Vega: sensitivity to volatility (for 1% change).
    /// Always positive.
    pub vega: f64,

    /// Rho: sensitivity to interest rates (for 1% change).
    /// Positive for calls, negative for puts.
    pub rho: f64,

    /// Phi (or vanna): sensitivity of delta to volatility.
    /// Also known as DvegaDspot or DdeltaDvol.
    pub phi: f64,
}

impl Greeks {
    /// Create a new Greeks struct with all values.
    pub fn new(delta: f64, gamma: f64, theta: f64, vega: f64, rho: f64) -> Self {
        Self {
            delta,
            gamma,
            theta,
            vega,
            rho,
            phi: 0.0,
        }
    }

    /// Create a new Greeks struct with all values including phi.
    pub fn with_phi(delta: f64, gamma: f64, theta: f64, vega: f64, rho: f64, phi: f64) -> Self {
        Self {
            delta,
            gamma,
            theta,
            vega,
            rho,
            phi,
        }
    }

    /// Create Greeks with all zeros.
    pub fn zeros() -> Self {
        Self {
            delta: 0.0,
            gamma: 0.0,
            theta: 0.0,
            vega: 0.0,
            rho: 0.0,
            phi: 0.0,
        }
    }

    /// Scale all Greeks by a factor.
    pub fn scale(&self, factor: f64) -> Self {
        Self {
            delta: self.delta * factor,
            gamma: self.gamma * factor,
            theta: self.theta * factor,
            vega: self.vega * factor,
            rho: self.rho * factor,
            phi: self.phi * factor,
        }
    }

    /// Get delta (convenience method).
    pub fn delta(&self) -> f64 {
        self.delta
    }

    /// Get gamma (convenience method).
    pub fn gamma(&self) -> f64 {
        self.gamma
    }

    /// Get theta (convenience method).
    pub fn theta(&self) -> f64 {
        self.theta
    }

    /// Get vega (convenience method).
    pub fn vega(&self) -> f64 {
        self.vega
    }

    /// Get rho (convenience method).
    pub fn rho(&self) -> f64 {
        self.rho
    }

    /// Get phi (convenience method).
    pub fn phi(&self) -> f64 {
        self.phi
    }

    /// Calculate the total P&L approximation for small changes.
    ///
    /// Uses Taylor expansion: ΔP ≈ Δ*ΔS + ½*Γ*ΔS² + Θ*Δt + ν*Δσ + ρ*Δr
    ///
    /// # Arguments
    ///
    /// * `delta_spot` - Change in underlying price.
    /// * `delta_time` - Change in time (days).
    /// * `delta_vol` - Change in volatility (decimal, e.g., 0.01 for 1%).
    /// * `delta_rate` - Change in rate (decimal, e.g., 0.01 for 1%).
    pub fn pnl_approximation(
        &self,
        delta_spot: f64,
        delta_time: f64,
        delta_vol: f64,
        delta_rate: f64,
    ) -> f64 {
        self.delta * delta_spot
            + 0.5 * self.gamma * delta_spot * delta_spot
            + self.theta * delta_time
            + self.vega * delta_vol * 100.0 // vega is per 1%
            + self.rho * delta_rate * 100.0 // rho is per 1%
    }

    /// Check if the Greeks are reasonable for a standard option.
    pub fn is_valid(&self) -> bool {
        // Delta should be between -1 and 1
        if self.delta < -1.0 || self.delta > 1.0 {
            return false;
        }

        // Gamma should be non-negative
        if self.gamma < 0.0 {
            return false;
        }

        // Vega should be non-negative
        if self.vega < 0.0 {
            return false;
        }

        true
    }

    /// Get the absolute sum of all Greeks (measure of total risk).
    pub fn total_risk(&self) -> f64 {
        self.delta.abs() + self.gamma + self.theta.abs() + self.vega + self.rho.abs()
    }

    /// Create a summary of the Greeks.
    pub fn summary(&self) -> String {
        format!(
            "Δ={:+.4} Γ={:.6} Θ={:+.6} ν={:.6} ρ={:+.6}",
            self.delta, self.gamma, self.theta, self.vega, self.rho
        )
    }
}

impl fmt::Display for Greeks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Greeks {{ delta: {:.4}, gamma: {:.6}, theta: {:.6}, vega: {:.6}, rho: {:.6} }}",
            self.delta, self.gamma, self.theta, self.vega, self.rho
        )
    }
}

/// Second-order Greeks (higher order sensitivities).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SecondOrderGreeks {
    /// Vanna: DdeltaDvol or DvegaDspot.
    /// Cross sensitivity to spot and volatility.
    pub vanna: f64,

    /// Charm: DdeltaDtime.
    /// Rate of change of delta over time.
    pub charm: f64,

    /// Vomma: DvegaDvol.
    /// Second derivative with respect to volatility.
    pub vomma: f64,

    /// Veta: DvegaDtime.
    /// Rate of change of vega over time.
    pub veta: f64,

    /// Speed: DgammaDspot.
    /// Third derivative with respect to spot.
    pub speed: f64,

    /// Zomma: DgammaDvol.
    /// Sensitivity of gamma to volatility.
    pub zomma: f64,

    /// Color: DgammaDtime.
    /// Rate of change of gamma over time.
    pub color: f64,

    /// Ultima: DvommaDvol.
    /// Third derivative with respect to volatility.
    pub ultima: f64,
}

/// Portfolio Greeks aggregated across multiple positions.
#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioGreeks {
    /// Net delta.
    pub net_delta: f64,
    /// Net gamma.
    pub net_gamma: f64,
    /// Net theta.
    pub net_theta: f64,
    /// Net vega.
    pub net_vega: f64,
    /// Net rho.
    pub net_rho: f64,
    /// Number of positions.
    pub position_count: usize,
}

impl PortfolioGreeks {
    /// Create a new empty portfolio Greeks.
    pub fn new() -> Self {
        Self {
            net_delta: 0.0,
            net_gamma: 0.0,
            net_theta: 0.0,
            net_vega: 0.0,
            net_rho: 0.0,
            position_count: 0,
        }
    }

    /// Add a position's Greeks to the portfolio.
    ///
    /// # Arguments
    ///
    /// * `greeks` - The Greeks of the position.
    /// * `quantity` - Number of contracts (can be negative for short positions).
    pub fn add_position(&mut self, greeks: &Greeks, quantity: f64) {
        self.net_delta += greeks.delta * quantity;
        self.net_gamma += greeks.gamma * quantity;
        self.net_theta += greeks.theta * quantity;
        self.net_vega += greeks.vega * quantity;
        self.net_rho += greeks.rho * quantity;
        self.position_count += 1;
    }

    /// Calculate the portfolio's total risk.
    pub fn total_risk(&self) -> f64 {
        self.net_delta.abs()
            + self.net_gamma
            + self.net_theta.abs()
            + self.net_vega
            + self.net_rho.abs()
    }

    /// Check if the portfolio is delta-neutral.
    pub fn is_delta_neutral(&self, tolerance: f64) -> bool {
        self.net_delta.abs() < tolerance
    }

    /// Check if the portfolio is gamma-neutral.
    pub fn is_gamma_neutral(&self, tolerance: f64) -> bool {
        self.net_gamma.abs() < tolerance
    }

    /// Create a hedge suggestion to make the portfolio delta-neutral.
    ///
    /// Returns the number of underlying units to buy (positive) or sell (negative).
    pub fn delta_hedge_suggestion(&self) -> f64 {
        -self.net_delta
    }
}

impl Default for PortfolioGreeks {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PortfolioGreeks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PortfolioGreeks {{ Δ={:+.4} Γ={:.6} Θ={:+.6} ν={:.6} ρ={:+.6} | Positions: {} }}",
            self.net_delta,
            self.net_gamma,
            self.net_theta,
            self.net_vega,
            self.net_rho,
            self.position_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeks_creation() {
        let greeks = Greeks::new(0.5, 0.05, -0.1, 0.3, 0.05);
        assert_eq!(greeks.delta, 0.5);
        assert_eq!(greeks.gamma, 0.05);
        assert_eq!(greeks.theta, -0.1);
        assert_eq!(greeks.vega, 0.3);
        assert_eq!(greeks.rho, 0.05);
    }

    #[test]
    fn test_greeks_zeros() {
        let greeks = Greeks::zeros();
        assert_eq!(greeks.delta, 0.0);
        assert_eq!(greeks.gamma, 0.0);
        assert_eq!(greeks.theta, 0.0);
        assert_eq!(greeks.vega, 0.0);
        assert_eq!(greeks.rho, 0.0);
    }

    #[test]
    fn test_greeks_scale() {
        let greeks = Greeks::new(0.5, 0.05, -0.1, 0.3, 0.05);
        let scaled = greeks.scale(2.0);
        assert_eq!(scaled.delta, 1.0);
        assert_eq!(scaled.gamma, 0.1);
        assert_eq!(scaled.theta, -0.2);
    }

    #[test]
    fn test_greeks_valid() {
        let valid = Greeks::new(0.5, 0.05, -0.1, 0.3, 0.05);
        assert!(valid.is_valid());

        let invalid_delta = Greeks::new(1.5, 0.05, -0.1, 0.3, 0.05);
        assert!(!invalid_delta.is_valid());

        let invalid_gamma = Greeks::new(0.5, -0.05, -0.1, 0.3, 0.05);
        assert!(!invalid_gamma.is_valid());
    }

    #[test]
    fn test_pnl_approximation() {
        let greeks = Greeks::new(0.5, 0.05, -0.1, 0.3, 0.05);
        // Small spot change of 1
        let pnl = greeks.pnl_approximation(1.0, 0.0, 0.0, 0.0);
        // Should be approximately delta + 0.5 * gamma
        assert!(pnl > 0.5);
    }

    #[test]
    fn test_portfolio_greeks() {
        let mut portfolio = PortfolioGreeks::new();

        let greeks1 = Greeks::new(0.5, 0.05, -0.1, 0.3, 0.05);
        let greeks2 = Greeks::new(-0.3, 0.03, -0.05, 0.2, 0.03);

        portfolio.add_position(&greeks1, 1.0);
        portfolio.add_position(&greeks2, 2.0);

        assert!((portfolio.net_delta - (-0.1)).abs() < 1e-10);
        assert_eq!(portfolio.position_count, 2);
    }

    #[test]
    fn test_delta_hedge() {
        let mut portfolio = PortfolioGreeks::new();
        let greeks = Greeks::new(0.5, 0.05, -0.1, 0.3, 0.05);
        portfolio.add_position(&greeks, 100.0);

        let hedge = portfolio.delta_hedge_suggestion();
        assert_eq!(hedge, -50.0);
    }

    #[test]
    fn test_is_delta_neutral() {
        let mut portfolio = PortfolioGreeks::new();
        assert!(portfolio.is_delta_neutral(0.01));

        let greeks = Greeks::new(0.5, 0.05, -0.1, 0.3, 0.05);
        portfolio.add_position(&greeks, 1.0);
        assert!(!portfolio.is_delta_neutral(0.01));
    }
}

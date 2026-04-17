//! Pricing engine trait and implementations.

pub use crate::core::traits::PricingEngine;

use crate::core::money::Money;

// Note: CompositeEngine removed because PricingEngine is not object-safe
// due to generic methods. Consider using an enum-based approach or
// trait objects with concrete types if needed.

/// Configuration for pricing engines.
#[derive(Debug, Clone)]
pub struct PricingConfig {
    /// Numerical precision tolerance.
    pub tolerance: f64,
    /// Maximum number of iterations for numerical methods.
    pub max_iterations: usize,
    /// Whether to use parallel computation where applicable.
    pub parallel: bool,
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            tolerance: 1e-10,
            max_iterations: 100,
            parallel: false,
        }
    }
}

/// Result of a pricing calculation with metadata.
#[derive(Debug, Clone)]
pub struct PricingResult {
    /// The calculated price.
    pub price: Money,
    /// Number of iterations used (for numerical methods).
    pub iterations: Option<usize>,
    /// Convergence status.
    pub converged: bool,
    /// Additional metadata.
    pub metadata: PricingMetadata,
}

/// Metadata for pricing calculations.
#[derive(Debug, Clone, Default)]
pub struct PricingMetadata {
    /// Method used for calculation.
    pub method: String,
    /// Time taken for calculation in milliseconds.
    pub computation_time_ms: Option<f64>,
    /// Any warnings or notes.
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::Result;
    use crate::core::traits::Instrument;

    #[allow(dead_code)]
    struct MockInstrument;
    impl Instrument for MockInstrument {
        fn notional(&self) -> crate::core::money::Money {
            use crate::core::currency::CurrencyCode;
            use rust_decimal_macros::dec;
            crate::core::money::Money::new(dec!(100), CurrencyCode::USD)
        }
        fn maturity(&self) -> Option<chrono::NaiveDate> {
            None
        }
        fn instrument_type(&self) -> &'static str {
            "MockInstrument"
        }
    }

    #[allow(dead_code)]
    struct MockEngine;
    impl PricingEngine for MockEngine {
        fn price<I: Instrument + 'static>(&self, _instrument: &I) -> Result<Money> {
            use crate::core::currency::CurrencyCode;
            use rust_decimal_macros::dec;
            Ok(crate::core::money::Money::new(dec!(50), CurrencyCode::USD))
        }
    }

    #[test]
    fn test_pricing_config_default() {
        let config = PricingConfig::default();
        assert_eq!(config.tolerance, 1e-10);
        assert_eq!(config.max_iterations, 100);
        assert!(!config.parallel);
    }

    // Note: CompositeEngine tests removed because the struct was removed
    // due to PricingEngine not being object-safe with generic methods
}

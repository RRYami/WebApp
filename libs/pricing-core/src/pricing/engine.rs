//! Pricing engine trait and implementations.

pub use crate::core::traits::PricingEngine;

use crate::core::error::{Error, Result};
use crate::core::money::Money;
use crate::core::traits::Instrument;
use std::collections::HashMap;

// Note: CompositeEngine removed because PricingEngine is not object-safe
// due to generic methods. Consider using an enum-based approach or
// trait objects with concrete types if needed.

/// Registry for managing multiple pricing engines at runtime.
///
/// This allows users to register different pricing engines and select
/// which one to use for pricing instruments dynamically.
///
/// # Example
///
/// ```
/// use pricing_core::EngineRegistry;
/// use pricing_core::prelude::*;
///
/// let mut registry = EngineRegistry::new();
/// registry.register("bs", Box::new(BlackScholes::new()));
/// registry.register("baw", Box::new(BaroneAdesiWhaley::new()));
///
/// let engines = registry.list_engines();
/// assert_eq!(engines.len(), 2);
/// ```
#[derive(Default)]
pub struct EngineRegistry {
    engines: HashMap<String, Box<dyn PricingEngine>>,
}

impl EngineRegistry {
    /// Create a new empty engine registry.
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
        }
    }

    /// Register a new pricing engine.
    ///
    /// # Arguments
    ///
    /// * `name` - A unique name for this engine
    /// * `engine` - The pricing engine implementation
    ///
    /// # Example
    ///
    /// ```
    /// use pricing_core::EngineRegistry;
    /// use pricing_core::prelude::*;
    ///
    /// let mut registry = EngineRegistry::new();
    /// registry.register("black_scholes", Box::new(BlackScholes::new()));
    /// ```
    pub fn register(&mut self, name: &str, engine: Box<dyn PricingEngine>) {
        self.engines.insert(name.to_string(), engine);
    }

    /// Price an instrument using a specific engine.
    ///
    /// # Arguments
    ///
    /// * `engine_name` - The name of the engine to use
    /// * `instrument` - The instrument to price
    ///
    /// # Returns
    ///
    /// The price as a Money value, or an error if the engine doesn't exist
    /// or doesn't support the instrument.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing_core::EngineRegistry;
    /// use pricing_core::prelude::*;
    ///
    /// let mut registry = EngineRegistry::new();
    /// registry.register("bs", Box::new(BlackScholes::new()));
    ///
    /// let option = EuropeanOption::new(
    ///     dec!(100), dec!(100), dec!(0.05),
    ///     dec!(0.2), 1.0, OptionType::Call,
    /// );
    ///
    /// let price = registry.price("bs", &option).unwrap();
    /// ```
    pub fn price(&self, engine_name: &str, instrument: &dyn Instrument) -> Result<Money> {
        let engine = self
            .engines
            .get(engine_name)
            .ok_or_else(|| Error::invalid_input(format!("Engine '{}' not found", engine_name)))?;

        if !engine.supports(instrument) {
            return Err(Error::invalid_input(format!(
                "Engine '{}' does not support instrument type '{}'",
                engine_name,
                instrument.instrument_type()
            )));
        }

        engine.price(instrument)
    }

    /// Get a reference to a specific engine.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the engine
    ///
    /// # Returns
    ///
    /// Some(&dyn PricingEngine) if found, None otherwise.
    pub fn get_engine(&self, name: &str) -> Option<&dyn PricingEngine> {
        self.engines.get(name).map(|e| e.as_ref())
    }

    /// List all registered engine names.
    ///
    /// # Returns
    ///
    /// A vector of engine names.
    pub fn list_engines(&self) -> Vec<String> {
        self.engines.keys().cloned().collect()
    }

    /// Check if an engine is registered.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to check
    ///
    /// # Returns
    ///
    /// true if the engine exists, false otherwise.
    pub fn has_engine(&self, name: &str) -> bool {
        self.engines.contains_key(name)
    }

    /// Remove an engine from the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the engine to remove
    ///
    /// # Returns
    ///
    /// The removed engine if it existed, None otherwise.
    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn PricingEngine>> {
        self.engines.remove(name)
    }

    /// Get the number of registered engines.
    pub fn len(&self) -> usize {
        self.engines.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.engines.is_empty()
    }

    /// Find engines that support a given instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument` - The instrument to check
    ///
    /// # Returns
    ///
    /// A vector of engine names that can price this instrument.
    pub fn find_supporting_engines(&self, instrument: &dyn Instrument) -> Vec<String> {
        self.engines
            .iter()
            .filter(|(_, engine)| engine.supports(instrument))
            .map(|(name, _)| name.clone())
            .collect()
    }
}

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
    use crate::instruments::option::{EuropeanOption, OptionType};
    use crate::{BaroneAdesiWhaley, BlackScholes};
    use rust_decimal_macros::dec;

    #[test]
    fn test_engine_registry_new() {
        let registry = EngineRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_engine_registry_default() {
        let registry: EngineRegistry = Default::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_register_and_price() {
        let mut registry = EngineRegistry::new();

        // Register BlackScholes engine
        registry.register("bs", Box::new(BlackScholes::new()));

        // Create an option
        let option = EuropeanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );

        // Price it
        let price = registry.price("bs", &option).unwrap();
        assert!(price.amount() > dec!(0));
    }

    #[test]
    fn test_list_engines() {
        let mut registry = EngineRegistry::new();
        registry.register("bs", Box::new(BlackScholes::new()));
        registry.register("baw", Box::new(BaroneAdesiWhaley::new()));

        let engines = registry.list_engines();
        assert_eq!(engines.len(), 2);
        assert!(engines.contains(&"bs".to_string()));
        assert!(engines.contains(&"baw".to_string()));
    }

    #[test]
    fn test_has_engine() {
        let mut registry = EngineRegistry::new();
        registry.register("bs", Box::new(BlackScholes::new()));

        assert!(registry.has_engine("bs"));
        assert!(!registry.has_engine("nonexistent"));
    }

    #[test]
    fn test_price_with_nonexistent_engine() {
        let registry = EngineRegistry::new();
        let option = EuropeanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );

        let result = registry.price("nonexistent", &option);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_supporting_engines() {
        let mut registry = EngineRegistry::new();
        registry.register("bs", Box::new(BlackScholes::new()));
        registry.register("baw", Box::new(BaroneAdesiWhaley::new()));

        let option = EuropeanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );

        let supporting = registry.find_supporting_engines(&option);
        assert!(supporting.contains(&"bs".to_string()));
        // BAW doesn't support European options, so shouldn't be in list
        assert!(!supporting.contains(&"baw".to_string()));
    }

    #[test]
    fn test_engine_not_supporting_instrument() {
        let mut registry = EngineRegistry::new();
        registry.register("baw", Box::new(BaroneAdesiWhaley::new()));

        let option = EuropeanOption::new(
            dec!(100),
            dec!(100),
            dec!(0.05),
            dec!(0.2),
            1.0,
            OptionType::Call,
        );

        let result = registry.price("baw", &option);
        assert!(result.is_err());
    }

    #[test]
    fn test_unregister() {
        let mut registry = EngineRegistry::new();
        registry.register("bs", Box::new(BlackScholes::new()));

        assert!(registry.has_engine("bs"));

        let removed = registry.unregister("bs");
        assert!(removed.is_some());
        assert!(!registry.has_engine("bs"));
    }

    #[test]
    fn test_pricing_config_default() {
        let config = PricingConfig::default();
        assert_eq!(config.tolerance, 1e-10);
        assert_eq!(config.max_iterations, 100);
        assert!(!config.parallel);
    }
}

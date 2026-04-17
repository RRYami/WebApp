# Traits: Defining Shared Behavior

## 🎭 What Are Traits?

Traits define shared behavior—similar to interfaces in other languages, but more powerful:

```rust
// Define what it means to be "pricable"
pub trait Pricable {
    fn price(&self) -> Result<Money>;
}

// Define what it means to be an "instrument"
pub trait Instrument {
    fn notional(&self) -> Money;
    fn maturity(&self) -> Option<NaiveDate>;
}
```

**Key insight**: Traits decouple "what something can do" from "how it's implemented."

## 🔧 Basic Trait Syntax

### Defining a Trait

```rust
pub trait HasGreeks {
    // Required methods (must be implemented)
    fn greeks(&self) -> Result<Greeks>;
    fn delta(&self) -> Result<f64>;
    fn gamma(&self) -> Result<f64>;
    
    // Provided methods (default implementation)
    fn vega(&self) -> Result<f64> {
        self.greeks().map(|g| g.vega)
    }
}
```

### Implementing a Trait

```rust
impl HasGreeks for EuropeanOption {
    fn greeks(&self) -> Result<Greeks> {
        BlackScholes::greeks(
            self.spot(), self.strike(),
            self.risk_free_rate(), self.volatility(),
            self.time_to_expiry(), self.option_type()
        )
    }
    
    fn delta(&self) -> Result<f64> {
        self.greeks().map(|g| g.delta)
    }
    
    fn gamma(&self) -> Result<f64> {
        self.greeks().map(|g| g.gamma)
    }
}
```

## 🎯 Traits in the Codebase

### 1. **Instrument Trait**

The base trait for all financial instruments:

```rust
pub trait Instrument {
    fn notional(&self) -> Money;
    fn maturity(&self) -> Option<NaiveDate>;
    fn instrument_type(&self) -> &'static str;
    
    // Default implementation
    fn currency(&self) -> CurrencyCode {
        self.notional().currency()
    }
}
```

**Usage**:
```rust
impl Instrument for EuropeanOption {
    fn notional(&self) -> Money {
        Money::new(Decimal::ONE, self.underlying_currency())
    }
    
    fn maturity(&self) -> Option<NaiveDate> {
        None  // We store time to expiry, not a date
    }
    
    fn instrument_type(&self) -> &'static str {
        "EuropeanOption"
    }
}
```

### 2. **Pricable Trait**

For instruments that can be priced:

```rust
pub trait Pricable {
    fn price(&self) -> Result<Money>;
    fn price_with<E: PricingEngine>(&self, engine: &E) -> Result<Money>;
}

impl Pricable for EuropeanOption {
    fn price(&self) -> Result<Money> {
        // Use default Black-Scholes
        let price = BlackScholes::price(
            self.spot(), self.strike(),
            self.risk_free_rate(), self.volatility(),
            self.time_to_expiry(), self.option_type()
        )?;
        Ok(Money::new(price, self.underlying_currency()))
    }
    
    fn price_with<E: PricingEngine>(&self, engine: &E) -> Result<Money> {
        // Use custom pricing engine
        engine.price(self)
    }
}
```

### 3. **PricingEngine Trait**

For different pricing models:

```rust
pub trait PricingEngine {
    fn price<I: Instrument>(&self, instrument: &I) -> Result<Money>;
}

// Black-Scholes implementation
pub struct BlackScholes;

impl PricingEngine for BlackScholes {
    fn price<I: Instrument>(&self, instrument: &I) -> Result<Money> {
        // Implementation specific to Black-Scholes
    }
}

// Could add more engines:
// - MonteCarlo
// - BinomialTree
// - FiniteDifference
```

## 🔗 Trait Bounds

### Constraining Generics

```rust
// Only accept types that implement Instrument
fn print_maturity<I: Instrument>(instrument: &I) {
    match instrument.maturity() {
        Some(date) => println!("Matures on: {}", date),
        None => println!("No maturity date"),
    }
}

// Multiple trait bounds
fn price_and_print<I>(instrument: &I) -> Result<Money>
where
    I: Instrument + Pricable,
{
    let price = instrument.price()?;
    println!("{} price: {}", instrument.instrument_type(), price);
    Ok(price)
}
```

### Associated Types vs Generics

**Generic approach** (what we use):
```rust
pub trait PricingEngine {
    fn price<I: Instrument>(&self, instrument: &I) -> Result<Money>;
}
// One engine can price many instruments
```

**Associated type approach**:
```rust
pub trait PricingEngine {
    type Instrument: Instrument;
    fn price(&self, instrument: &Self::Instrument) -> Result<Money>;
}
// Each engine prices one specific instrument type
```

**Trade-offs**:
- Generics: More flexible, but can't make trait objects (dyn Trait)
- Associated types: Less flexible, but can make trait objects

## 🧬 Trait Inheritance

Traits can require other traits:

```rust
// CashFlowGenerating requires Instrument
pub trait CashFlowGenerating: Instrument {
    fn cash_flows(&self) -> Vec<(NaiveDate, Money)>;
}

// Bond implements both
impl Instrument for CouponBond { ... }
impl CashFlowGenerating for CouponBond { ... }
```

**Benefits**:
- Type safety: Can't have cash flows without being an instrument
- Code reuse: Methods requiring `Instrument` also work with `CashFlowGenerating`

## 🎨 Blanket Implementations

Implement traits for all types that meet criteria:

```rust
// Not in our codebase, but useful pattern:
impl<T> Instrument for T
where
    T: Clone + Debug,
{
    // Default implementation
}
```

## 🔍 Advanced Trait Features

### Trait Objects

When you need runtime polymorphism:

```rust
// Static dispatch (monomorphization)
fn price_static<I: Pricable>(instrument: &I) -> Result<Money> {
    instrument.price()  // Compiled for each type
}

// Dynamic dispatch (vtable)
fn price_dyn(instrument: &dyn Pricable) -> Result<Money> {
    instrument.price()  // Runtime lookup
}
```

**Trade-offs**:
- Static: Faster, but code bloat
- Dynamic: Slower, but smaller code

### Marker Traits

Traits with no methods, just semantics:

```rust
// Send + Sync are marker traits from std
// Our instruments are automatically Send + Sync if their fields are
pub struct EuropeanOption { ... }  // Automatically Send + Sync
```

**Why this matters**: Thread safety is checked at compile time!

## 🧪 Testing with Traits

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Mock instrument for testing
    struct MockInstrument {
        price: Money,
    }

    impl Instrument for MockInstrument {
        fn notional(&self) -> Money { self.price }
        fn maturity(&self) -> Option<NaiveDate> { None }
        fn instrument_type(&self) -> &'static str { "Mock" }
    }

    impl Pricable for MockInstrument {
        fn price(&self) -> Result<Money> { Ok(self.price) }
        fn price_with<E: PricingEngine>(&self, _: &E) -> Result<Money> {
            Ok(self.price)
        }
    }

    #[test]
    fn test_pricable() {
        let mock = MockInstrument {
            price: Money::new(dec!(100), CurrencyCode::USD),
        };
        assert_eq!(mock.price().unwrap().amount(), dec!(100));
    }
}
```

## 🎯 Real-World Pattern: Plugin System

Traits enable extensible architectures:

```rust
// Define plugin interface
pub trait PricingModel: PricingEngine {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
}

// Users can add new models
pub struct HestonModel;
impl PricingModel for HestonModel { ... }
impl PricingEngine for HestonModel { ... }

// Registry stores different models
pub struct ModelRegistry {
    models: Vec<Box<dyn PricingModel>>,
}
```

## 💡 Key Takeaways

1. **Traits define behavior**: What can this type do?
2. **Implementations provide behavior**: How does this type do it?
3. **Trait bounds constrain generics**: Only accept types with certain capabilities
4. **Default implementations**: Provide common behavior, override when needed
5. **Composition over inheritance**: Traits compose better than class hierarchies

## 🎯 Exercises

1. **Add a new trait**: Create a `HasYield` trait for instruments with yield metrics
2. **Implement for multiple types**: Make both `Bond` and `Option` implement it
3. **Use trait bounds**: Write a function that accepts any `HasYield + Pricable` type

```rust
// Exercise template
pub trait HasYield {
    fn yield_to_maturity(&self, price: Money) -> Result<f64>;
    fn current_yield(&self, price: Money) -> Result<f64>;
}

fn analyze_yield<I>(instrument: &I, price: Money) -> Result<()>
where
    I: HasYield + Pricable + Instrument,
{
    let ytm = instrument.yield_to_maturity(price)?;
    let theoretical = instrument.price()?;
    println!("{}:", instrument.instrument_type());
    println!("  Price: {}", price);
    println!("  Theoretical: {}", theoretical);
    println!("  YTM: {:.2}%", ytm * 100.0);
    Ok(())
}
```

---

Next: [Error Handling](./04-error-handling.md)

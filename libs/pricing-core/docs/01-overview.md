# Project Overview: Architecture & Design

## 🎯 Design Philosophy

This library demonstrates how Rust's type system can prevent common financial programming errors:

1. **Currency Safety**: You can't accidentally add USD to EUR
2. **Precision**: No floating-point errors in monetary calculations
3. **Correctness**: The type system enforces valid financial instrument states

## 🏛️ Architecture Layers

### Layer 1: Core Types (Foundation)

The foundation provides types that are mathematically correct and safe:

```rust
// core/money.rs
pub struct Money {
    amount: Decimal,      // Exact decimal representation
    currency: CurrencyCode, // ISO 4217 currency code
}
```

**Why Decimal?**
- Binary floating-point (f64) can't represent 0.1 exactly
- Financial calculations require exact precision
- `rust_decimal` provides 28 decimal places of precision

### Layer 2: Traits (Contracts)

Traits define what financial instruments can do:

```rust
// core/traits.rs
pub trait Instrument {
    fn notional(&self) -> Money;
    fn maturity(&self) -> Option<NaiveDate>;
    fn instrument_type(&self) -> &'static str;
}

pub trait Pricable {
    fn price(&self) -> Result<Money>;
}
```

**Key Insight**: Traits are like interfaces in other languages, but more powerful. They enable:
- Polymorphism without inheritance
- Zero-cost abstractions
- Compile-time type checking

### Layer 3: Instruments (Domain Models)

Concrete implementations of financial instruments:

```rust
// instruments/option.rs
pub struct EuropeanOption {
    strike: Decimal,
    spot: Decimal,
    volatility: Decimal,
    // ...
}

impl Instrument for EuropeanOption { ... }
impl Pricable for EuropeanOption { ... }
```

### Layer 4: Pricing Engines (Algorithms)

Separate pricing logic from instrument data:

```rust
// pricing/black_scholes.rs
pub struct BlackScholes;

impl PricingEngine for BlackScholes {
    fn price<I: Instrument>(&self, instrument: &I) -> Result<Money> {
        // Algorithm implementation
    }
}
```

**Why this separation?**
- Same instrument can be priced by different models
- Easy to add new pricing models
- Test models independently

## 🧩 Module System

Rust's module system organizes code hierarchically:

```rust
// lib.rs - Library entry point
pub mod core;           // Makes core module public
pub mod instruments;    // Makes instruments module public

// core/mod.rs - Core submodule declarations
pub mod currency;
pub mod money;
// ...

// Re-exports for convenient access
pub use core::money::Money;
```

### Public vs Private

```rust
// Private by default
struct InternalHelper;  // Only visible in this module

// Public with `pub`
pub struct Money;       // Visible outside the module

// Pub(crate) - visible within the crate
pub(crate) fn helper(); // Visible throughout the crate
```

## 🎨 Design Patterns

### 1. **Typestate Pattern**

Use types to represent valid states:

```rust
// Invalid state is unrepresentable
pub struct EuropeanOption {
    strike: Decimal,    // Must be positive
    spot: Decimal,      // Must be positive
    time_to_expiry: f64, // Must be positive
}

impl EuropeanOption {
    pub fn new(strike: Decimal, spot: Decimal, ...) -> Self {
        // Validation ensures valid state
        assert!(strike > 0);
        assert!(spot > 0);
        // ...
    }
}
```

### 2. **Newtype Pattern**

Wrap primitive types to add meaning:

```rust
// Instead of using String for currency codes
pub struct CurrencyCode([u8; 3]);

impl CurrencyCode {
    pub fn new(code: &str) -> Result<Self> {
        // Validate ISO 4217 format
    }
}
```

### 3. **Builder Lite**

Simple constructors with many parameters:

```rust
let option = EuropeanOption::new(
    dec!(100),   // strike
    dec!(105),   // spot
    dec!(0.05),  // rate
    dec!(0.2),   // vol
    1.0,         // time
    OptionType::Call,
);
```

## 📦 Crate Organization

### Cargo.toml

```toml
[package]
name = "pricing_lib"
version = "0.1.0"
edition = "2021"

[dependencies]
rust_decimal = { version = "1.35", features = ["maths"] }
chrono = "0.4"
thiserror = "1.0"

[dev-dependencies]
approx = "0.5"
criterion = "0.5"
```

**Key dependencies:**
- `rust_decimal`: Exact decimal arithmetic
- `chrono`: Date and time handling
- `thiserror`: Easy error type derivation
- `criterion`: Benchmarking

## 🔐 Safety Guarantees

### Compile-Time Checks

```rust
// This will NOT compile - type mismatch!
let usd = Money::new(dec!(100), CurrencyCode::USD);
let eur = Money::new(dec!(100), CurrencyCode::EUR);
let sum = usd + eur;  // Error: mismatched types!
```

### Runtime Validation

```rust
// This will panic at runtime
let invalid = CurrencyCode::new("US");  // Error: must be 3 chars
```

### Mathematical Correctness

```rust
// Zero volatility is caught before calculation
let result = BlackScholes::price_call(
    dec!(100), dec!(100), dec!(0.05),
    dec!(0),  // Zero volatility!
    1.0
);  // Returns Err, not NaN or panic
```

## 🎯 Exercises

1. **Add a new currency**: Add `CurrencyCode::CHF` (Swiss Franc) to the currency module
2. **Create a new instrument**: Implement a `Forward` contract type
3. **Add validation**: Ensure option time to expiry is not negative

---

Next: [Core Concepts](./02-core-concepts.md)

# Pricing Library: Learning Rust Through Finance

Welcome! This documentation teaches Rust programming concepts through a practical finance pricing library. Each section builds on the previous one, explaining both the financial concepts and the Rust patterns used.

## 📚 Documentation Structure

1. **[Overview](./01-overview.md)** - Project architecture and design philosophy
2. **[Core Concepts](./02-core-concepts.md)** - Money, Currency, and Interest Rates
3. **[Traits in Action](./03-traits.md)** - Understanding Rust's trait system
4. **[Error Handling](./04-error-handling.md)** - The `Result` type and custom errors
5. **[Option Pricing](./05-option-pricing.md)** - Black-Scholes model implementation
6. **[Bond Pricing](./06-bond-pricing.md)** - Fixed income calculations
7. **[Testing](./07-testing.md)** - Unit tests, integration tests, benchmarks
8. **[Rust Patterns](./08-rust-patterns.md)** - Common idioms and best practices
9. **[Practice Exercises](./09-exercises.md)** - Hands-on coding challenges

## 🎯 What You'll Learn

By studying this codebase, you'll learn:

- **Type Safety**: Using Rust's type system to prevent financial calculation errors
- **Traits**: How to define shared behavior across different financial instruments
- **Precision**: Handling monetary values with `rust_decimal` for exact calculations
- **Error Handling**: Graceful handling of invalid inputs and calculation errors
- **Testing**: Writing comprehensive tests for numerical algorithms
- **Performance**: Benchmarking and optimizing financial calculations

## 🏗️ Project Structure

```
src/
├── lib.rs              # Library entry point, module declarations
├── main.rs             # CLI entry point
├── core/               # Fundamental types and utilities
│   ├── currency.rs     # Currency codes and definitions
│   ├── money.rs        # Money type with arithmetic
│   ├── interest_rate.rs # Interest rate calculations
│   ├── day_count.rs    # Day count conventions
│   ├── error.rs        # Error types
│   └── traits.rs       # Core trait definitions
├── instruments/        # Financial instruments
│   ├── bond.rs         # Bond types and pricing
│   └── option.rs       # Option types
├── pricing/            # Pricing engines
│   ├── black_scholes.rs # Black-Scholes model
│   └── engine.rs       # Pricing engine trait
├── risk/               # Risk calculations
│   └── greeks.rs       # Option Greeks
└── utils/              # Utility functions
    └── mod.rs
```

## 🚀 Quick Start

Run the examples:
```bash
# Option pricing example
cargo run --example option_pricing

# Bond pricing example
cargo run --example bond_pricing

# Run all tests
cargo test

# Run benchmarks
cargo bench
```

## 💡 Key Rust Concepts Used

### 1. **Newtype Pattern**
```rust
// Money wraps a Decimal to add currency information
pub struct Money {
    amount: Decimal,
    currency: CurrencyCode,
}
```

### 2. **Trait Bounds**
```rust
// PricingEngine can price any instrument
pub trait PricingEngine {
    fn price<I: Instrument>(&self, instrument: &I) -> Result<Money>;
}
```

### 3. **Associated Types vs Generics**
We use generics for flexibility - the same pricing engine can price different instruments.

### 4. **Error Propagation**
```rust
// The ? operator makes error handling clean
let price = option.price()?;
let greeks = option.greeks()?;
```

### 5. **Builder Pattern**
```rust
let option = EuropeanOption::new(strike, spot, rate, vol, time, option_type);
```

## 📖 Reading Guide

### For Beginners
Start with [Core Concepts](./02-core-concepts.md) to understand the basic types, then move to [Traits](./03-traits.md) to see how Rust's type system works.

### For Experienced Developers
Jump to [Option Pricing](./05-option-pricing.md) or [Bond Pricing](./06-bond-pricing.md) to see the mathematical implementations, then review [Rust Patterns](./08-rust-patterns.md) for idiomatic code.

### For Finance Professionals
Start with [Overview](./01-overview.md) to understand the architecture, then dive into specific instrument implementations.

## 🔗 Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [rust_decimal Documentation](https://docs.rs/rust_decimal/)
- [Black-Scholes Model](https://en.wikipedia.org/wiki/Black%E2%80%93Scholes_model)
- [Day Count Conventions](https://en.wikipedia.org/wiki/Day_count_convention)

---

Ready to start? Head to [Overview](./01-overview.md)!

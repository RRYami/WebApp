# Pricing Library Documentation

Welcome to the Pricing Library learning documentation! This comprehensive guide teaches Rust programming through a practical finance library.

## 📚 Table of Contents

### Getting Started
1. **[Introduction](./00-introduction.md)** - Overview and learning goals
2. **[Project Overview](./01-overview.md)** - Architecture and design philosophy

### Core Concepts
3. **[Core Concepts](./02-core-concepts.md)** - Money, Currency, Interest Rates, and Precision
4. **[Traits in Action](./03-traits.md)** - Understanding Rust's trait system
5. **[Error Handling](./04-error-handling.md)** - The `Result` type and custom errors

### Financial Instruments
6. **[Option Pricing](./05-option-pricing.md)** - Black-Scholes model implementation
7. **[Bond Pricing](./06-bond-pricing.md)** - Fixed income calculations

### Advanced Topics
8. **[Testing](./07-testing.md)** - Unit tests, integration tests, and benchmarks
9. **[Rust Patterns](./08-rust-patterns.md)** - Common idioms and best practices
10. **[Practice Exercises](./09-exercises.md)** - Hands-on coding challenges

## 🚀 Quick Start

### Run Examples
```bash
# Option pricing example
cargo run --example option_pricing

# Bond pricing example
cargo run --example bond_pricing
```

### Run Tests
```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench
```

### Read the Docs
Start with [Introduction](./00-introduction.md) if you're new, or jump to specific topics above.

## 🎯 What You'll Learn

### Rust Fundamentals
- **Type Safety**: Using types to prevent errors
- **Traits**: Defining shared behavior
- **Error Handling**: The `Result` type
- **Ownership & Borrowing**: Memory safety without GC
- **Generics**: Writing flexible code

### Financial Concepts
- **Option Pricing**: Black-Scholes model
- **Greeks**: Risk sensitivities
- **Bond Math**: YTM, duration, convexity
- **Day Count Conventions**: ACT/360, 30/360, etc.
- **Interest Rates**: Compounding methods

### Software Engineering
- **Testing**: Unit, integration, and property-based tests
- **Documentation**: Doc comments and examples
- **Benchmarking**: Performance measurement
- **API Design**: Clean, ergonomic interfaces

## 📊 Benchmark Results

| Operation | Time | Throughput |
|-----------|------|------------|
| Black-Scholes Price | 322 ns | ~3.1M ops/sec |
| Greeks Calculation | 559 ns | ~1.8M ops/sec |
| Implied Volatility | 2.14 µs | ~467K ops/sec |
| Bond Pricing | 15.3 µs | ~65K ops/sec |

## 🎓 Recommended Reading Order

### For Rust Beginners
1. Introduction → Core Concepts → Traits → Error Handling
2. Testing → Rust Patterns
3. Option/Bond Pricing (financial background)
4. Exercises

### For Experienced Developers
1. Overview → Rust Patterns (quick Rust review)
2. Option/Bond Pricing (implementation details)
3. Testing (see our approach)
4. Exercises

### For Finance Professionals
1. Overview (architecture)
2. Core Concepts (Rust types for finance)
3. Option/Bond Pricing (algorithms)
4. Traits/Error Handling (Rust specifics)

## 🔗 Additional Resources

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises

### Finance
- [Options, Futures, and Other Derivatives](https://www.pearson.com/en-us/subject-catalog/p/options-futures-and-other-derivatives/P200000005792) by John Hull
- [The Handbook of Fixed Income Securities](https://www.mhprofessional.com/the-handbook-of-fixed-income-securities-ninth-edition-9780071768467-usa) by Frank Fabozzi

### Libraries Used
- [rust_decimal](https://docs.rs/rust_decimal/) - Decimal arithmetic
- [chrono](https://docs.rs/chrono/) - Date and time handling
- [thiserror](https://docs.rs/thiserror/) - Error handling
- [criterion](https://docs.rs/criterion/) - Benchmarking

## 💡 Key Takeaways

### Rust Makes Financial Code Safer

```rust
// This won't compile - types prevent currency mismatch!
let usd = Money::new(dec!(100), CurrencyCode::USD);
let eur = Money::new(dec!(100), CurrencyCode::EUR);
let sum = usd + eur;  // Error!
```

### Decimal for Precision

```rust
// Exact representation
let a = dec!(0.1);
let b = dec!(0.2);
assert_eq!(a + b, dec!(0.3));  // true!

// Floating-point would fail
let a: f64 = 0.1;
let b: f64 = 0.2;
assert_eq!(a + b, 0.3);  // false!
```

### Traits Enable Flexibility

```rust
// Works with any Pricable instrument
fn print_price<P: Pricable>(instrument: &P) {
    println!("{}", instrument.price().unwrap());
}

// Can price options, bonds, forwards...
print_price(&option);
print_price(&bond);
```

## 🤝 Contributing

Found an issue or want to improve the docs? The documentation is in the `docs/` directory alongside the code.

## 📄 License

This documentation is part of the pricing_lib project. See the main project for license details.

---

**Ready to start?** → [Introduction](./00-introduction.md)

Happy learning! 🦀📈

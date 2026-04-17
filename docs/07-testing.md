# Testing: Ensuring Correctness

## 🎯 Testing Philosophy

In financial software, correctness is critical. This library uses multiple testing strategies:

1. **Unit tests** - Test individual functions
2. **Integration tests** - Test component interactions
3. **Doc tests** - Test examples in documentation
4. **Benchmarks** - Measure performance

## 🧪 Unit Tests

Unit tests live alongside the code they test:

```rust
// In src/core/money.rs

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_money_addition() {
        let m1 = Money::new(dec!(100), CurrencyCode::USD);
        let m2 = Money::new(dec!(50), CurrencyCode::USD);
        let result = m1.checked_add(&m2).unwrap();
        
        assert_eq!(result.amount(), dec!(150));
    }

    #[test]
    fn test_money_currency_mismatch() {
        let usd = Money::new(dec!(100), CurrencyCode::USD);
        let eur = Money::new(dec!(100), CurrencyCode::EUR);
        
        assert!(usd.checked_add(&eur).is_err());
    }
}
```

**Key points:**
- `#[cfg(test)]` compiles only during testing
- `use super::*;` brings parent module into scope
- Tests are just functions with `#[test]` attribute

## 🔗 Integration Tests

Integration tests are in the `tests/` directory:

```rust
// tests/integration_tests.rs

use pricing_lib::prelude::*;

#[test]
fn test_end_to_end_option_pricing() {
    // Create option
    let option = EuropeanOption::new(
        dec!(100), dec!(105), dec!(0.05),
        dec!(0.25), 0.5, OptionType::Call,
    );
    
    // Price it
    let price = option.price().expect("Should price successfully");
    assert!(price.amount() > dec!(0));
    
    // Get Greeks
    let greeks = option.greeks().expect("Should calculate Greeks");
    assert!(greeks.delta > 0.0 && greeks.delta < 1.0);
}
```

**Why separate integration tests?**
- Test public API like a user would
- Ensure modules work together
- Test at the crate boundary

## 📚 Doc Tests

Code examples in documentation become tests:

```rust
/// Create a new money value.
///
/// # Examples
///
/// ```
/// use pricing_lib::prelude::*;
/// use rust_decimal_macros::dec;
///
/// let amount = Money::new(dec!(100.50), CurrencyCode::USD);
/// ```
pub fn new(amount: Decimal, currency: CurrencyCode) -> Self {
    Self { amount, currency }
}
```

Run doc tests:
```bash
cargo test --doc
```

## ⚡ Testing Numerical Code

### Approximate Equality

Floating-point comparisons need tolerance:

```rust
// Don't do this!
assert_eq!(ndf(0.0), 0.5);  // Might fail due to precision

// Do this instead
assert!((ndf(0.0) - 0.5).abs() < 1e-10);

// Or use the approx crate
use approx::assert_relative_eq;
assert_relative_eq!(greeks.delta, 0.5, epsilon = 0.1);
```

### Property-Based Testing

Test mathematical properties:

```rust
#[test]
fn test_put_call_parity() {
    let spot = dec!(100);
    let strike = dec!(100);
    let rate = dec!(0.05);
    let vol = dec!(0.2);
    let time = 1.0;
    
    let call = BlackScholes::price_call(spot, strike, rate, vol, time).unwrap();
    let put = BlackScholes::price_put(spot, strike, rate, vol, time).unwrap();
    
    // C - P = S - K*e^(-rT)
    let lhs = call - put;
    let df = (-rate * Decimal::from(time)).exp();
    let rhs = spot - strike * df;
    
    assert!((lhs - rhs).abs() < dec!(0.01));
}
```

### Edge Cases

Test boundary conditions:

```rust
#[test]
fn test_zero_volatility_error() {
    let result = BlackScholes::price_call(
        dec!(100), dec!(100), dec!(0.05),
        dec!(0),  // Zero volatility!
        1.0,
    );
    assert!(result.is_err());
}

#[test]
fn test_same_date() {
    let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let yf = DayCountConvention::Act360.year_fraction(d, d);
    assert_eq!(yf, Decimal::ZERO);
}

#[test]
fn test_reverse_dates() {
    let start = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let yf = DayCountConvention::Act360.year_fraction(start, end);
    assert_eq!(yf, Decimal::ZERO);  // Should return 0, not negative
}
```

## 🎯 Test Organization

### Naming Convention

```rust
// Format: test_<function>_<scenario>
#[test]
fn test_price_call_atm() { ... }  // At-the-money

#[test]
fn test_price_call_itm() { ... }  // In-the-money

#[test]
fn test_price_call_otm() { ... }  // Out-of-the-money
```

### Grouping Tests

Use modules to organize:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod money_tests {
        use super::*;
        
        #[test]
        fn test_addition() { ... }
        
        #[test]
        fn test_subtraction() { ... }
    }
    
    mod currency_tests {
        use super::*;
        
        #[test]
        fn test_validation() { ... }
    }
}
```

## 🚀 Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_money_addition

# Run tests matching pattern
cargo test money_

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

### Test Output

```bash
$ cargo test

running 69 tests
test core::money::tests::test_money_addition ... ok
test core::money::tests::test_money_creation ... ok
...
test result: ok. 69 passed; 0 failed; 0 ignored
```

## 📊 Benchmarks

Benchmarks measure performance:

```rust
// benches/pricing_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pricing_lib::prelude::*;

fn benchmark_option_pricing(c: &mut Criterion) {
    c.bench_function("black_scholes_call", |b| {
        b.iter(|| {
            let _ = BlackScholes::price_call(
                black_box(dec!(100)),
                black_box(dec!(100)),
                black_box(dec!(0.05)),
                black_box(dec!(0.2)),
                black_box(1.0),
            );
        });
    });
}

criterion_group!(benches, benchmark_option_pricing);
criterion_main!(benches);
```

**Why `black_box`?**
- Prevents compiler from optimizing away the calculation
- Ensures realistic measurements

Run benchmarks:
```bash
cargo bench
```

## 🎭 Mocking

Create test doubles for dependencies:

```rust
#[cfg(test)]
mod tests {
    // Mock instrument
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
    fn test_with_mock() {
        let mock = MockInstrument {
            price: Money::new(dec!(100), CurrencyCode::USD),
        };
        assert_eq!(mock.price().unwrap().amount(), dec!(100));
    }
}
```

## 🐛 Debugging Tests

### Print Debugging

```rust
#[test]
fn test_debug() {
    let option = EuropeanOption::new(...);
    let greeks = option.greeks().unwrap();
    
    println!("Delta: {}", greeks.delta);
    println!("Gamma: {}", greeks.gamma);
    
    // View the output with --nocapture
    assert!(greeks.delta > 0.0);
}
```

### Panic Messages

```rust
#[test]
#[should_panic(expected = "Volatility cannot be zero")]
fn test_panic_on_zero_vol() {
    // This test passes if it panics with the expected message
    let option = EuropeanOption::new(
        dec!(100), dec!(100), dec!(0.05),
        dec!(0), 1.0, OptionType::Call,
    );
    option.price().unwrap();  // Should panic
}
```

## ✅ Best Practices

### DO:

1. **Test one thing per test**
   ```rust
   // Good
   #[test]
   fn test_addition() { ... }
   
   #[test]
   fn test_subtraction() { ... }
   ```

2. **Use descriptive names**
   ```rust
   // Good
   fn test_money_currency_mismatch_returns_error()
   
   // Bad
   fn test3()
   ```

3. **Test edge cases**
   - Zero values
   - Negative values (if invalid)
   - Maximum/minimum values
   - Boundary conditions

4. **Keep tests independent**
   - No shared state between tests
   - Each test should pass on its own

### DON'T:

1. **Don't test private functions directly**
   - Test through public API
   - If you need to test private functions, reconsider design

2. **Don't ignore test failures**
   ```rust
   // Bad
   let _ = operation();  // Ignoring error!
   
   // Good
   operation().expect("Should succeed");
   ```

3. **Don't rely on test order**
   - Tests can run in any order
   - Don't have Test B depend on Test A

## 🎯 Exercises

1. **Add a regression test**: Find a bug, write a test that fails, fix the bug, verify test passes

2. **Improve coverage**: Add tests for untested code paths

3. **Add property tests**: Test mathematical invariants
   ```rust
   #[test]
   fn test_discount_factor_between_0_and_1() {
       // For positive rates and times, DF should be in (0, 1]
   }
   ```

4. **Benchmark your own code**: Add benchmarks for your custom functions

---

Next: [Rust Patterns](./08-rust-patterns.md)

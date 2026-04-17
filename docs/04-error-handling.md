# Error Handling: The Rust Way

## ❌ The Problem with Exceptions

In many languages, errors are handled with exceptions:

```python
# Python - errors can happen anywhere!
def calculate_option_price(spot, strike, vol):
    if vol <= 0:
        raise ValueError("Volatility must be positive")
    return black_scholes(spot, strike, vol)  # Might also raise!

# Caller might not handle it - crash at runtime!
price = calculate_option_price(100, 100, -0.2)
```

**Problems:**
- Hidden control flow
- Runtime crashes
- Documentation often out of sync

## ✅ Rust's Solution: `Result` Type

Rust makes errors part of the type system:

```rust
pub enum Result<T, E> {
    Ok(T),   // Success case
    Err(E),  // Error case
}
```

**Key insight**: The compiler forces you to handle errors!

## 🎯 Our Error Type

### Using `thiserror`

```rust
// core/error.rs
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Arithmetic error: {0}")]
    Arithmetic(String),
    
    #[error("Currency mismatch: expected {expected}, got {actual}")]
    CurrencyMismatch { expected: String, actual: String },
    
    #[error("Pricing error: {0}")]
    Pricing(String),
}
```

**Why `thiserror`?**
- Generates `Display` implementation automatically
- Derives standard traits
- Clean, declarative syntax

### Convenience Methods

```rust
impl Error {
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        Error::InvalidInput(msg.into())
    }
    
    pub fn currency_mismatch<S: Into<String>>(expected: S, actual: S) -> Self {
        Error::CurrencyMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

// Usage
return Err(Error::invalid_input("Volatility must be positive"));
```

## 🔄 The `?` Operator

The `?` operator makes error propagation clean:

```rust
// Without ? operator (verbose)
fn price_option(&self) -> Result<Money> {
    let d1 = match self.calculate_d1() {
        Ok(d1) => d1,
        Err(e) => return Err(e),
    };
    let d2 = match self.calculate_d2(d1) {
        Ok(d2) => d2,
        Err(e) => return Err(e),
    };
    self.calculate_price(d1, d2)
}

// With ? operator (clean!)
fn price_option(&self) -> Result<Money> {
    let d1 = self.calculate_d1()?;  // Early return on error
    let d2 = self.calculate_d2(d1)?;
    self.calculate_price(d1, d2)
}
```

**How it works:**
- If `Result` is `Ok`, unwraps the value
- If `Result` is `Err`, returns early with the error

## 🛡️ Validation Patterns

### Fail Fast

```rust
impl EuropeanOption {
    pub fn price(&self) -> Result<Money> {
        // Validate inputs first
        if self.volatility.is_zero() {
            return Err(Error::invalid_input("Volatility cannot be zero"));
        }
        if self.time_to_expiry <= 0.0 {
            return Err(Error::invalid_input("Time must be positive"));
        }
        if self.spot <= Decimal::ZERO || self.strike <= Decimal::ZERO {
            return Err(Error::invalid_input("Prices must be positive"));
        }
        
        // Now calculate safely
        let price = self.calculate_price()?;
        Ok(Money::new(price, self.currency()))
    }
}
```

### Type-Driven Validation

Use types to make invalid states unrepresentable:

```rust
// Bad: Can create invalid state
pub struct Option {
    volatility: f64,  // Could be negative!
}

// Better: Newtype pattern
pub struct Volatility(Decimal);

impl Volatility {
    pub fn new(value: Decimal) -> Result<Self> {
        if value <= Decimal::ZERO {
            return Err(Error::invalid_input("Volatility must be positive"));
        }
        Ok(Self(value))
    }
}
```

## 🔀 Converting Errors

### The `From` Trait

```rust
// Automatically convert Decimal errors to our Error type
impl From<rust_decimal::Error> for Error {
    fn from(e: rust_decimal::Error) -> Self {
        Error::Arithmetic(e.to_string())
    }
}

// Now ? works with Decimal operations
fn calculate(&self) -> Result<Decimal> {
    let result = self.a.checked_div(self.b)?;  // Auto-converts!
    Ok(result)
}
```

### Mapping Errors

```rust
// Transform error types
let value = some_operation()
    .map_err(|e| Error::pricing(format!("Failed: {}", e)))?;
```

## 🎨 Error Handling Strategies

### 1. **Early Return** (Most Common)

```rust
fn process_instrument(&self) -> Result<Price> {
    let data = self.fetch_data()?;
    let validated = self.validate(data)?;
    let calculated = self.calculate(validated)?;
    Ok(calculated)
}
```

### 2. **Or Else**

```rust
// Provide default on error
let result = risky_operation()
    .or_else(|_| fallback_operation())?;
```

### 3. **Match for Recovery**

```rust
let price = match instrument.price() {
    Ok(p) => p,
    Err(Error::Pricing(_)) => {
        // Use fallback model
        fallback_model.price(instrument)?
    }
    Err(e) => return Err(e),  // Other errors are fatal
};
```

## 🧪 Testing Errors

```rust
#[test]
fn test_zero_volatility_error() {
    let option = EuropeanOption::new(
        dec!(100), dec!(100), dec!(0.05),
        dec!(0),  // Zero volatility!
        1.0,
        OptionType::Call,
    );
    
    let result = option.price();
    
    // Check we got the right error type
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    
    // Or check the error message
    if let Err(Error::InvalidInput(msg)) = result {
        assert!(msg.contains("Volatility"));
    }
}

#[test]
fn test_currency_mismatch() {
    let usd = Money::new(dec!(100), CurrencyCode::USD);
    let eur = Money::new(dec!(100), CurrencyCode::EUR);
    
    let result = usd.checked_add(&eur);
    
    assert!(matches!(
        result,
        Err(Error::CurrencyMismatch { expected, actual })
        if expected == "USD" && actual == "EUR"
    ));
}
```

## 🚀 Advanced Patterns

### Custom Result Type

```rust
// Type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

// Usage
fn calculate() -> Result<Money> {
    // ...
}
```

### Multiple Error Types

```rust
// When you need different error contexts
pub enum CalculationError {
    Input(Error),
    Numerical(Error),
    Convergence { iterations: usize, tolerance: f64 },
}

impl From<Error> for CalculationError {
    fn from(e: Error) -> Self {
        CalculationError::Input(e)
    }
}
```

### Result Combinators

```rust
// Chain operations
let result = fetch_data()
    .and_then(validate)
    .and_then(calculate)
    .map(|price| price * discount_factor)
    .map_err(|e| log_error(e))?;

// Provide defaults
let price = maybe_price.unwrap_or(default_price);

// Unwrap with custom message (in tests only!)
let price = result.expect("Price calculation should succeed");
```

## 📝 Best Practices

### ✅ DO:

1. **Use the `?` operator** for clean error propagation
2. **Validate early** and fail fast
3. **Provide context** in error messages
4. **Make errors descriptive** but not verbose
5. **Use types to prevent errors** when possible

### ❌ DON'T:

1. **Don't use `unwrap()` in production code**
   ```rust
   // Bad
   let price = option.price().unwrap();  // Panics on error!
   
   // Good
   let price = option.price()?;  // Returns error gracefully
   ```

2. **Don't use `expect()` without good reason**
   ```rust
   // Acceptable in tests
   let value = result.expect("Test data should be valid");
   
   // Never in library code
   let value = result.expect("Should work");  // Bad!
   ```

3. **Don't swallow errors**
   ```rust
   // Bad
   if let Err(_) = operation() {
       return Ok(());  // Error disappeared!
   }
   
   // Good
   operation()?;  // Propagate the error
   ```

## 🎯 Exercises

1. **Add Validation**: Create a `ValidatedOption` type that ensures all parameters are valid at construction

2. **Error Conversion**: Implement `From` for a standard library error type

3. **Error Collection**: Calculate prices for a portfolio, collecting all errors instead of failing on first

```rust
// Exercise 3 template
fn price_portfolio(instruments: &[Box<dyn Pricable>]) -> Vec<Result<Money>> {
    instruments
        .iter()
        .map(|inst| inst.price())
        .collect()
}

// Or accumulate errors
fn price_portfolio_valid(instruments: &[Box<dyn Pricable>]) -> Result<Vec<Money>> {
    instruments
        .iter()
        .map(|inst| inst.price())
        .collect()  // Returns Err if any failed
}
```

---

Next: [Option Pricing](./05-option-pricing.md)

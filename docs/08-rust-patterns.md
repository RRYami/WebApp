# Rust Patterns: Idiomatic Code

This document explains common Rust patterns used throughout the codebase.

## 🎭 Pattern 1: Newtype Pattern

### Problem
Primitive types don't carry semantic meaning:

```rust
// What do these mean?
let x: f64 = 100.5;
let y: String = "USD".to_string();
```

### Solution
Wrap primitives in structs:

```rust
// Currency code with validation
pub struct CurrencyCode([u8; 3]);

impl CurrencyCode {
    pub fn new(code: &str) -> Result<Self> {
        // Validation ensures valid ISO 4217 code
        if code.len() != 3 { ... }
        Ok(Self([code.as_bytes()[0], code.as_bytes()[1], code.as_bytes()[2]]))
    }
}

// Now types have meaning!
let currency = CurrencyCode::new("USD")?;
```

**Benefits:**
- Type safety at compile time
- Validation at construction
- Can't accidentally mix types

## 🎭 Pattern 2: Builder Pattern

### Problem
Constructors with many parameters are error-prone:

```rust
// Which parameter is which?
let option = EuropeanOption::new(
    100.0, 105.0, 0.05, 0.2, 1.0, OptionType::Call
);
```

### Solution
Use the type system to guide construction:

```rust
// With clear parameter names
let option = EuropeanOption::new(
    dec!(100),   // strike
    dec!(105),   // spot
    dec!(0.05),  // risk_free_rate
    dec!(0.2),   // volatility
    1.0,         // time_to_expiry
    OptionType::Call,
);

// Or use builder methods for optional params
let option = EuropeanOption::builder()
    .strike(dec!(100))
    .spot(dec!(105))
    .build()?;
```

**In our codebase:** We use named parameters with the `dec!` macro for clarity.

## 🎭 Pattern 3: Type State Pattern

### Problem
Invalid states can be constructed:

```rust
// Bad: Can create invalid option
let option = EuropeanOption {
    strike: dec!(-100),  // Negative strike!
    volatility: dec!(0), // Zero vol!
    ...
};
```

### Solution
Make invalid states unrepresentable:

```rust
impl EuropeanOption {
    pub fn new(
        strike: Decimal,
        spot: Decimal,
        ...
    ) -> Self {
        // Validate at construction
        assert!(strike > Decimal::ZERO, "Strike must be positive");
        assert!(volatility > Decimal::ZERO, "Volatility must be positive");
        assert!(time_to_expiry > 0.0, "Time must be positive");
        
        Self { strike, spot, ... }
    }
}
```

**Benefit:** If it compiles, it's likely valid!

## 🎭 Pattern 4: Result Type for Error Handling

### Problem
Exceptions are implicit and hard to track:

```python
# Python - might raise, who knows?
def calculate(x, y):
    return x / y  # Could raise ZeroDivisionError
```

### Solution
Explicit error types:

```rust
pub fn discount_factor(&self, time: Decimal) -> Result<Decimal> {
    if time < Decimal::ZERO {
        return Err(Error::invalid_input("Time cannot be negative"));
    }
    
    let df = match self.compounding {
        Compounding::Continuous => (-self.rate * time).exp(),
        ...
    };
    
    Ok(df)
}

// Caller must handle the error
let df = rate.discount_factor(dec!(1.0))?;  // Propagates error
// or
let df = rate.discount_factor(dec!(1.0))
    .expect("Valid time");  // Panics on error
// or
match rate.discount_factor(dec!(1.0)) {
    Ok(df) => println!("DF: {}", df),
    Err(e) => println!("Error: {}", e),
}
```

**Benefits:**
- Compiler forces error handling
- Errors are part of the type signature
- No hidden control flow

## 🎭 Pattern 5: Trait Bounds

### Problem
Functions should work with multiple types:

```rust
// This only works with EuropeanOption
fn print_price(option: &EuropeanOption) {
    println!("{}", option.price());
}
```

### Solution
Use trait bounds:

```rust
// Works with any Pricable type
fn print_price<P: Pricable>(instrument: &P) {
    match instrument.price() {
        Ok(price) => println!("Price: {}", price),
        Err(e) => println!("Error: {}", e),
    }
}

// Or multiple bounds
fn analyze<I>(instrument: &I) -> Result<()>
where
    I: Instrument + Pricable + HasGreeks,
{
    println!("Type: {}", instrument.instrument_type());
    println!("Price: {}", instrument.price()?);
    println!("Delta: {}", instrument.delta()?);
    Ok(())
}
```

**Benefit:** Write generic code that works with any compatible type.

## 🎭 Pattern 6: Iterator Pattern

### Problem
Manual iteration is verbose:

```rust
let mut sum = Decimal::ZERO;
for i in 0..cash_flows.len() {
    let cf = &cash_flows[i];
    sum += cf.amount();
}
```

### Solution
Use iterators:

```rust
// Sum all cash flows
let total: Decimal = cash_flows
    .iter()
    .map(|(_, amount)| amount.amount())
    .sum();

// Filter and map
let future_flows: Vec<_> = cash_flows
    .iter()
    .filter(|(date, _)| *date > pricing_date)
    .map(|(date, amount)| (date, amount))
    .collect();

// Fold for complex accumulation
let pv = cash_flows
    .iter()
    .filter(|(date, _)| *date > pricing_date)
    .try_fold(Decimal::ZERO, |acc, (date, amount)| {
        let time = day_count.year_fraction(pricing_date, *date);
        let df = rate.discount_factor(time)?;
        Ok(acc + amount.amount() * df)
    })?;
```

**Benefits:**
- More declarative
- Often faster (iterator optimizations)
- Chainable operations

## 🎭 Pattern 7: Deref and AsRef

### Problem
Converting between types is verbose:

```rust
let code = CurrencyCode::new("USD").unwrap();
let s = code.as_str();  // Need a method to access inner value
```

### Solution
Implement `AsRef`:

```rust
impl AsRef<str> for CurrencyCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// Now works with anything accepting &str
fn print_currency<C: AsRef<str>>(code: C) {
    println!("Currency: {}", code.as_ref());
}

print_currency(CurrencyCode::USD);
print_currency("EUR");
```

## 🎭 Pattern 8: From and Into

### Problem
Converting types requires explicit functions:

```rust
let decimal = Decimal::from_f64(0.5).unwrap();
```

### Solution
Implement `From` trait:

```rust
// Automatic conversion from i32
let d: Decimal = 42.into();

// From string
let d = Decimal::from_str("42.5").unwrap();

// Custom conversion
impl From<Money> for Decimal {
    fn from(money: Money) -> Self {
        money.amount()
    }
}

let amount: Decimal = money.into();
```

**Note:** `From` automatically provides `Into`.

## 🎭 Pattern 9: Default Trait

### Problem
Creating default values is repetitive:

```rust
let config = PricingConfig {
    tolerance: 1e-10,
    max_iterations: 100,
    parallel: false,
};
```

### Solution
Implement `Default`:

```rust
impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            tolerance: 1e-10,
            max_iterations: 100,
            parallel: false,
        }
    }
}

// Now use default values
let config = PricingConfig::default();

// Or override specific fields
let config = PricingConfig {
    parallel: true,
    ..PricingConfig::default()
};
```

## 🎭 Pattern 10: Debug and Display

### Problem
Printing structs is painful:

```rust
println!("Option: strike={}, spot={}, ...", option.strike, option.spot);
```

### Solution
Derive or implement formatting traits:

```rust
// For debugging (auto-generated)
#[derive(Debug)]
pub struct EuropeanOption { ... }

println!("{:?}", option);  // Debug format
println!("{:#?}", option); // Pretty debug format

// For user display (custom implementation)
impl fmt::Display for EuropeanOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "European {} Option: strike={}, spot={}, T={:.2}y",
            self.option_type,
            self.strike,
            self.spot,
            self.time_to_expiry
        )
    }
}

println!("{}", option);  // User-friendly format
```

## 🎯 Applying These Patterns

Here's how they work together:

```rust
// 1. Newtype for type safety
let currency = CurrencyCode::new("USD")?;

// 2. Type state - validated at construction
let option = EuropeanOption::new(strike, spot, rate, vol, time, opt_type);

// 3. Result for error handling
let price = option.price()?;

// 4. Trait bounds for flexibility
fn print_analysis<I: Instrument + Pricable>(inst: &I) { ... }

// 5. Iterator for collections
let total: Decimal = portfolio.iter().map(|p| p.notional().amount()).sum();

// 6. Default for configuration
let config = PricingConfig::default();

// 7. Display for output
println!("{}", option);
```

## 🎓 Learning Resources

- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [Idiomatic Rust](https://github.com/mre/idiomatic-rust)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

---

Next: [Practice Exercises](./09-exercises.md)

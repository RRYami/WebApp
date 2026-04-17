# Core Concepts: Money, Currency, and Precision

## 💰 Why Exact Precision Matters

### The Floating-Point Problem

```rust
// This is surprising!
let a: f64 = 0.1;
let b: f64 = 0.2;
println!("{}", a + b == 0.3);  // false!
println!("{:.17}", a + b);       // 0.30000000000000004
```

**Why this happens:**
- Computers use binary (base-2) representation
- 0.1 in decimal is an infinite repeating fraction in binary
- f64 approximates 0.1, introducing tiny errors

### Financial Impact

```rust
// In financial calculations, these errors compound!
let price: f64 = 100.10;
let quantity: f64 = 1000000.0;
let total = price * quantity;
// Expected: 100,100,000
// Actual:   100,099,999.99999999
```

## 🎯 The Solution: Decimal

### What is Decimal?

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Exact representation!
let a = dec!(0.1);
let b = dec!(0.2);
println!("{}", a + b == dec!(0.3));  // true!
```

**How it works:**
- Stores numbers as integers with a scale (decimal places)
- 0.1 is stored as `1` with scale `1` (meaning 1 × 10⁻¹)
- All operations are integer arithmetic

### Creating Decimals

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Method 1: Macro (compile-time validation)
let d1 = dec!(123.456);

// Method 2: From string (runtime parsing)
let d2 = Decimal::from_str("123.456").unwrap();

// Method 3: From integers
let d3 = Decimal::new(123456, 3);  // 123.456

// Method 4: From f64 (careful!)
let d4 = Decimal::from_f64(0.1).unwrap();  // May lose precision
```

## 💵 The Money Type

### Definition

```rust
// core/money.rs
pub struct Money {
    amount: Decimal,
    currency: CurrencyCode,
}
```

**Why not just use Decimal?**
- Prevents adding different currencies
- Attaches meaning to the number
- Enables currency-specific formatting

### Using Money

```rust
use pricing_lib::prelude::*;

// Create money values
let usd = Money::new(dec!(100.50), CurrencyCode::USD);
let eur = Money::new(dec!(85.00), CurrencyCode::EUR);

// Same currency: OK!
let usd2 = Money::new(dec!(50.00), CurrencyCode::USD);
let sum = usd.checked_add(&usd2)?;  // Returns Ok(Money)

// Different currencies: Error!
let bad = usd.checked_add(&eur)?;  // Returns Err!
```

### Arithmetic Operations

```rust
impl Money {
    // Safe addition with currency checking
    pub fn checked_add(&self, other: &Self) -> Result<Self> {
        if self.currency != other.currency {
            return Err(Error::currency_mismatch(
                self.currency.as_str(),
                other.currency.as_str(),
            ));
        }
        Ok(Self {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }

    // Scalar multiplication (no currency check needed)
    pub fn mul_scalar(&self, scalar: Decimal) -> Self {
        Self {
            amount: self.amount * scalar,
            currency: self.currency,
        }
    }
}
```

**Key pattern**: `checked_` prefix indicates operations that can fail.

## 🌍 Currency Codes

### ISO 4217 Standard

```rust
// core/currency.rs
pub struct CurrencyCode([u8; 3]);

impl CurrencyCode {
    pub fn new(code: &str) -> Result<Self> {
        // Validation ensures:
        // - Exactly 3 characters
        // - All uppercase letters
        // - No numbers or special chars
        if code.len() != 3 {
            return Err(Error::invalid_input("Must be 3 chars"));
        }
        if !code.chars().all(|c| c.is_ascii_alphabetic() && c.is_ascii_uppercase()) {
            return Err(Error::invalid_input("Must be uppercase letters"));
        }
        // ...
    }
}
```

### Compile-Time Constants

```rust
impl CurrencyCode {
    /// Predefined USD currency code
    pub const USD: Self = CurrencyCode([b'U', b'S', b'D']);
    pub const EUR: Self = CurrencyCode([b'E', b'U', b'R']);
    // ...
}

// Usage
let usd = CurrencyCode::USD;  // No runtime cost!
```

**Why constants?**
- Zero runtime cost
- No validation needed
- Type safety

## 📈 Interest Rates

### Compounding Methods

```rust
pub enum Compounding {
    Simple,                    // No compounding: r × t
    Compounded(u32),          // k times per year: (1 + r/k)^(kt)
    Continuous,               // e^(r×t)
    SimpleThenCompounded,     // Hybrid approach
}
```

### Discount Factors

The fundamental building block of pricing:

```rust
impl InterestRate {
    /// Calculate discount factor: how much is $1 in the future worth today?
    pub fn discount_factor(&self, time: Decimal) -> Result<Decimal> {
        match self.compounding {
            Compounding::Simple => {
                Ok(Decimal::ONE / (Decimal::ONE + self.rate * time))
            }
            Compounding::Continuous => {
                Ok((-self.rate * time).exp())
            }
            Compounding::Compounded(k) => {
                let base = Decimal::ONE + self.rate / Decimal::from(k);
                Ok(base.powd(-Decimal::from(k) * time))
            }
            // ...
        }
    }
}
```

**Financial meaning**: A discount factor of 0.95 means $1 received in the future is worth $0.95 today.

### Converting Between Compounding Methods

```rust
// 5% annual can be expressed in different ways:
let annual = InterestRate::annual(dec!(0.05));      // 5% compounded annually
let continuous = annual.to_compounding(Compounding::Continuous, dec!(1))?;
// Continuous rate ≈ 4.88%

// They're equivalent! Same discount factor for the same time period.
let df1 = annual.discount_factor(dec!(1))?;
let df2 = continuous.discount_factor(dec!(1))?;
assert!(df1 == df2);
```

## 📅 Day Count Conventions

Finance uses different ways to count days between dates:

```rust
pub enum DayCountConvention {
    Act360,        // Actual days / 360 (money markets)
    Act365Fixed,   // Actual days / 365 (sterling markets)
    ActAct,        // Actual days / actual days in year (bonds)
    Thirty360,     // 30-day months / 360 (corporate bonds)
}
```

### Example

```rust
let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
let end = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();

// ACT/360: 182 / 360 = 0.5056 years
let yf_360 = DayCountConvention::Act360.year_fraction(start, end);

// ACT/365: 182 / 365 = 0.4986 years
let yf_365 = DayCountConvention::Act365Fixed.year_fraction(start, end);
```

**Why different conventions?**
- Historical reasons
- Market conventions vary by instrument type
- Small differences matter in large trades

## 🧪 Testing Precision

```rust
#[test]
fn test_money_precision() {
    let m1 = Money::new(dec!(0.1), CurrencyCode::USD);
    let m2 = Money::new(dec!(0.2), CurrencyCode::USD);
    let sum = m1.checked_add(&m2).unwrap();
    
    // Exact comparison works with Decimal!
    assert_eq!(sum.amount(), dec!(0.3));
}
```

## 🎯 Exercises

1. **Create a Currency Converter**
   ```rust
   // Implement a function to convert Money between currencies
   fn convert(money: Money, target: CurrencyCode, rate: Decimal) -> Money
   ```

2. **Add a Fee Calculator**
   ```rust
   // Calculate trading fees as a percentage of notional
   fn calculate_fee(trade_value: Money, fee_bps: Decimal) -> Money
   // 1 bps (basis point) = 0.01% = 0.0001
   ```

3. **Implement Compound Interest**
   ```rust
   // Calculate final amount after compound interest
   fn compound_interest(principal: Money, rate: InterestRate, years: Decimal) -> Money
   ```

---

Next: [Traits in Action](./03-traits.md)

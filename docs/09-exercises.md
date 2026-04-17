# Practice Exercises

Now it's time to apply what you've learned! These exercises progress from beginner to advanced.

## 🟢 Beginner Exercises

### Exercise 1: Add a New Currency

**Goal:** Add Swiss Franc (CHF) support

**Steps:**
1. Open `src/core/currency.rs`
2. Add `CHF` constant similar to `USD`
3. Add `chf()` method to `Currency` struct
4. Test it works

**Solution template:**
```rust
// In CurrencyCode impl
pub const CHF: Self = CurrencyCode([b'C', b'H', b'F']);

// In Currency impl
pub fn chf() -> Self {
    Self::new(CurrencyCode::CHF, "Swiss Franc", "Fr", 2)
}
```

**Test:**
```rust
#[test]
fn test_chf_currency() {
    let chf = CurrencyCode::CHF;
    assert_eq!(chf.as_str(), "CHF");
}
```

---

### Exercise 2: Money Comparison

**Goal:** Implement comparison operators for Money

**Task:** Add methods to compare Money values:
```rust
impl Money {
    pub fn is_greater_than(&self, other: &Self) -> Result<bool> {
        // Check currencies match, then compare amounts
    }
    
    pub fn is_less_than(&self, other: &Self) -> Result<bool> {
        // Similar to above
    }
}
```

**Hint:** Remember to check currencies match first!

---

### Exercise 3: Simple Interest Calculator

**Goal:** Create a function to calculate simple interest

**Task:** Implement in `src/utils/mod.rs`:
```rust
/// Calculate simple interest
/// Formula: I = P × r × t
pub fn simple_interest(principal: Money, rate: Decimal, years: Decimal) -> Result<Money> {
    // Your implementation
}
```

**Test:**
```rust
#[test]
fn test_simple_interest() {
    let principal = Money::new(dec!(1000), CurrencyCode::USD);
    let interest = simple_interest(principal, dec!(0.05), dec!(2)).unwrap();
    assert_eq!(interest.amount(), dec!(100));  // 1000 * 0.05 * 2 = 100
}
```

## 🟡 Intermediate Exercises

### Exercise 4: Create a Forward Contract

**Goal:** Implement a new financial instrument

**Task:** Create `src/instruments/forward.rs`:

```rust
//! Forward contract implementation

use crate::prelude::*;
use chrono::NaiveDate;

/// A forward contract - agreement to buy/sell at future date
pub struct Forward {
    underlying_spot: Decimal,
    strike: Decimal,
    risk_free_rate: Decimal,
    maturity: NaiveDate,
    currency: CurrencyCode,
}

impl Forward {
    pub fn new(
        underlying_spot: Decimal,
        strike: Decimal,
        risk_free_rate: Decimal,
        maturity: NaiveDate,
        currency: CurrencyCode,
    ) -> Self {
        Self { ... }
    }
    
    /// Forward price = S × e^(rT)
    pub fn fair_value(&self, pricing_date: NaiveDate) -> Result<Money> {
        // Calculate time to maturity
        // Calculate forward price
    }
    
    /// Current value = S - K × e^(-rT)
    pub fn current_value(&self, pricing_date: NaiveDate) -> Result<Money> {
        // Similar to fair value but discounted
    }
}

impl Instrument for Forward { ... }
impl Pricable for Forward { ... }
```

**Mathematical reference:**
- Forward price = Spot × e^(r×T)
- Forward value = Spot - Strike × e^(-r×T)

---

### Exercise 5: Portfolio Value

**Goal:** Calculate total value of a portfolio

**Task:** Create a `Portfolio` struct:

```rust
pub struct Position {
    instrument: Box<dyn Pricable>,
    quantity: Decimal,
}

pub struct Portfolio {
    positions: Vec<Position>,
}

impl Portfolio {
    pub fn new() -> Self {
        Self { positions: Vec::new() }
    }
    
    pub fn add_position(&mut self, instrument: Box<dyn Pricable>, quantity: Decimal) {
        self.positions.push(Position { instrument, quantity });
    }
    
    /// Calculate total portfolio value
    pub fn total_value(&self) -> Result<Money> {
        // Sum (price × quantity) for all positions
        // Handle currency conversions if needed
    }
}
```

**Challenge:** Handle different currencies in the same portfolio!

---

### Exercise 6: Greeks Aggregator

**Goal:** Calculate portfolio-level Greeks

**Task:** Extend the Portfolio to aggregate Greeks:

```rust
impl Portfolio {
    /// Calculate weighted average delta
    pub fn portfolio_delta(&self) -> Result<f64> {
        // For each option position:
        // - Get delta
        // - Multiply by quantity
        // - Sum and divide by total notional
    }
    
    /// Calculate portfolio gamma
    pub fn portfolio_gamma(&self) -> Result<f64> {
        // Similar to delta
    }
}
```

---

## 🔴 Advanced Exercises

### Exercise 7: Binomial Tree Option Pricing

**Goal:** Implement American option pricing using binomial trees

**Task:** Create `src/pricing/binomial.rs`:

```rust
//! Binomial tree option pricing

pub struct BinomialTree {
    steps: usize,  // Number of time steps
}

impl BinomialTree {
    pub fn new(steps: usize) -> Self {
        Self { steps }
    }
    
    /// Price American option using binomial tree
    pub fn price_american(
        &self,
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Decimal> {
        // 1. Calculate tree parameters
        let dt = time / self.steps as f64;
        let u = (volatility * Decimal::from(dt).sqrt()).exp();  // Up factor
        let d = Decimal::ONE / u;  // Down factor
        let p = (rate.exp() - d) / (u - d);  // Risk-neutral probability
        
        // 2. Build price tree
        // 3. Calculate option values at each node
        // 4. Work backwards from leaves to root
        // 5. Return root value
    }
}

impl PricingEngine for BinomialTree {
    fn price<I: Instrument>(&self, instrument: &I) -> Result<Money> {
        // Implementation
    }
}
```

**Algorithm outline:**
1. Build stock price tree (up/down movements)
2. Calculate option values at maturity (leaf nodes)
3. Work backwards:
   - At each node: value = max(exercise value, discounted expected value)
   - Exercise value = intrinsic value if American
   - Expected value = p×V_up + (1-p)×V_down

---

### Exercise 8: Yield Curve Bootstrap

**Goal:** Build a yield curve from market prices

**Task:** Create `src/curve/bootstrap.rs`:

```rust
//! Yield curve bootstrapping from market instruments

pub struct YieldCurve {
    tenors: Vec<Decimal>,  // Time in years
    rates: Vec<InterestRate>,  // Spot rates at each tenor
}

impl YieldCurve {
    /// Bootstrap yield curve from zero-coupon bonds
    pub fn from_zero_coupon_bonds(
        bonds: &[ZeroCouponBond],
        prices: &[Money],
    ) -> Result<Self> {
        // For each bond:
        // 1. Calculate yield from price
        // 2. Convert to spot rate
        // 3. Store in curve
    }
    
    /// Interpolate rate at any tenor
    pub fn rate_at(&self, tenor: Decimal) -> InterestRate {
        // Use linear interpolation between known points
    }
    
    /// Discount factor at any tenor
    pub fn discount_factor(&self, tenor: Decimal) -> Decimal {
        // DF = exp(-r×T) for continuous rates
    }
}
```

**Mathematical steps:**
1. Price = Face × DF
2. DF = Price / Face
3. For continuous rate: r = -ln(DF) / T

---

### Exercise 9: Parallel Processing

**Goal:** Speed up portfolio calculations using parallelism

**Task:** Use `rayon` crate for parallel iteration:

```rust
use rayon::prelude::*;

impl Portfolio {
    /// Calculate all position values in parallel
    pub fn calculate_values_parallel(&self) -> Vec<Result<Money>> {
        self.positions
            .par_iter()  // Parallel iterator
            .map(|pos| {
                let price = pos.instrument.price()?;
                Ok(price * pos.quantity)
            })
            .collect()
    }
    
    /// Parallel Greeks calculation
    pub fn calculate_greeks_parallel(&self) -> Result<PortfolioGreeks> {
        let greeks: Vec<_> = self.positions
            .par_iter()
            .filter_map(|pos| {
                pos.instrument
                    .as_any()
                    .downcast_ref::<dyn HasGreeks>()
                    .map(|opt| (opt.greeks().ok()?, pos.quantity))
            })
            .collect();
        
        // Aggregate results
        Ok(greeks.iter().fold(PortfolioGreeks::new(), |acc, (g, qty)| {
            acc.add_position(g, qty.to_f64().unwrap())
        }))
    }
}
```

**Add to Cargo.toml:**
```toml
[dependencies]
rayon = "1.8"
```

---

### Exercise 10: Monte Carlo Simulation

**Goal:** Price options using Monte Carlo simulation

**Task:** Create `src/pricing/monte_carlo.rs`:

```rust
//! Monte Carlo option pricing

use rand::prelude::*;
use rand_distr::{Distribution, StandardNormal};

pub struct MonteCarlo {
    simulations: usize,
    rng: ThreadRng,
}

impl MonteCarlo {
    pub fn new(simulations: usize) -> Self {
        Self {
            simulations,
            rng: thread_rng(),
        }
    }
    
    /// Price European option using Monte Carlo
    pub fn price_european(
        &mut self,
        spot: Decimal,
        strike: Decimal,
        rate: Decimal,
        volatility: Decimal,
        time: f64,
        option_type: OptionType,
    ) -> Result<Decimal> {
        let mut sum_payoffs = 0.0;
        
        let spot_f = spot.to_f64().unwrap();
        let strike_f = strike.to_f64().unwrap();
        let rate_f = rate.to_f64().unwrap();
        let vol_f = volatility.to_f64().unwrap();
        
        let normal = StandardNormal;
        
        for _ in 0..self.simulations {
            // Generate random normal variable
            let z: f64 = normal.sample(&mut self.rng);
            
            // Simulate stock price at maturity
            // S_T = S_0 × exp((r - 0.5×σ²)×T + σ×√T×Z)
            let drift = (rate_f - 0.5 * vol_f * vol_f) * time;
            let diffusion = vol_f * time.sqrt() * z;
            let spot_at_maturity = spot_f * (drift + diffusion).exp();
            
            // Calculate payoff
            let payoff = match option_type {
                OptionType::Call => (spot_at_maturity - strike_f).max(0.0),
                OptionType::Put => (strike_f - spot_at_maturity).max(0.0),
            };
            
            sum_payoffs += payoff;
        }
        
        // Discount average payoff
        let avg_payoff = sum_payoffs / self.simulations as f64;
        let price = avg_payoff * (-rate_f * time).exp();
        
        Decimal::try_from(price)
            .map_err(|_| Error::arithmetic("Failed to convert MC result"))
    }
}
```

**Add to Cargo.toml:**
```toml
[dependencies]
rand = "0.8"
rand_distr = "0.4"
```

**Compare results:**
```rust
#[test]
fn test_mc_vs_black_scholes() {
    let spot = dec!(100);
    let strike = dec!(100);
    let rate = dec!(0.05);
    let vol = dec!(0.2);
    let time = 1.0;
    
    // Black-Scholes price
    let bs_price = BlackScholes::price_call(spot, strike, rate, vol, time).unwrap();
    
    // Monte Carlo price
    let mut mc = MonteCarlo::new(100_000);
    let mc_price = mc.price_european(spot, strike, rate, vol, time, OptionType::Call).unwrap();
    
    // Should be close (within 1%)
    let diff = (bs_price - mc_price).abs() / bs_price;
    assert!(diff < dec!(0.01));
}
```

## 🏆 Challenge Projects

### Challenge 1: Complete Trading System

Build a command-line tool that:
1. Reads a portfolio from a CSV file
2. Prices all instruments
3. Calculates total value and risk metrics
4. Outputs a report

**CSV format:**
```csv
type,spot,strike,rate,volatility,time,quantity
option,100,100,0.05,0.2,1.0,10
bond,1000,0.06,2024-01-01,2029-01-01,2,5
```

### Challenge 2: Real-Time Risk Dashboard

Create a web dashboard using `actix-web` that:
1. Accepts portfolio uploads
2. Calculates real-time Greeks
3. Shows P&L scenarios
4. Displays risk metrics

### Challenge 3: Options Strategy Analyzer

Implement common option strategies:
- Covered call
- Protective put
- Straddle
- Butterfly spread
- Iron condor

Calculate break-even points, max profit, max loss for each.

## 📚 Hints and Tips

### Debugging Tips

1. **Use `dbg!` macro:**
   ```rust
   let price = dbg!(option.price()?);
   ```

2. **Print intermediate values:**
   ```rust
   println!("d1 = {}, d2 = {}", d1, d2);
   ```

3. **Use tests to explore:**
   ```rust
   #[test]
   fn explore() {
       let option = EuropeanOption::new(...);
       let greeks = option.greeks().unwrap();
       println!("{:?}", greeks);
   }
   ```

### Performance Tips

1. **Profile first:**
   ```bash
   cargo bench
   ```

2. **Use release mode:**
   ```bash
   cargo run --release
   ```

3. **Check for allocations:**
   ```rust
   // Avoid Vec in hot paths
   let mut vec = Vec::new();  // Slow
   
   // Use arrays when size is known
   let arr = [0.0; 100];  // Fast
   ```

## ✅ Submission Checklist

For each exercise:
- [ ] Code compiles without warnings
- [ ] Tests pass
- [ ] Documentation added
- [ ] Example usage provided
- [ ] Edge cases handled

## 🎓 Learning Path

1. **Start with:** Exercises 1-3 (familiarize with codebase)
2. **Progress to:** Exercises 4-6 (build new features)
3. **Challenge yourself:** Exercises 7-10 (advanced topics)
4. **Show mastery:** Challenge projects

Good luck, and have fun learning Rust! 🦀

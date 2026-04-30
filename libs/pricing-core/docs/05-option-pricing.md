# Option Pricing: Black-Scholes Model

## 📊 The Black-Scholes Formula

The Black-Scholes model prices European options:

**Call Option:**
```
C = S × N(d₁) - K × e^(-rT) × N(d₂)
```

**Put Option:**
```
P = K × e^(-rT) × N(-d₂) - S × N(-d₁)
```

Where:
```
d₁ = [ln(S/K) + (r + σ²/2)T] / (σ√T)
d₂ = d₁ - σ√T
```

Variables:
- S = Spot price
- K = Strike price
- r = Risk-free rate
- σ = Volatility
- T = Time to expiry
- N(x) = Cumulative normal distribution

## 🧮 Implementation Walkthrough

### Step 1: The Normal Distribution

```rust
// pricing/black_scholes.rs

/// Standard normal CDF using Hart approximation
pub fn ndf(x: f64) -> f64 {
    if x < -10.0 { return 0.0; }
    if x > 10.0 { return 1.0; }
    
    // Hart approximation constants
    let b1 = 0.319381530;
    let b2 = -0.356563782;
    let b3 = 1.781477937;
    let b4 = -1.821255978;
    let b5 = 1.330274429;
    let p = 0.2316419;
    let c = 0.39894228;  // 1/sqrt(2*PI)
    
    let ax = x.abs();
    let t = 1.0 / (1.0 + p * ax);
    
    let phi = c * (-ax * ax / 2.0).exp();
    let poly = b1*t + b2*t*t + b3*t*t*t + b4*t*t*t*t + b5*t*t*t*t*t;
    
    let result = 1.0 - phi * poly;
    
    if x >= 0.0 { result } else { 1.0 - result }
}
```

**Why Hart approximation?**
- Fast: No integration required
- Accurate: Error < 7.5×10⁻⁸
- Industry standard

### Step 2: Calculate d₁ and d₂

```rust
pub fn d1(
    spot: Decimal,
    strike: Decimal,
    rate: Decimal,
    volatility: Decimal,
    time: f64,
) -> Result<f64> {
    // Validation
    if volatility.is_zero() {
        return Err(Error::invalid_input("Volatility cannot be zero"));
    }
    
    // Convert to f64 for calculation
    let spot_f = spot.to_f64().ok_or_else(|| Error::arithmetic("Invalid spot"))?;
    let strike_f = strike.to_f64().ok_or_else(|| Error::arithmetic("Invalid strike"))?;
    let rate_f = rate.to_f64().ok_or_else(|| Error::arithmetic("Invalid rate"))?;
    let vol_f = volatility.to_f64().ok_or_else(|| Error::arithmetic("Invalid vol"))?;
    
    // Black-Scholes formula
    let ln_sk = (spot_f / strike_f).ln();
    let numerator = ln_sk + (rate_f + vol_f * vol_f / 2.0) * time;
    let denominator = vol_f * time.sqrt();
    
    Ok(numerator / denominator)
}

pub fn d2(d1: f64, volatility: Decimal, time: f64) -> Result<f64> {
    let vol_f = volatility.to_f64().unwrap();
    Ok(d1 - vol_f * time.sqrt())
}
```

**Why convert to f64?**
- Mathematical functions (ln, exp, sqrt) are native to f64
- Decimal has these operations but they're slower
- Minimal precision loss for this use case

### Step 3: Price Calculation

```rust
pub fn price_call(
    spot: Decimal,
    strike: Decimal,
    rate: Decimal,
    volatility: Decimal,
    time: f64,
) -> Result<Decimal> {
    let d1_val = Self::d1(spot, strike, rate, volatility, time)?;
    let d2_val = Self::d2(d1_val, volatility, time)?;
    
    let nd1 = ndf(d1_val);
    let nd2 = ndf(d2_val);
    
    // Convert inputs
    let spot_f = spot.to_f64().unwrap();
    let strike_f = strike.to_f64().unwrap();
    let rate_f = rate.to_f64().unwrap();
    
    // C = S*N(d1) - K*e^(-rT)*N(d2)
    let price = spot_f * nd1 - strike_f * (-rate_f * time).exp() * nd2;
    
    Decimal::try_from(price)
        .map_err(|_| Error::arithmetic("Failed to convert price"))
}
```

### Step 4: The Greeks

Greeks measure sensitivity to various factors:

```rust
pub fn greeks(
    spot: Decimal,
    strike: Decimal,
    rate: Decimal,
    volatility: Decimal,
    time: f64,
    option_type: OptionType,
) -> Result<Greeks> {
    let d1_val = Self::d1(spot, strike, rate, volatility, time)?;
    let d2_val = Self::d2(d1_val, volatility, time)?;
    
    let nd1 = ndf(d1_val);
    let nd2 = ndf(d2_val);
    let n_prime_d1 = npdf(d1_val);
    
    let spot_f = spot.to_f64().unwrap();
    let strike_f = strike.to_f64().unwrap();
    let rate_f = rate.to_f64().unwrap();
    let vol_f = volatility.to_f64().unwrap();
    
    // Delta: Sensitivity to underlying price
    let delta = match option_type {
        OptionType::Call => nd1,
        OptionType::Put => nd1 - 1.0,
    };
    
    // Gamma: Sensitivity of Delta (same for calls and puts)
    let gamma = n_prime_d1 / (spot_f * vol_f * time.sqrt());
    
    // Theta: Time decay
    let term1 = -spot_f * n_prime_d1 * vol_f / (2.0 * time.sqrt());
    let term2 = match option_type {
        OptionType::Call => -rate_f * strike_f * (-rate_f * time).exp() * nd2,
        OptionType::Put => rate_f * strike_f * (-rate_f * time).exp() * ndf(-d2_val),
    };
    let theta = (term1 + term2) / 365.0;  // Daily theta
    
    // Vega: Sensitivity to volatility (same for calls and puts)
    let vega = spot_f * n_prime_d1 * time.sqrt() / 100.0;  // Per 1%
    
    // Rho: Sensitivity to interest rate
    let rho = match option_type {
        OptionType::Call => strike_f * time * (-rate_f * time).exp() * nd2 / 100.0,
        OptionType::Put => -strike_f * time * (-rate_f * time).exp() * ndf(-d2_val) / 100.0,
    };
    
    Ok(Greeks { delta, gamma, theta, vega, rho })
}
```

**What Greeks tell us:**
- **Delta**: How much option price changes when stock moves $1
- **Gamma**: How much Delta changes when stock moves $1
- **Theta**: How much value is lost each day (time decay)
- **Vega**: How much price changes when volatility changes 1%
- **Rho**: How much price changes when rates change 1%

## 🔄 Implied Volatility

Implied volatility is the volatility that makes the model price equal to market price. We find it by solving:

```rust
pub fn implied_volatility(
    market_price: Decimal,
    spot: Decimal,
    strike: Decimal,
    rate: Decimal,
    time: f64,
    option_type: OptionType,
    guess: Option<f64>,
) -> Result<f64> {
    let mut vol = guess.unwrap_or(0.2);
    let target = market_price.to_f64().unwrap();
    let tolerance = 1e-10;
    let max_iterations = 100;
    
    for _ in 0..max_iterations {
        // Calculate price with current vol guess
        let price = Self::price(spot, strike, rate, 
                               Decimal::from_f64(vol).unwrap(), 
                               time, option_type)?;
        let price_f = price.to_f64().unwrap();
        
        let diff = price_f - target;
        if diff.abs() < tolerance {
            return Ok(vol);  // Converged!
        }
        
        // Calculate derivative (Vega)
        let vega = Self::calculate_vega(spot, strike, rate, vol, time)?;
        
        if vega.abs() < 1e-10 {
            return Err(Error::pricing("Vega too small, cannot converge"));
        }
        
        // Newton-Raphson update
        vol -= diff / vega;
        
        if vol < 0.0 || vol > 5.0 {
            return Err(Error::pricing("Implied vol calculation diverged"));
        }
    }
    
    Err(Error::pricing("Did not converge"))
}
```

**Newton-Raphson method:**
1. Start with initial guess
2. Calculate option price and vega (derivative)
3. Update guess: vol_new = vol_old - (price - target) / vega
4. Repeat until converged

## 🧪 Put-Call Parity

A fundamental relationship:

```
C - P = S - K×e^(-rT)
```

**Test it:**
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

## 📈 Performance

From our benchmarks:

| Operation | Time | Throughput |
|-----------|------|------------|
| Price calculation | 322 ns | ~3.1M ops/sec |
| Greeks calculation | 559 ns | ~1.8M ops/sec |
| Implied vol | 2.14 µs | ~467K ops/sec |

**What this means:**
- You can price **3 million options per second**
- Real-time risk calculations are feasible
- Even complex portfolios (10,000 options) calculate in milliseconds

## 🎯 Exercises

1. **Add Delta Hedging**: Calculate shares needed to hedge an option position
   ```rust
   fn delta_hedge_shares(option: &EuropeanOption, quantity: f64) -> f64 {
       let delta = option.delta().unwrap();
       -delta * quantity  // Negative because we hedge
   }
   ```

2. **Implement Straddle Pricing**: A straddle is a call + put at same strike
   ```rust
   fn straddle_price(spot: Decimal, strike: Decimal, ...) -> Decimal {
       let call = BlackScholes::price_call(spot, strike, ...).unwrap();
       let put = BlackScholes::price_put(spot, strike, ...).unwrap();
       call + put
   }
   ```

3. **Monte Carlo Option Pricing**: Implement a simple Monte Carlo pricer
   ```rust
   fn monte_carlo_price(spot: Decimal, strike: Decimal, 
                        vol: Decimal, time: f64, 
                        simulations: usize) -> Decimal {
       let mut sum = 0.0;
       for _ in 0..simulations {
           let path = simulate_path(spot, vol, time);
           let payoff = (path - strike).max(0.0);
           sum += payoff;
       }
       Decimal::from_f64(sum / simulations as f64).unwrap()
   }
   ```

---

Next: [Bond Pricing](./06-bond-pricing.md)

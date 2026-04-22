# Bond Pricing: Fixed Income

## 🏛️ Bond Basics

A bond is a loan where:
- **Issuer** borrows money
- **Investor** lends money
- **Coupon** = periodic interest payments
- **Face Value** = amount repaid at maturity

### Zero-Coupon Bond

Simplest bond - no coupons, just pays face value at maturity:

```
Price = Face Value × e^(-y×T)
```

Where:
- y = yield
- T = time to maturity

### Coupon Bond

Pays periodic coupons plus face value at maturity:

```
Price = Σ(Coupon × e^(-y×tᵢ)) + Face Value × e^(-y×T)
```

## 💻 Implementation

### Zero-Coupon Bond

```rust
// instruments/bond.rs

pub struct ZeroCouponBond {
    face_value: Money,
    maturity_date: NaiveDate,
    issue_date: NaiveDate,
    day_count: DayCountConvention,
}

impl ZeroCouponBond {
    pub fn new(
        face_value: Money,
        issue_date: NaiveDate,
        maturity_date: NaiveDate,
        day_count: DayCountConvention,
    ) -> Result<Self> {
        if maturity_date <= issue_date {
            return Err(Error::invalid_input("Maturity must be after issue date"));
        }
        
        Ok(Self {
            face_value,
            maturity_date,
            issue_date,
            day_count,
        })
    }
    
    pub fn price_with_yield(
        &self,
        yield_rate: &InterestRate,
        pricing_date: NaiveDate,
    ) -> Result<Money> {
        if pricing_date >= self.maturity_date {
            return Ok(Money::zero(self.face_value.currency()));
        }
        
        // Calculate time to maturity
        let time_to_maturity = self
            .day_count
            .year_fraction(pricing_date, self.maturity_date);
        
        // Discount factor: DF = e^(-y×T)
        let df = yield_rate.discount_factor(time_to_maturity)?;
        
        // Price = Face Value × DF
        let price = self.face_value.amount() * df;
        
        Ok(Money::new(price, self.face_value.currency()))
    }
}
```

**Key insight:** Zero-coupon bonds are simple - just one discount factor!

### Coupon Bond

```rust
pub struct CouponBond {
    face_value: Money,
    coupon_rate: Decimal,     // Annual rate
    maturity_date: NaiveDate,
    issue_date: NaiveDate,
    coupon_frequency: u8,     // 1=annual, 2=semi-annual, 4=quarterly
    day_count: DayCountConvention,
}

impl CouponBond {
    pub fn new(
        face_value: Money,
        coupon_rate: Decimal,
        issue_date: NaiveDate,
        maturity_date: NaiveDate,
        coupon_frequency: u8,
        day_count: DayCountConvention,
    ) -> Result<Self> {
        // Validate frequency
        if ![1, 2, 4, 12].contains(&coupon_frequency) {
            return Err(Error::invalid_input(
                "Coupon frequency must be 1, 2, 4, or 12"
            ));
        }
        
        Ok(Self { ... })
    }
    
    /// Calculate coupon payment amount
    pub fn coupon_amount(&self) -> Money {
        let amount = self.face_value.amount() 
            * self.coupon_rate 
            / Decimal::from(self.coupon_frequency);
        Money::new(amount, self.face_value.currency())
    }
}
```

### Pricing Coupon Bonds

```rust
impl CouponBond {
    pub fn price_with_yield(
        &self,
        yield_rate: &InterestRate,
        pricing_date: NaiveDate,
    ) -> Result<Money> {
        if pricing_date >= self.maturity_date {
            return Ok(Money::zero(self.face_value.currency()));
        }
        
        let cash_flows = self.cash_flows();
        let mut pv = Decimal::ZERO;
        
        // Discount each cash flow
        for (date, amount) in cash_flows {
            if date <= pricing_date {
                continue;  // Already paid
            }
            
            let time = self.day_count.year_fraction(pricing_date, date);
            let df = yield_rate.discount_factor(time)?;
            
            pv += amount.amount() * df;
        }
        
        Ok(Money::new(pv, self.face_value.currency()))
    }
    
    /// Generate all future cash flows
    fn generate_coupon_dates(&self) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let months_between = 12 / self.coupon_frequency as i32;
        
        // Work backwards from maturity
        let mut current = self.maturity_date;
        while current > self.issue_date {
            let year = current.year();
            let month = current.month() as i32 - months_between;
            
            let (new_year, new_month) = if month <= 0 {
                (year - 1, (month + 12) as u32)
            } else {
                (year, month as u32)
            };
            
            // Handle month-end dates
            current = NaiveDate::from_ymd_opt(new_year, new_month, current.day())
                .unwrap_or_else(|| {
                    // Last day of month
                    NaiveDate::from_ymd_opt(new_year, new_month + 1, 1)
                        .unwrap()
                        .pred_opt()
                        .unwrap()
                });
            
            if current >= self.issue_date {
                dates.push(current);
            }
        }
        
        dates.reverse();
        dates
    }
}
```

## 📊 Yield to Maturity (YTM)

YTM is the discount rate that makes present value equal to market price. We solve:

```
Market Price = Σ(CFᵢ / (1 + y)^tᵢ)
```

Using Newton-Raphson iteration:

```rust
impl HasYield for CouponBond {
    fn yield_to_maturity(
        &self,
        market_price: Money,
        guess: Option<f64>,
    ) -> Result<f64> {
        let mut y = guess.unwrap_or(self.coupon_rate.to_f64().unwrap_or(0.05));
        let tolerance = 1e-10;
        let max_iterations = 100;
        
        for _ in 0..max_iterations {
            // Create rate with current yield guess
            let rate = InterestRate::new(
                Decimal::try_from(y).map_err(|_| Error::arithmetic("Invalid rate"))?,
                Compounding::Continuous,
                self.day_count,
            );
            
            // Calculate price at this yield
            let price = self.price_with_yield(&rate, self.issue_date)?;
            let error = price.amount() - market_price.amount();
            
            if error.abs() < Decimal::from_f64(tolerance).unwrap() {
                return Ok(y);  // Converged!
            }
            
            // Numerical derivative (DV01 approximation)
            let dy = 1e-7;
            let rate_dy = InterestRate::new(
                Decimal::try_from(y + dy).map_err(|_| Error::arithmetic("Invalid rate"))?,
                Compounding::Continuous,
                self.day_count,
            );
            let price_dy = self.price_with_yield(&rate_dy, self.issue_date)?;
            let derivative = (price_dy.amount() - price.amount()) 
                / Decimal::from_f64(dy).unwrap();
            
            if derivative.abs() < Decimal::from_f64(1e-15).unwrap() {
                return Err(Error::arithmetic("Derivative too small"));
            }
            
            // Newton-Raphson update
            y -= (error / derivative).to_f64().unwrap_or(0.0);
            
            if y < -1.0 || y > 2.0 {
                return Err(Error::pricing("YTM calculation diverged"));
            }
        }
        
        Err(Error::pricing("YTM did not converge"))
    }
}
```

## 📏 Duration

Duration measures bond price sensitivity to yield changes.

### Macaulay Duration

Weighted average time until cash flows:

```rust
impl CouponBond {
    pub fn macaulay_duration(
        &self,
        yield_rate: &InterestRate,
        pricing_date: NaiveDate,
    ) -> Result<Decimal> {
        let price = self.price_with_yield(yield_rate, pricing_date)?;
        
        if price.is_zero() {
            return Err(Error::arithmetic("Price is zero"));
        }
        
        let cash_flows = self.cash_flows();
        let mut weighted_time = Decimal::ZERO;
        
        for (date, amount) in cash_flows {
            if date <= pricing_date {
                continue;
            }
            
            let t = self.day_count.year_fraction(pricing_date, date);
            let df = yield_rate.discount_factor(t)?;
            
            // Weighted by present value
            weighted_time += t * amount.amount() * df;
        }
        
        Ok(weighted_time / price.amount())
    }
}
```

### Modified Duration

Price sensitivity approximation:

```
%ΔPrice ≈ -Modified Duration × ΔYield
```

```rust
pub fn modified_duration(
    &self,
    yield_rate: &InterestRate,
    pricing_date: NaiveDate,
) -> Result<Decimal> {
    let mac_duration = self.macaulay_duration(yield_rate, pricing_date)?;
    
    // For continuous compounding: Modified = Macaulay
    // Otherwise: Modified = Macaulay / (1 + y/k)
    let adjustment = match yield_rate.compounding() {
        Compounding::Continuous => Decimal::ONE,
        Compounding::Compounded(k) => {
            Decimal::ONE + yield_rate.rate() / Decimal::from(k)
        }
        _ => Decimal::ONE + yield_rate.rate(),
    };
    
    Ok(mac_duration / adjustment)
}
```

**Example:**
- Modified duration = 5 years
- Yield increases 1% (0.01)
- Price change ≈ -5 × 0.01 = -5%

## 💰 Accrued Interest

When you buy a bond between coupon dates, you pay accrued interest to the seller:

```rust
impl CouponBond {
    pub fn accrued_interest(&self, settlement_date: NaiveDate) -> Result<Money> {
        let coupon_dates = self.generate_coupon_dates();
        
        // Find previous coupon date
        let prev_coupon = coupon_dates
            .iter()
            .filter(|&&d| d <= settlement_date)
            .last()
            .copied()
            .unwrap_or(self.issue_date);
        
        // Days accrued
        let days_accrued = self
            .day_count
            .day_count(prev_coupon, settlement_date);
        
        // Days in coupon period
        let days_in_period = self
            .day_count
            .day_count(
                prev_coupon,
                self.next_cash_flow_date(prev_coupon)
                    .unwrap_or(self.maturity_date)
            );
        
        if days_in_period == 0 {
            return Ok(Money::zero(self.face_value.currency()));
        }
        
        // Accrued = Coupon × (Days Accrued / Days in Period)
        let accrued = self.coupon_amount().amount()
            * Decimal::from(days_accrued)
            / Decimal::from(days_in_period);
        
        Ok(Money::new(accrued, self.face_value.currency()))
    }
}
```

## 🧪 Testing Bond Calculations

```rust
#[test]
fn test_zero_coupon_pricing() {
    let bond = ZeroCouponBond::new(
        Money::new(dec!(1000), CurrencyCode::USD),
        date(2024, 1, 1),
        date(2025, 1, 1),
        DayCountConvention::Act360,
    ).unwrap();
    
    let rate = InterestRate::continuous(dec!(0.05));
    let price = bond.price_with_yield(&rate, date(2024, 1, 1)).unwrap();
    
    // 1000 × e^(-0.05) ≈ 951.23
    assert!(price.amount() > dec!(950) && price.amount() < dec!(952));
}

#[test]
fn test_coupon_bond_ytm() {
    let bond = CouponBond::new(
        Money::new(dec!(1000), CurrencyCode::USD),
        dec!(0.05),  // 5% coupon
        date(2024, 1, 1),
        date(2029, 1, 1),
        2,  // Semi-annual
        DayCountConvention::Thirty360,
    ).unwrap();
    
    // Price at par should give YTM ≈ coupon rate
    let price = Money::new(dec!(1000), CurrencyCode::USD);
    let ytm = bond.yield_to_maturity(price, None).unwrap();
    
    assert!((ytm - 0.05).abs() < 0.01);
}
```

## 📊 Performance

From benchmarks:

| Operation | Time | Throughput |
|-----------|------|------------|
| Zero-coupon bond | 2.24 µs | ~446K ops/sec |
| Coupon bond (5yr) | 15.3 µs | ~65K ops/sec |

**Why coupon bonds are slower:**
- Must iterate through all cash flows
- Calculate discount factor for each
- More memory access

## 🎯 Exercises

1. **Bootstrap a Yield Curve**: Given bond prices, calculate spot rates
   ```rust
   fn bootstrap_yield_curve(bonds: &[Bond], prices: &[Money]) -> YieldCurve {
       // Solve for spot rates that price all bonds correctly
   }
   ```

2. **Calculate Convexity**: Add convexity calculation to CouponBond
   ```rust
   fn convexity(&self, yield_rate: &InterestRate) -> Result<Decimal> {
       // Second derivative of price with respect to yield
   }
   ```

3. **Bond Portfolio**: Calculate portfolio duration and YTM
   ```rust
   struct BondPortfolio {
       positions: Vec<(Bond, f64)>,  // Bond and quantity
   }
   
   impl BondPortfolio {
       fn weighted_average_duration(&self) -> Decimal { ... }
       fn portfolio_ytm(&self) -> f64 { ... }
   }
   ```

---

Next: [Testing](./07-testing.md)

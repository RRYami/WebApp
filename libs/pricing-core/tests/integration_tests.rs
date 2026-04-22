//! Integration tests for the pricing library

use chrono::NaiveDate;
use pricing_core::prelude::*;

#[test]
fn test_end_to_end_option_pricing() {
    // Create a European call option
    let option = EuropeanOption::new(
        dec!(100),
        dec!(105),
        dec!(0.05),
        dec!(0.25),
        0.5, // 6 months
        OptionType::Call,
    );

    // Price it
    let price = option.price().expect("Should price successfully");
    assert!(price.amount() > dec!(0));

    // Get Greeks
    let greeks = option.greeks().expect("Should calculate Greeks");
    assert!(greeks.delta > 0.0 && greeks.delta < 1.0);
    assert!(greeks.gamma > 0.0);
    assert!(greeks.vega > 0.0);
}

#[test]
fn test_end_to_end_bond_pricing() {
    let bond = CouponBond::new(
        Money::new(dec!(1000), CurrencyCode::USD),
        dec!(0.06),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2029, 1, 1).unwrap(),
        2,
        DayCountConvention::Thirty360,
    )
    .expect("Should create bond");

    let rate = InterestRate::continuous(dec!(0.05));
    let price = bond
        .price_with_yield(&rate, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        .expect("Should price bond");

    // Bond at discount when yield < coupon
    assert!(price.amount() > dec!(1000));
}

#[test]
fn test_money_operations() {
    let usd = CurrencyCode::USD;
    let eur = CurrencyCode::EUR;

    let m1 = Money::new(dec!(100), usd);
    let m2 = Money::new(dec!(50), usd);

    // Same currency addition
    let sum = m1.checked_add(&m2).expect("Should add same currency");
    assert_eq!(sum.amount(), dec!(150));

    // Different currency should fail
    let m3 = Money::new(dec!(100), eur);
    assert!(m1.checked_add(&m3).is_err());

    // Scalar multiplication
    let scaled = m1 * dec!(1.5);
    assert_eq!(scaled.amount(), dec!(150));
}

#[test]
fn test_day_count_conventions() {
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();

    let act360 = DayCountConvention::Act360.year_fraction(start, end);
    let act365 = DayCountConvention::Act365Fixed.year_fraction(start, end);

    // ACT/360 should give a larger year fraction than ACT/365
    assert!(act360 > act365);
}

#[test]
fn test_interest_rate_conversions() {
    let rate = InterestRate::annual(dec!(0.06));

    // Convert to continuous
    let continuous = rate
        .to_compounding(Compounding::Continuous, dec!(1))
        .expect("Should convert");

    // Convert back
    let back_to_annual = continuous
        .to_compounding(Compounding::Compounded(1), dec!(1))
        .expect("Should convert back");

    // Should be approximately equal
    let diff = (rate.rate() - back_to_annual.rate()).abs();
    assert!(diff < dec!(0.0001));
}

#[test]
fn test_put_call_parity() {
    let spot = dec!(100);
    let strike = dec!(100);
    let rate = dec!(0.05);
    let vol = dec!(0.2);
    let time = 1.0;

    let call = EuropeanOption::new(strike, spot, rate, vol, time, OptionType::Call);
    let put = EuropeanOption::new(strike, spot, rate, vol, time, OptionType::Put);

    let call_price = call.price().expect("Should price call").amount();
    let put_price = put.price().expect("Should price put").amount();

    // Put-Call Parity: C - P = S - K * e^(-rT)
    let lhs = call_price - put_price;
    let df = (-rate * Decimal::try_from(time).unwrap()).exp();
    let rhs = spot - strike * df;

    assert!((lhs - rhs).abs() < dec!(0.01));
}

#[test]
fn test_portfolio_greeks() {
    let mut portfolio = PortfolioGreeks::new();

    let option1 = EuropeanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.2),
        1.0,
        OptionType::Call,
    );
    let option2 = EuropeanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.2),
        1.0,
        OptionType::Put,
    );

    let greeks1 = option1.greeks().expect("Should get Greeks");
    let greeks2 = option2.greeks().expect("Should get Greeks");

    // Long call, short put (synthetic long stock)
    portfolio.add_position(&greeks1, 1.0);
    portfolio.add_position(&greeks2, -1.0);

    // Synthetic long should have delta ≈ 1
    assert!((portfolio.net_delta - 1.0).abs() < 0.1);
}

#[test]
fn test_zero_coupon_bond_ytm() {
    let bond = ZeroCouponBond::new(
        Money::new(dec!(1000), CurrencyCode::USD),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        DayCountConvention::Act360,
    )
    .expect("Should create bond");

    let price = Money::new(dec!(951.23), CurrencyCode::USD);
    let ytm = bond
        .yield_to_maturity(price, None)
        .expect("Should calculate YTM");

    // Verify by pricing back
    let rate = InterestRate::continuous(Decimal::try_from(ytm).unwrap());
    let calculated_price = bond
        .price_with_yield(&rate, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        .expect("Should price");

    let price_diff = (calculated_price.amount() - price.amount()).abs();
    assert!(price_diff < dec!(0.1));
}

#[test]
fn test_implied_volatility() {
    let target_vol = 0.25;
    let price = BlackScholes::price_call(
        dec!(100),
        dec!(100),
        dec!(0.05),
        Decimal::from_f64(target_vol).unwrap(),
        1.0,
    )
    .expect("Should price");

    let implied = BlackScholes::implied_volatility(
        price,
        dec!(100),
        dec!(100),
        dec!(0.05),
        1.0,
        OptionType::Call,
        None,
    )
    .expect("Should calculate implied vol");

    assert!((implied - target_vol).abs() < 1e-6);
}

#[test]
fn test_currency_code_validation() {
    // Valid codes
    assert!(CurrencyCode::new("USD").is_ok());
    assert!(CurrencyCode::new("EUR").is_ok());
    assert!(CurrencyCode::new("GBP").is_ok());

    // Invalid codes
    assert!(CurrencyCode::new("US").is_err()); // Too short
    assert!(CurrencyCode::new("USDD").is_err()); // Too long
    assert!(CurrencyCode::new("usd").is_err()); // Lowercase
    assert!(CurrencyCode::new("US1").is_err()); // Contains number
}

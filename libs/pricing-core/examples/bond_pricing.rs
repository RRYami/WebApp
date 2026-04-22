//! Bond pricing example
//!
//! This example demonstrates how to price various types of bonds
//! and calculate their yield metrics.

use chrono::NaiveDate;
use pricing_core::prelude::*;

fn main() -> Result<()> {
    println!("=== Bond Pricing Examples ===\n");

    // Example 1: Zero-coupon bond
    println!("1. Zero-Coupon Bond");
    println!("   Face Value: $1000, Maturity: 1 year, Yield: 5%\n");

    let zero_coupon = ZeroCouponBond::new(
        Money::new(dec!(1000), CurrencyCode::USD),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        DayCountConvention::Act360,
    )?;

    let yield_rate = InterestRate::continuous(dec!(0.05));
    let price =
        zero_coupon.price_with_yield(&yield_rate, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())?;

    println!("   Price: {}", price);

    // Calculate YTM from market price
    let market_price = Money::new(dec!(951.23), CurrencyCode::USD);
    let ytm = zero_coupon.yield_to_maturity(market_price, None)?;
    println!(
        "   YTM from price ${}: {:.4}%",
        market_price.amount(),
        ytm * 100.0
    );
    println!();

    // Example 2: Coupon bond
    println!("2. Coupon Bond");
    println!("   Face Value: $1000, Coupon: 6%, Semi-annual, Maturity: 5 years\n");

    let coupon_bond = CouponBond::new(
        Money::new(dec!(1000), CurrencyCode::USD),
        dec!(0.06),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2029, 1, 1).unwrap(),
        2, // Semi-annual
        DayCountConvention::Thirty360,
    )?;

    let coupon_rate = InterestRate::continuous(dec!(0.05));
    let coupon_price =
        coupon_bond.price_with_yield(&coupon_rate, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())?;

    println!("   Price at 5% yield: {}", coupon_price);

    // Calculate coupon amount
    let coupon_amount = coupon_bond.coupon_amount();
    println!("   Semi-annual coupon: {}", coupon_amount);

    // Calculate duration
    let duration = coupon_bond
        .macaulay_duration(&coupon_rate, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())?;
    println!("   Macaulay Duration: {:.4} years", duration);

    let modified_duration = coupon_bond
        .modified_duration(&coupon_rate, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())?;
    println!("   Modified Duration: {:.4} years", modified_duration);

    // Calculate YTM
    let market_price = Money::new(dec!(1043.29), CurrencyCode::USD);
    let ytm = coupon_bond.yield_to_maturity(market_price, None)?;
    println!(
        "   YTM from price ${}: {:.4}%",
        market_price.amount(),
        ytm * 100.0
    );

    // Calculate accrued interest
    let settlement = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
    let accrued = coupon_bond.accrued_interest(settlement)?;
    println!("   Accrued interest (Apr 15, 2024): {}", accrued);
    println!();

    // Example 3: Interest rate conversions
    println!("3. Interest Rate Conversions");
    let annual_rate = InterestRate::annual(dec!(0.06));
    println!(
        "   Annual rate (6% compounded annually): {} ",
        annual_rate.rate() * dec!(100)
    );

    let continuous_equivalent = annual_rate.to_compounding(Compounding::Continuous, dec!(1))?;
    println!(
        "   Equivalent continuous rate: {:.4}%",
        continuous_equivalent.rate() * dec!(100)
    );

    let simple_equivalent = annual_rate.to_compounding(Compounding::Simple, dec!(1))?;
    println!(
        "   Equivalent simple rate: {:.4}%",
        simple_equivalent.rate() * dec!(100)
    );
    println!();

    // Example 4: Cash flows
    println!("4. Cash Flow Schedule");
    let cash_flows = coupon_bond.cash_flows();
    println!("   Date          | Amount");
    println!("   ------------------------");
    for (date, amount) in cash_flows.iter().take(6) {
        println!("   {} | {}", date, amount);
    }

    Ok(())
}

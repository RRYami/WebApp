//! Option pricing example
//!
//! This example demonstrates Black-Scholes option pricing
//! and Greeks calculation.

use pricing_core::prelude::*;

fn main() -> Result<()> {
    println!("=== Option Pricing Examples ===\n");

    // Example 1: European Call Option
    println!("1. European Call Option");
    println!("   Spot: $100, Strike: $100, Rate: 5%, Vol: 20%, T: 1 year\n");

    let call_option = EuropeanOption::new(
        dec!(100),  // strike
        dec!(100),  // spot
        dec!(0.05), // risk-free rate
        dec!(0.20), // volatility
        1.0,        // time to expiry
        OptionType::Call,
    );

    // Price the option using Black-Scholes
    let price = call_option.price()?;
    println!("   Black-Scholes Price: {}", price);

    // Calculate Greeks
    let greeks = call_option.greeks()?;
    println!("   Greeks:");
    println!("     Delta: {:.6}", greeks.delta);
    println!("     Gamma: {:.6}", greeks.gamma);
    println!("     Theta (daily): {:.6}", greeks.theta);
    println!("     Vega (per 1% vol): {:.6}", greeks.vega);
    println!("     Rho (per 1% rate): {:.6}", greeks.rho);
    println!();

    // Example 2: European Put Option
    println!("2. European Put Option");
    println!("   Spot: $100, Strike: $100, Rate: 5%, Vol: 20%, T: 1 year\n");

    let put_option = EuropeanOption::new(
        dec!(100),  // strike
        dec!(100),  // spot
        dec!(0.05), // risk-free rate
        dec!(0.20), // volatility
        1.0,        // time to expiry
        OptionType::Put,
    );

    let put_price = put_option.price()?;
    println!("   Black-Scholes Price: {}", put_price);

    let put_greeks = put_option.greeks()?;
    println!("   Greeks:");
    println!("     Delta: {:.6}", put_greeks.delta);
    println!("     Gamma: {:.6}", put_greeks.gamma);
    println!("     Theta (daily): {:.6}", put_greeks.theta);
    println!("     Vega (per 1% vol): {:.6}", put_greeks.vega);
    println!("     Rho (per 1% rate): {:.6}", put_greeks.rho);
    println!();

    // Verify put-call parity: C - P = S - K * e^(-rT)
    println!("3. Put-Call Parity Verification");
    let lhs = price.amount() - put_price.amount();
    let discount_factor = (-dec!(0.05)).exp();
    let rhs = dec!(100) - dec!(100) * discount_factor;
    println!("   LHS (C - P): {:.4}", lhs);
    println!("   RHS (S - K*e^(-rT)): {:.4}", rhs);
    println!("   Difference: {:.10}", (lhs - rhs).abs());
    println!();

    // Example 4: Implied Volatility
    println!("4. Implied Volatility Calculation");
    let market_price = dec!(10.45);
    let implied_vol = BlackScholes::implied_volatility(
        market_price,
        dec!(100),  // spot
        dec!(100),  // strike
        dec!(0.05), // rate
        1.0,        // time
        OptionType::Call,
        Some(0.2), // initial guess
    )?;
    println!("   Market Price: ${}", market_price);
    println!("   Implied Volatility: {:.2}%", implied_vol * 100.0);
    println!();

    // Example 5: Moneyness scenarios
    println!("5. Option Moneyness Scenarios");
    let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];

    println!("   Strike | Call Price | Call Delta | Put Price | Put Delta");
    println!("   {:-<55}", "");

    for strike in strikes {
        let call = EuropeanOption::new(
            Decimal::from_f64(strike).unwrap(),
            dec!(100),
            dec!(0.05),
            dec!(0.20),
            1.0,
            OptionType::Call,
        );

        let put = EuropeanOption::new(
            Decimal::from_f64(strike).unwrap(),
            dec!(100),
            dec!(0.05),
            dec!(0.20),
            1.0,
            OptionType::Put,
        );

        let call_price = call.price()?.amount();
        let call_delta = call.delta()?;
        let put_price = put.price()?.amount();
        let put_delta = put.delta()?;

        println!(
            "   ${:<6} | ${:<10.2} | {:<10.4} | ${:<9.2} | {:<10.4}",
            strike as i32, call_price, call_delta, put_price, put_delta
        );
    }
    println!();

    // Example 6: Portfolio Greeks
    println!("6. Portfolio Greeks");
    let mut portfolio = PortfolioGreeks::new();

    // Add some positions
    let long_call = EuropeanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Call,
    );
    let short_put = EuropeanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Put,
    );
    let long_call_105 = EuropeanOption::new(
        dec!(105),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Call,
    );

    portfolio.add_position(&long_call.greeks()?, 10.0); // Long 10 calls
    portfolio.add_position(&short_put.greeks()?, -5.0); // Short 5 puts
    portfolio.add_position(&long_call_105.greeks()?, 5.0); // Long 5 calls @ 105

    println!("   Portfolio composition:");
    println!("     Long 10 ATM calls");
    println!("     Short 5 ATM puts");
    println!("     Long 5 OTM calls @ 105");
    println!();
    println!("   {}", portfolio);
    println!();

    let hedge = portfolio.delta_hedge_suggestion();
    println!("   To delta-neutral: {} shares", hedge);
    println!();

    // Example 7: P&L approximation using Greeks
    println!("7. P&L Approximation");
    let atm_call = EuropeanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Call,
    );
    let g = atm_call.greeks()?;

    // Scenario: Spot +$2, 1 day passes, Vol +1%, Rate +0.5%
    let pnl = g.pnl_approximation(2.0, 1.0, 0.01, 0.005);
    println!("   Scenario: Spot +$2, 1 day, Vol +1%, Rate +0.5%");
    println!("   Expected P&L per contract: ${:.4}", pnl);

    Ok(())
}

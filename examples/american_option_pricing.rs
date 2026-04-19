//! American option pricing example
//!
//! This example demonstrates the Barone-Adesi Whaley (BAW) approximation
//! for pricing American options with early exercise features.

use pricing_lib::prelude::*;

fn main() -> Result<()> {
    println!("=== American Option Pricing with BAW ===\n");

    // Example 1: Basic American Call (no dividends)
    println!("1. American Call Option (No Dividends)");
    println!("   Spot: $100, Strike: $100, Rate: 5%, Vol: 20%, T: 1 year\n");

    let american_call = AmericanOption::new(
        dec!(100),  // strike
        dec!(100),  // spot
        dec!(0.05), // risk-free rate
        dec!(0.20), // volatility
        1.0,        // time to expiry
        OptionType::Call,
    );

    // Price using BAW
    let american_price = american_call.price()?;
    println!("   American Call (BAW): {}", american_price);

    // Compare with European (Black-Scholes)
    let european_call = EuropeanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Call,
    );
    let european_price = european_call.price()?;
    println!("   European Call (BS):  {}", european_price);

    // Early exercise premium
    let premium = american_price.amount() - european_price.amount();
    println!("   Early Exercise Premium: {}", premium);
    println!();

    // Example 2: American Put (shows early exercise value)
    println!("2. American Put Option (Deep ITM)");
    println!("   Spot: $80, Strike: $100, Rate: 5%, Vol: 20%, T: 1 year\n");

    let american_put = AmericanOption::new(
        dec!(100), // strike
        dec!(80),  // spot (deep ITM)
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Put,
    );

    let american_put_price = american_put.price()?;
    let european_put = EuropeanOption::new(
        dec!(100),
        dec!(80),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Put,
    );
    let european_put_price = european_put.price()?;

    println!("   American Put (BAW):  {}", american_put_price);
    println!("   European Put (BS):   {}", european_put_price);
    println!(
        "   Early Exercise Premium: {}",
        american_put_price.amount() - european_put_price.amount()
    );
    println!("   Intrinsic Value:     {}", american_put.intrinsic_value());
    println!();

    // Example 3: Effect of Dividends on Calls
    println!("3. Effect of Dividends on American Calls");
    println!("   Spot: $100, Strike: $100, Rate: 5%, Vol: 20%, T: 1 year\n");

    let no_div_call = AmericanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Call,
    );

    let with_div_call = AmericanOption::new_with_dividends(
        dec!(100),  // strike
        dec!(100),  // spot
        dec!(0.05), // risk-free rate
        dec!(0.20), // volatility
        1.0,        // time
        dec!(0.03), // dividend yield (3%)
        OptionType::Call,
    );

    let no_div_price = no_div_call.price()?;
    let with_div_price = with_div_call.price()?;

    println!("   No Dividends (q=0%):  {}", no_div_price);
    println!("   With Dividends (q=3%): {}", with_div_price);
    println!(
        "   Difference:            {}",
        no_div_price.amount() - with_div_price.amount()
    );
    println!();

    // Example 4: Greeks for American Options
    println!("4. American Option Greeks");
    println!("   Spot: $100, Strike: $100, Rate: 5%, Vol: 20%, T: 1 year\n");

    let option = AmericanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Call,
    );

    let greeks = option.greeks()?;

    println!("   Greeks (American Call):");
    println!("     Delta: {:.6}", greeks.delta);
    println!("     Gamma: {:.6}", greeks.gamma);
    println!("     Theta: {:.6} (daily)", greeks.theta);
    println!("     Vega:  {:.6} (per 1%)", greeks.vega);
    println!("     Rho:   {:.6} (per 1%)", greeks.rho);
    println!();

    // Compare with European Greeks
    let euro_greeks = european_call.greeks()?;
    println!("   Comparison with European Call:");
    println!(
        "     Delta: {:.6} (European: {:.6})",
        greeks.delta, euro_greeks.delta
    );
    println!(
        "     Gamma: {:.6} (European: {:.6})",
        greeks.gamma, euro_greeks.gamma
    );
    println!();

    // Example 5: Various Moneyness Levels
    println!("5. American vs European Comparison (Various Strikes)");
    println!("   Spot: $100, Rate: 5%, Vol: 20%, T: 1 year, No Dividends\n");

    let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];

    println!(
        "   {:<10} {:<15} {:<15} {:<12}",
        "Strike", "American", "European", "Premium"
    );
    println!("   {}", "-".repeat(55));

    for strike_f in strikes {
        let strike = Decimal::from_f64(strike_f).unwrap();

        let am = AmericanOption::new(
            strike,
            dec!(100),
            dec!(0.05),
            dec!(0.20),
            1.0,
            OptionType::Put,
        );
        let eu = EuropeanOption::new(
            strike,
            dec!(100),
            dec!(0.05),
            dec!(0.20),
            1.0,
            OptionType::Put,
        );

        let am_price = am.price()?.amount();
        let eu_price = eu.price()?.amount();
        let prem = am_price - eu_price;

        println!(
            "   ${:<9} ${:<14.2} ${:<14.2} ${:<11.2}",
            strike_f, am_price, eu_price, prem
        );
    }
    println!();

    // Example 6: Early Exercise Boundary
    println!("6. Early Exercise Premium Analysis");
    println!("   ATM Put: Spot=$100, Strike=$100\n");

    let _atm_put = AmericanOption::new(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        1.0,
        OptionType::Put,
    );

    let baw_price = BaroneAdesiWhaley::price(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        Decimal::ZERO,
        1.0,
        OptionType::Put,
    )?;

    let premium = BaroneAdesiWhaley::early_exercise_premium(
        dec!(100),
        dec!(100),
        dec!(0.05),
        dec!(0.20),
        Decimal::ZERO,
        1.0,
        OptionType::Put,
    )?;

    println!(
        "   American Price:          {}",
        Money::new(baw_price, CurrencyCode::USD)
    );
    println!(
        "   Early Exercise Premium:  {}",
        Money::new(premium, CurrencyCode::USD)
    );
    println!(
        "   Premium as % of Price:   {:.2}%",
        (premium / baw_price) * dec!(100)
    );

    Ok(())
}

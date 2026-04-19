//! # Pricing Library
//!
//! A Rust library for financial pricing and risk calculations.
//!
//! ## Features
//!
//! - **Precise decimal arithmetic** using `rust_decimal`
//! - **Fixed income instruments**: Bonds, zero-coupon bonds
//! - **Options pricing**: Black-Scholes, Greeks calculation
//! - **Day count conventions**: ACT/360, 30/360, ACT/ACT
//! - **Yield curves**: Bootstrapping, interpolation
//!
//! ## Example
//!
//! ```rust
//! use pricing_lib::prelude::*;
//!
//! fn main() -> Result<()> {
//!     let option = EuropeanOption::new(
//!         dec!(100), // strike
//!         dec!(105), // spot
//!         dec!(0.05), // risk-free rate
//!         dec!(0.2), // volatility
//!         1.0, // time to maturity in years
//!         OptionType::Call,
//!     );
//!
//!     let price = option.price()?;
//!     println!("Option price: {}", price);
//!
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod instruments;
pub mod pricing;
pub mod risk;
pub mod utils;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::{
        currency::*,
        day_count::*,
        error::{Error, Result},
        interest_rate::*,
        money::*,
        traits::*,
    };
    pub use crate::instruments::{bond::*, option::*};
    pub use crate::pricing::{barone_adesi_whaley::*, black_scholes::*, engine::*};
    pub use crate::risk::greeks::*;
    pub use rust_decimal::prelude::*;
    pub use rust_decimal_macros::dec;
}

// Re-export main types at crate root for convenience
pub use core::{
    currency::{Currency, CurrencyCode},
    day_count::DayCountConvention,
    error::{Error, Result},
    interest_rate::{Compounding, InterestRate},
    money::Money,
    traits::{HasGreeks, Pricable},
};

pub use instruments::{
    bond::{Bond, CouponBond, ZeroCouponBond},
    option::{AmericanOption, EuropeanOption, OptionType},
};

pub use pricing::{
    barone_adesi_whaley::BaroneAdesiWhaley, black_scholes::BlackScholes, engine::PricingEngine,
};

pub use risk::greeks::Greeks;

use std::collections::HashMap;
use std::str::FromStr;

use pricing_core::{AmericanOption, EuropeanOption, Instrument, OptionType};
use pricing_core::{HasGreeks, HasSecondOrderGreeks, Pricable};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::models::{
    GenericGreeksCurvePoint, GenericGreeksCurveResponse, GenericPriceCurvePoint,
    GenericPriceCurveResponse, GenericSecondOrderGreeksCurvePoint,
    GenericSecondOrderGreeksCurveResponse, ProductCatalogResponse, ProductParameter,
    ProductSchema,
};
use crate::handlers::AppError;

// ------------------------------------------------------------------
// Product catalog
// ------------------------------------------------------------------

pub fn product_catalog() -> ProductCatalogResponse {
    ProductCatalogResponse {
        products: vec![
            ProductSchema {
                id: "european-option".to_string(),
                name: "European Option".to_string(),
                category: "Derivatives".to_string(),
                parameters: vec![
                    ProductParameter::decimal("strike", "Strike Price", true),
                    ProductParameter::decimal("spot", "Spot Price", true),
                    ProductParameter::decimal_pct("risk_free_rate", "Risk-Free Rate", true),
                    ProductParameter::decimal_pct("volatility", "Volatility", true),
                    ProductParameter::float("time_to_maturity", "Time to Maturity", true, Some("years")),
                    ProductParameter::choice("option_type", "Option Type", true, vec!["Call", "Put"]),
                ],
                analytics: vec![
                    "price".to_string(),
                    "greeks".to_string(),
                    "second-order-greeks".to_string(),
                    "curve".to_string(),
                ],
            },
            ProductSchema {
                id: "american-option".to_string(),
                name: "American Option".to_string(),
                category: "Derivatives".to_string(),
                parameters: vec![
                    ProductParameter::decimal("strike", "Strike Price", true),
                    ProductParameter::decimal("spot", "Spot Price", true),
                    ProductParameter::decimal_pct("risk_free_rate", "Risk-Free Rate", true),
                    ProductParameter::decimal_pct("volatility", "Volatility", true),
                    ProductParameter::float("time_to_maturity", "Time to Maturity", true, Some("years")),
                    ProductParameter::choice("option_type", "Option Type", true, vec!["Call", "Put"]),
                    ProductParameter::decimal_pct("dividend_yield", "Dividend Yield", false),
                ],
                analytics: vec![
                    "price".to_string(),
                    "greeks".to_string(),
                    "second-order-greeks".to_string(),
                    "curve".to_string(),
                ],
            },
        ],
    }
}

// ------------------------------------------------------------------
// Parameter extraction helpers
// ------------------------------------------------------------------

fn get_decimal(params: &HashMap<String, serde_json::Value>, key: &str) -> Result<Decimal, AppError> {
    let val = params.get(key).ok_or_else(|| {
        AppError::BadRequest(format!("Missing required parameter: {}", key))
    })?;

    if let Some(s) = val.as_str() {
        Decimal::from_str(s)
            .map_err(|e| AppError::BadRequest(format!("Invalid decimal for {}: {}", key, e)))
    } else if let Some(n) = val.as_f64() {
        Decimal::from_f64(n)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid decimal for {}: {}", key, n)))
    } else {
        Err(AppError::BadRequest(format!(
            "Parameter {} must be a string or number",
            key
        )))
    }
}

fn get_f64(params: &HashMap<String, serde_json::Value>, key: &str) -> Result<f64, AppError> {
    let val = params.get(key).ok_or_else(|| {
        AppError::BadRequest(format!("Missing required parameter: {}", key))
    })?;

    if let Some(n) = val.as_f64() {
        Ok(n)
    } else if let Some(s) = val.as_str() {
        s.parse::<f64>()
            .map_err(|e| AppError::BadRequest(format!("Invalid float for {}: {}", key, e)))
    } else {
        Err(AppError::BadRequest(format!(
            "Parameter {} must be a number or string",
            key
        )))
    }
}

fn get_string(params: &HashMap<String, serde_json::Value>, key: &str) -> Result<String, AppError> {
    let val = params.get(key).ok_or_else(|| {
        AppError::BadRequest(format!("Missing required parameter: {}", key))
    })?;

    val.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::BadRequest(format!("Parameter {} must be a string", key)))
}

fn get_optional_decimal(
    params: &HashMap<String, serde_json::Value>,
    key: &str,
) -> Result<Option<Decimal>, AppError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) if v.is_null() => Ok(None),
        Some(v) if v.as_str() == Some("") => Ok(None),
        Some(v) => {
            if let Some(s) = v.as_str() {
                Decimal::from_str(s)
                    .map(Some)
                    .map_err(|e| AppError::BadRequest(format!("Invalid decimal for {}: {}", key, e)))
            } else if let Some(n) = v.as_f64() {
                Decimal::from_f64(n)
                    .map(Some)
                    .ok_or_else(|| AppError::BadRequest(format!("Invalid decimal for {}: {}", key, n)))
            } else {
                Err(AppError::BadRequest(format!(
                    "Parameter {} must be a string or number",
                    key
                )))
            }
        }
    }
}

fn parse_option_type(s: &str) -> Result<OptionType, AppError> {
    match s.to_lowercase().as_str() {
        "call" => Ok(OptionType::Call),
        "put" => Ok(OptionType::Put),
        _ => Err(AppError::BadRequest(format!(
            "Invalid option_type: {}. Use 'Call' or 'Put'.",
            s
        ))),
    }
}

// ------------------------------------------------------------------
// Instrument construction
// ------------------------------------------------------------------

/// Build an instrument from a product id and raw parameters.
pub fn build_instrument(
    product: &str,
    params: &HashMap<String, serde_json::Value>,
) -> Result<Box<dyn Instrument>, AppError> {
    match product {
        "european-option" => {
            let strike = get_decimal(params, "strike")?;
            let spot = get_decimal(params, "spot")?;
            let rate = get_decimal(params, "risk_free_rate")?;
            let vol = get_decimal(params, "volatility")?;
            let time = get_f64(params, "time_to_maturity")?;
            let opt_type = parse_option_type(&get_string(params, "option_type")?)?;
            Ok(Box::new(EuropeanOption::new(
                strike, spot, rate, vol, time, opt_type,
            )))
        }
        "american-option" => {
            let strike = get_decimal(params, "strike")?;
            let spot = get_decimal(params, "spot")?;
            let rate = get_decimal(params, "risk_free_rate")?;
            let vol = get_decimal(params, "volatility")?;
            let time = get_f64(params, "time_to_maturity")?;
            let opt_type = parse_option_type(&get_string(params, "option_type")?)?;
            let div = get_optional_decimal(params, "dividend_yield")?;
            if let Some(div) = div {
                Ok(Box::new(AmericanOption::new_with_dividends(
                    strike, spot, rate, vol, time, div, opt_type,
                )))
            } else {
                Ok(Box::new(AmericanOption::new(
                    strike, spot, rate, vol, time, opt_type,
                )))
            }
        }
        other => Err(AppError::BadRequest(format!(
            "Unknown product: {}. Use 'european-option' or 'american-option'.",
            other
        ))),
    }
}

// ------------------------------------------------------------------
// Trait dispatch
// ------------------------------------------------------------------

pub fn dispatch_price(instrument: &dyn Instrument) -> Result<pricing_core::core::money::Money, AppError> {
    if let Some(opt) = instrument.as_any().downcast_ref::<EuropeanOption>() {
        Ok(opt.price()?)
    } else if let Some(opt) = instrument.as_any().downcast_ref::<AmericanOption>() {
        Ok(opt.price()?)
    } else {
        Err(AppError::BadRequest(
            "Product does not support pricing".to_string(),
        ))
    }
}

pub fn dispatch_greeks(instrument: &dyn Instrument) -> Result<pricing_core::risk::greeks::Greeks, AppError> {
    if let Some(opt) = instrument.as_any().downcast_ref::<EuropeanOption>() {
        Ok(opt.greeks()?)
    } else if let Some(opt) = instrument.as_any().downcast_ref::<AmericanOption>() {
        Ok(opt.greeks()?)
    } else {
        Err(AppError::BadRequest(
            "Product does not support Greeks".to_string(),
        ))
    }
}

pub fn dispatch_second_order_greeks(
    instrument: &dyn Instrument,
) -> Result<pricing_core::risk::greeks::SecondOrderGreeks, AppError> {
    if let Some(opt) = instrument.as_any().downcast_ref::<EuropeanOption>() {
        Ok(opt.second_order_greeks()?)
    } else if let Some(opt) = instrument.as_any().downcast_ref::<AmericanOption>() {
        Ok(opt.second_order_greeks()?)
    } else {
        Err(AppError::BadRequest(
            "Product does not support second-order Greeks".to_string(),
        ))
    }
}

// ------------------------------------------------------------------
// Curve helpers
// ------------------------------------------------------------------

/// Build a EuropeanOption from parameters for curve calculations.
fn build_european_option(
    params: &HashMap<String, serde_json::Value>,
) -> Result<EuropeanOption, AppError> {
    let strike = get_decimal(params, "strike")?;
    let spot = get_decimal(params, "spot")?;
    let rate = get_decimal(params, "risk_free_rate")?;
    let vol = get_decimal(params, "volatility")?;
    let time = get_f64(params, "time_to_maturity")?;
    let opt_type = parse_option_type(&get_string(params, "option_type")?)?;
    Ok(EuropeanOption::new(strike, spot, rate, vol, time, opt_type))
}

/// Build an AmericanOption from parameters for curve calculations.
fn build_american_option(
    params: &HashMap<String, serde_json::Value>,
) -> Result<AmericanOption, AppError> {
    let strike = get_decimal(params, "strike")?;
    let spot = get_decimal(params, "spot")?;
    let rate = get_decimal(params, "risk_free_rate")?;
    let vol = get_decimal(params, "volatility")?;
    let time = get_f64(params, "time_to_maturity")?;
    let opt_type = parse_option_type(&get_string(params, "option_type")?)?;
    let div = get_optional_decimal(params, "dividend_yield")?;
    if let Some(div) = div {
        Ok(AmericanOption::new_with_dividends(
            strike, spot, rate, vol, time, div, opt_type,
        ))
    } else {
        Ok(AmericanOption::new(strike, spot, rate, vol, time, opt_type))
    }
}

/// Generic price curve generator.
pub fn generic_price_curve(
    product: &str,
    params: &HashMap<String, serde_json::Value>,
    strikes: Option<Vec<Decimal>>,
    spots: Option<Vec<Decimal>>,
    fixed_strike: Option<Decimal>,
) -> Result<GenericPriceCurveResponse, AppError> {
    let varying_spots = spots.filter(|s| !s.is_empty());
    let _fixed_strike = fixed_strike.or_else(|| {
        get_decimal(params, "spot").ok()
    });

    let points: Vec<GenericPriceCurvePoint> = if let Some(spots) = varying_spots {
        spots
            .into_iter()
            .map(|spot| {
                let mut p = params.clone();
                p.insert("spot".to_string(), serde_json::Value::String(spot.to_string()));
                let price = match product {
                    "european-option" => {
                        let opt = build_european_option(&p)?;
                        opt.price()?.amount()
                    }
                    "american-option" => {
                        let opt = build_american_option(&p)?;
                        opt.price()?.amount()
                    }
                    other => return Err(AppError::BadRequest(format!(
                        "Invalid product for curve: {}", other
                    ))),
                };
                Ok(GenericPriceCurvePoint { x: spot, price })
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let strikes = strikes.ok_or_else(|| {
            AppError::BadRequest("Either strikes or spots must be provided for curve".to_string())
        })?;
        strikes
            .into_iter()
            .map(|strike| {
                let mut p = params.clone();
                p.insert("strike".to_string(), serde_json::Value::String(strike.to_string()));
                let price = match product {
                    "european-option" => {
                        let opt = build_european_option(&p)?;
                        opt.price()?.amount()
                    }
                    "american-option" => {
                        let opt = build_american_option(&p)?;
                        opt.price()?.amount()
                    }
                    other => return Err(AppError::BadRequest(format!(
                        "Invalid product for curve: {}", other
                    ))),
                };
                Ok(GenericPriceCurvePoint { x: strike, price })
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(GenericPriceCurveResponse {
        currency: "USD".to_string(),
        points,
    })
}

/// Generic Greeks curve generator.
pub fn generic_greeks_curve(
    product: &str,
    params: &HashMap<String, serde_json::Value>,
    strikes: Option<Vec<Decimal>>,
    spots: Option<Vec<Decimal>>,
    fixed_strike: Option<Decimal>,
) -> Result<GenericGreeksCurveResponse, AppError> {
    let varying_spots = spots.filter(|s| !s.is_empty());
    let _fixed_strike = fixed_strike.or_else(|| {
        get_decimal(params, "spot").ok()
    });

    let points: Vec<GenericGreeksCurvePoint> = if let Some(spots) = varying_spots {
        spots
            .into_iter()
            .map(|spot| {
                let mut p = params.clone();
                p.insert("spot".to_string(), serde_json::Value::String(spot.to_string()));
                let greeks = match product {
                    "european-option" => {
                        let opt = build_european_option(&p)?;
                        opt.greeks()?
                    }
                    "american-option" => {
                        let opt = build_american_option(&p)?;
                        opt.greeks()?
                    }
                    other => return Err(AppError::BadRequest(format!(
                        "Invalid product for curve: {}", other
                    ))),
                };
                Ok(GenericGreeksCurvePoint {
                    x: spot,
                    delta: greeks.delta,
                    gamma: greeks.gamma,
                    theta: greeks.theta,
                    vega: greeks.vega,
                    rho: greeks.rho,
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let strikes = strikes.ok_or_else(|| {
            AppError::BadRequest("Either strikes or spots must be provided for curve".to_string())
        })?;
        strikes
            .into_iter()
            .map(|strike| {
                let mut p = params.clone();
                p.insert("strike".to_string(), serde_json::Value::String(strike.to_string()));
                let greeks = match product {
                    "european-option" => {
                        let opt = build_european_option(&p)?;
                        opt.greeks()?
                    }
                    "american-option" => {
                        let opt = build_american_option(&p)?;
                        opt.greeks()?
                    }
                    other => return Err(AppError::BadRequest(format!(
                        "Invalid product for curve: {}", other
                    ))),
                };
                Ok(GenericGreeksCurvePoint {
                    x: strike,
                    delta: greeks.delta,
                    gamma: greeks.gamma,
                    theta: greeks.theta,
                    vega: greeks.vega,
                    rho: greeks.rho,
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(GenericGreeksCurveResponse { points })
}

/// Generic second-order Greeks curve generator.
pub fn generic_second_order_greeks_curve(
    product: &str,
    params: &HashMap<String, serde_json::Value>,
    strikes: Option<Vec<Decimal>>,
    spots: Option<Vec<Decimal>>,
    fixed_strike: Option<Decimal>,
) -> Result<GenericSecondOrderGreeksCurveResponse, AppError> {
    let varying_spots = spots.filter(|s| !s.is_empty());
    let _fixed_strike = fixed_strike.or_else(|| {
        get_decimal(params, "spot").ok()
    });

    let points: Vec<GenericSecondOrderGreeksCurvePoint> = if let Some(spots) = varying_spots {
        spots
            .into_iter()
            .map(|spot| {
                let mut p = params.clone();
                p.insert("spot".to_string(), serde_json::Value::String(spot.to_string()));
                let sog = match product {
                    "european-option" => {
                        let opt = build_european_option(&p)?;
                        opt.second_order_greeks()?
                    }
                    "american-option" => {
                        let opt = build_american_option(&p)?;
                        opt.second_order_greeks()?
                    }
                    other => return Err(AppError::BadRequest(format!(
                        "Invalid product for curve: {}", other
                    ))),
                };
                Ok(GenericSecondOrderGreeksCurvePoint {
                    x: spot,
                    vanna: sog.vanna,
                    charm: sog.charm,
                    vomma: sog.vomma,
                    speed: sog.speed,
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let strikes = strikes.ok_or_else(|| {
            AppError::BadRequest("Either strikes or spots must be provided for curve".to_string())
        })?;
        strikes
            .into_iter()
            .map(|strike| {
                let mut p = params.clone();
                p.insert("strike".to_string(), serde_json::Value::String(strike.to_string()));
                let sog = match product {
                    "european-option" => {
                        let opt = build_european_option(&p)?;
                        opt.second_order_greeks()?
                    }
                    "american-option" => {
                        let opt = build_american_option(&p)?;
                        opt.second_order_greeks()?
                    }
                    other => return Err(AppError::BadRequest(format!(
                        "Invalid product for curve: {}", other
                    ))),
                };
                Ok(GenericSecondOrderGreeksCurvePoint {
                    x: strike,
                    vanna: sog.vanna,
                    charm: sog.charm,
                    vomma: sog.vomma,
                    speed: sog.speed,
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(GenericSecondOrderGreeksCurveResponse { points })
}

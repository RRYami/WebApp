use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct EuropeanOptionRequest {
    pub strike: Decimal,
    pub spot: Decimal,
    pub risk_free_rate: Decimal,
    pub volatility: Decimal,
    pub time_to_maturity: f64,
    pub option_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AmericanOptionRequest {
    pub strike: Decimal,
    pub spot: Decimal,
    pub risk_free_rate: Decimal,
    pub volatility: Decimal,
    pub time_to_maturity: f64,
    pub option_type: String,
    pub dividend_yield: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct PriceResponse {
    pub price: Decimal,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct GreeksResponse {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[derive(Debug, Deserialize)]
pub struct CurveRequest {
    pub instrument: String,
    pub option_type: String,
    pub spot: Decimal,
    pub risk_free_rate: Decimal,
    pub volatility: Decimal,
    pub time_to_maturity: f64,
    pub dividend_yield: Option<Decimal>,
    pub strikes: Vec<Decimal>,
    pub spots: Option<Vec<Decimal>>,
    pub fixed_strike: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct PriceCurvePoint {
    pub strike: Decimal,
    pub price: Decimal,
}

#[derive(Debug, Serialize)]
pub struct PriceCurveResponse {
    pub currency: String,
    pub points: Vec<PriceCurvePoint>,
}

#[derive(Debug, Serialize)]
pub struct GreeksCurvePoint {
    pub strike: Decimal,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[derive(Debug, Serialize)]
pub struct GreeksCurveResponse {
    pub points: Vec<GreeksCurvePoint>,
}

#[derive(Debug, Serialize)]
pub struct SecondOrderGreeksResponse {
    pub vanna: f64,
    pub charm: f64,
    pub vomma: f64,
    pub speed: f64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ------------------------------------------------------------------
// Heston model
// ------------------------------------------------------------------

/// Heston model parameters as they appear on the wire.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct HestonParamsDto {
    pub v0: f64,
    pub kappa: f64,
    pub theta: f64,
    pub sigma: f64,
    pub rho: f64,
}

/// Request to price a vanilla option under user-supplied Heston parameters.
#[derive(Debug, Deserialize)]
pub struct HestonOptionRequest {
    pub strike: Decimal,
    pub spot: Decimal,
    pub risk_free_rate: Decimal,
    pub time_to_maturity: f64,
    pub option_type: String,
    pub heston_params: HestonParamsDto,
}

/// Request to calibrate Heston parameters from stored market quotes.
#[derive(Debug, Deserialize)]
pub struct CalibrateHestonRequest {
    /// Underlying symbol to load quotes for from `options_data`.
    pub symbol: String,
    pub spot: Decimal,
    pub risk_free_rate: Decimal,
    /// Calibration timestamp; defaults to now. The latest quote at or before
    /// this time is used for each contract.
    pub as_of: Option<DateTime<Utc>>,
    /// Optional optimiser starting point.
    pub initial_guess: Option<HestonParamsDto>,
}

#[derive(Debug, Serialize)]
pub struct CalibrateHestonResponse {
    pub params: HestonParamsDto,
    pub rmse: f64,
    pub iterations: usize,
    pub converged: bool,
    pub quotes_used: usize,
}

// ------------------------------------------------------------------
// Generic product-driven API models
// ------------------------------------------------------------------

/// A single parameter definition for a product schema.
#[derive(Debug, Serialize, Clone)]
pub struct ProductParameter {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_as_percentage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

impl ProductParameter {
    pub fn decimal(id: &str, label: &str, required: bool) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            param_type: "decimal".to_string(),
            required,
            display_as_percentage: None,
            unit: None,
            options: None,
        }
    }

    pub fn decimal_pct(id: &str, label: &str, required: bool) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            param_type: "decimal".to_string(),
            required,
            display_as_percentage: Some(true),
            unit: None,
            options: None,
        }
    }

    pub fn float(id: &str, label: &str, required: bool, unit: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            param_type: "float".to_string(),
            required,
            display_as_percentage: None,
            unit: unit.map(|s| s.to_string()),
            options: None,
        }
    }

    pub fn choice(id: &str, label: &str, required: bool, options: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            param_type: "choice".to_string(),
            required,
            display_as_percentage: None,
            unit: None,
            options: Some(options.into_iter().map(|s| s.to_string()).collect()),
        }
    }
}

/// Schema describing a single product available in the platform.
#[derive(Debug, Serialize, Clone)]
pub struct ProductSchema {
    pub id: String,
    pub name: String,
    pub category: String,
    pub parameters: Vec<ProductParameter>,
    pub analytics: Vec<String>,
}

/// Response returned by `GET /api/products`.
#[derive(Debug, Serialize)]
pub struct ProductCatalogResponse {
    pub products: Vec<ProductSchema>,
}

/// Generic request body for any analytics calculation.
#[derive(Debug, Deserialize)]
pub struct GenericAnalyticsRequest {
    pub product: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Generic response for a price calculation.
#[derive(Debug, Serialize)]
pub struct GenericPriceResponse {
    pub price: Decimal,
    pub currency: String,
}

/// Generic response for a Greeks calculation.
#[derive(Debug, Serialize)]
pub struct GenericGreeksResponse {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

/// Generic response for a second-order Greeks calculation.
#[derive(Debug, Serialize)]
pub struct GenericSecondOrderGreeksResponse {
    pub vanna: f64,
    pub charm: f64,
    pub vomma: f64,
    pub speed: f64,
}

/// Generic request body for curve generation.
#[derive(Debug, Deserialize)]
pub struct GenericCurveRequest {
    pub product: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub strikes: Option<Vec<Decimal>>,
    pub spots: Option<Vec<Decimal>>,
    pub fixed_strike: Option<Decimal>,
}

/// Generic response for a price curve.
#[derive(Debug, Serialize)]
pub struct GenericPriceCurvePoint {
    pub x: Decimal,
    pub price: Decimal,
}

#[derive(Debug, Serialize)]
pub struct GenericPriceCurveResponse {
    pub currency: String,
    pub points: Vec<GenericPriceCurvePoint>,
}

/// Generic response for a Greeks curve.
#[derive(Debug, Serialize)]
pub struct GenericGreeksCurvePoint {
    pub x: Decimal,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[derive(Debug, Serialize)]
pub struct GenericGreeksCurveResponse {
    pub points: Vec<GenericGreeksCurvePoint>,
}

/// Generic response for a second-order Greeks curve.
#[derive(Debug, Serialize)]
pub struct GenericSecondOrderGreeksCurvePoint {
    pub x: Decimal,
    pub vanna: f64,
    pub charm: f64,
    pub vomma: f64,
    pub speed: f64,
}

#[derive(Debug, Serialize)]
pub struct GenericSecondOrderGreeksCurveResponse {
    pub points: Vec<GenericSecondOrderGreeksCurvePoint>,
}

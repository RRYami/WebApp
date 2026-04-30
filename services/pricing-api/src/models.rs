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
    pub phi: f64,
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
    pub phi: f64,
}

#[derive(Debug, Serialize)]
pub struct GreeksCurveResponse {
    pub points: Vec<GreeksCurvePoint>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

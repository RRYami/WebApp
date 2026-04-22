use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use pricing_core::{AmericanOption, EuropeanOption, OptionType, Pricable, HasGreeks};

use crate::models::{
    AmericanOptionRequest, ErrorResponse, EuropeanOptionRequest, GreeksResponse, PriceResponse,
};

pub async fn health() -> &'static str {
    "ok"
}

pub async fn price_european_option(
    Json(req): Json<EuropeanOptionRequest>,
) -> Result<Json<PriceResponse>, AppError> {
    let option_type = parse_option_type(&req.option_type)?;
    let option = EuropeanOption::new(
        req.strike,
        req.spot,
        req.risk_free_rate,
        req.volatility,
        req.time_to_maturity,
        option_type,
    );
    let price = option.price()?;
    Ok(Json(PriceResponse {
        price: price.amount(),
        currency: price.currency().to_string(),
    }))
}

pub async fn price_american_option(
    Json(req): Json<AmericanOptionRequest>,
) -> Result<Json<PriceResponse>, AppError> {
    let option_type = parse_option_type(&req.option_type)?;
    let option = if let Some(div) = req.dividend_yield {
        AmericanOption::new_with_dividends(
            req.strike,
            req.spot,
            req.risk_free_rate,
            req.volatility,
            req.time_to_maturity,
            div,
            option_type,
        )
    } else {
        AmericanOption::new(
            req.strike,
            req.spot,
            req.risk_free_rate,
            req.volatility,
            req.time_to_maturity,
            option_type,
        )
    };
    let price = option.price()?;
    Ok(Json(PriceResponse {
        price: price.amount(),
        currency: price.currency().to_string(),
    }))
}

pub async fn greeks_european_option(
    Json(req): Json<EuropeanOptionRequest>,
) -> Result<Json<GreeksResponse>, AppError> {
    let option_type = parse_option_type(&req.option_type)?;
    let option = EuropeanOption::new(
        req.strike,
        req.spot,
        req.risk_free_rate,
        req.volatility,
        req.time_to_maturity,
        option_type,
    );
    let greeks = option.greeks()?;
    Ok(Json(GreeksResponse {
        delta: greeks.delta,
        gamma: greeks.gamma,
        theta: greeks.theta,
        vega: greeks.vega,
        rho: greeks.rho,
        phi: greeks.phi,
    }))
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

#[derive(Debug)]
pub enum AppError {
    Pricing(pricing_core::Error),
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Pricing(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

impl From<pricing_core::Error> for AppError {
    fn from(err: pricing_core::Error) -> Self {
        AppError::Pricing(err)
    }
}

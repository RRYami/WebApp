use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use pricing_core::{AmericanOption, EuropeanOption, OptionType, Pricable, HasGreeks, BaroneAdesiWhaley};

use crate::models::{
    AmericanOptionRequest, CurveRequest, ErrorResponse, EuropeanOptionRequest, GreeksCurvePoint,
    GreeksCurveResponse, GreeksResponse, PriceCurvePoint, PriceCurveResponse, PriceResponse,
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

pub async fn greeks_american_option(
    Json(req): Json<AmericanOptionRequest>,
) -> Result<Json<GreeksResponse>, AppError> {
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

pub async fn price_baw_american_option(
    Json(req): Json<AmericanOptionRequest>,
) -> Result<Json<PriceResponse>, AppError> {
    let option_type = parse_option_type(&req.option_type)?;
    let price = BaroneAdesiWhaley::price(
        req.spot,
        req.strike,
        req.risk_free_rate,
        req.volatility,
        req.dividend_yield.unwrap_or_default(),
        req.time_to_maturity,
        option_type,
    )?;
    Ok(Json(PriceResponse {
        price,
        currency: "USD".to_string(),
    }))
}

pub async fn price_curve(
    Json(req): Json<CurveRequest>,
) -> Result<Json<PriceCurveResponse>, AppError> {
    let option_type = parse_option_type(&req.option_type)?;
    let div = req.dividend_yield.unwrap_or_default();

    let points: Result<Vec<_>, _> = req
        .strikes
        .into_iter()
        .map(|strike| {
            let price = match req.instrument.to_lowercase().as_str() {
                "european" => {
                    let opt = EuropeanOption::new(
                        strike, req.spot, req.risk_free_rate, req.volatility,
                        req.time_to_maturity, option_type,
                    );
                    opt.price()?.amount()
                }
                "american" | "baw" => {
                    BaroneAdesiWhaley::price(
                        req.spot, strike, req.risk_free_rate, req.volatility,
                        div, req.time_to_maturity, option_type,
                    )?
                }
                other => return Err(AppError::BadRequest(format!(
                    "Invalid instrument for curve: {}. Use 'European', 'American', or 'BAW'.",
                    other
                ))),
            };
            Ok(PriceCurvePoint { strike, price })
        })
        .collect();

    Ok(Json(PriceCurveResponse {
        currency: "USD".to_string(),
        points: points?,
    }))
}

pub async fn greeks_curve(
    Json(req): Json<CurveRequest>,
) -> Result<Json<GreeksCurveResponse>, AppError> {
    let option_type = parse_option_type(&req.option_type)?;
    let div = req.dividend_yield.unwrap_or_default();

    // If spots are provided, compute Greeks vs underlying price (varying spot, fixed strike).
    // Otherwise compute Greeks vs strike (varying strike, fixed spot).
    let varying_spots = req.spots.filter(|s| !s.is_empty());
    let fixed_strike = req.fixed_strike.unwrap_or(req.spot);

    let points: Result<Vec<_>, _> = if let Some(spots) = varying_spots {
        spots
            .into_iter()
            .map(|spot| {
                let greeks = match req.instrument.to_lowercase().as_str() {
                    "european" => {
                        let opt = EuropeanOption::new(
                            fixed_strike, spot, req.risk_free_rate, req.volatility,
                            req.time_to_maturity, option_type,
                        );
                        opt.greeks()?
                    }
                    "american" | "baw" => {
                        let opt = AmericanOption::new_with_dividends(
                            fixed_strike, spot, req.risk_free_rate, req.volatility,
                            req.time_to_maturity, div, option_type,
                        );
                        opt.greeks()?
                    }
                    other => return Err(AppError::BadRequest(format!(
                        "Invalid instrument for curve: {}. Use 'European', 'American', or 'BAW'.",
                        other
                    ))),
                };
                Ok(GreeksCurvePoint {
                    strike: spot, // reuse field for x-axis value (spot price)
                    delta: greeks.delta,
                    gamma: greeks.gamma,
                    theta: greeks.theta,
                    vega: greeks.vega,
                    rho: greeks.rho,
                    phi: greeks.phi,
                })
            })
            .collect()
    } else {
        req.strikes
            .into_iter()
            .map(|strike| {
                let greeks = match req.instrument.to_lowercase().as_str() {
                    "european" => {
                        let opt = EuropeanOption::new(
                            strike, req.spot, req.risk_free_rate, req.volatility,
                            req.time_to_maturity, option_type,
                        );
                        opt.greeks()?
                    }
                    "american" | "baw" => {
                        let opt = AmericanOption::new_with_dividends(
                            strike, req.spot, req.risk_free_rate, req.volatility,
                            req.time_to_maturity, div, option_type,
                        );
                        opt.greeks()?
                    }
                    other => return Err(AppError::BadRequest(format!(
                        "Invalid instrument for curve: {}. Use 'European', 'American', or 'BAW'.",
                        other
                    ))),
                };
                Ok(GreeksCurvePoint {
                    strike,
                    delta: greeks.delta,
                    gamma: greeks.gamma,
                    theta: greeks.theta,
                    vega: greeks.vega,
                    rho: greeks.rho,
                    phi: greeks.phi,
                })
            })
            .collect()
    };

    Ok(Json(GreeksCurveResponse { points: points? }))
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

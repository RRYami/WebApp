use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use pricing_core::{
    AmericanOption, BaroneAdesiWhaley, CalibrationConfig, EuropeanOption, HasGreeks,
    HasSecondOrderGreeks, Heston, HestonCalibrator, HestonParams, MarketQuote, OptionType,
    Pricable,
};
use rust_decimal::Decimal;

use crate::db::{self, AppState};
use crate::dispatch;
use crate::models::{
    AmericanOptionRequest, CalibrateHestonRequest, CalibrateHestonResponse, CurveRequest,
    ErrorResponse, EuropeanOptionRequest, GenericAnalyticsRequest, GenericCurveRequest,
    GenericGreeksCurveResponse, GenericGreeksResponse, GenericPriceCurveResponse,
    GenericPriceResponse, GenericSecondOrderGreeksCurveResponse, GenericSecondOrderGreeksResponse,
    GreeksCurvePoint, GreeksCurveResponse, GreeksResponse, HestonOptionRequest, HestonParamsDto,
    PriceCurvePoint, PriceCurveResponse, PriceResponse, ProductCatalogResponse,
    SecondOrderGreeksResponse,
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
    }))
}

pub async fn second_order_greeks_european_option(
    Json(req): Json<EuropeanOptionRequest>,
) -> Result<Json<SecondOrderGreeksResponse>, AppError> {
    let option_type = parse_option_type(&req.option_type)?;
    let option = EuropeanOption::new(
        req.strike,
        req.spot,
        req.risk_free_rate,
        req.volatility,
        req.time_to_maturity,
        option_type,
    );
    let sog = option.second_order_greeks()?;
    Ok(Json(SecondOrderGreeksResponse {
        vanna: sog.vanna,
        charm: sog.charm,
        vomma: sog.vomma,
        speed: sog.speed,
    }))
}

pub async fn second_order_greeks_american_option(
    Json(req): Json<AmericanOptionRequest>,
) -> Result<Json<SecondOrderGreeksResponse>, AppError> {
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
    let sog = option.second_order_greeks()?;
    Ok(Json(SecondOrderGreeksResponse {
        vanna: sog.vanna,
        charm: sog.charm,
        vomma: sog.vomma,
        speed: sog.speed,
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
                        strike,
                        req.spot,
                        req.risk_free_rate,
                        req.volatility,
                        req.time_to_maturity,
                        option_type,
                    );
                    opt.price()?.amount()
                }
                "american" | "baw" => BaroneAdesiWhaley::price(
                    req.spot,
                    strike,
                    req.risk_free_rate,
                    req.volatility,
                    div,
                    req.time_to_maturity,
                    option_type,
                )?,
                other => {
                    return Err(AppError::BadRequest(format!(
                        "Invalid instrument for curve: {}. Use 'European', 'American', or 'BAW'.",
                        other
                    )))
                }
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
                            fixed_strike,
                            spot,
                            req.risk_free_rate,
                            req.volatility,
                            req.time_to_maturity,
                            option_type,
                        );
                        opt.greeks()?
                    }
                    "american" | "baw" => {
                        let opt = AmericanOption::new_with_dividends(
                            fixed_strike,
                            spot,
                            req.risk_free_rate,
                            req.volatility,
                            req.time_to_maturity,
                            div,
                            option_type,
                        );
                        opt.greeks()?
                    }
                    other => {
                        return Err(AppError::BadRequest(format!(
                        "Invalid instrument for curve: {}. Use 'European', 'American', or 'BAW'.",
                        other
                    )))
                    }
                };
                Ok(GreeksCurvePoint {
                    strike: spot, // reuse field for x-axis value (spot price)
                    delta: greeks.delta,
                    gamma: greeks.gamma,
                    theta: greeks.theta,
                    vega: greeks.vega,
                    rho: greeks.rho,
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
                            strike,
                            req.spot,
                            req.risk_free_rate,
                            req.volatility,
                            req.time_to_maturity,
                            option_type,
                        );
                        opt.greeks()?
                    }
                    "american" | "baw" => {
                        let opt = AmericanOption::new_with_dividends(
                            strike,
                            req.spot,
                            req.risk_free_rate,
                            req.volatility,
                            req.time_to_maturity,
                            div,
                            option_type,
                        );
                        opt.greeks()?
                    }
                    other => {
                        return Err(AppError::BadRequest(format!(
                        "Invalid instrument for curve: {}. Use 'European', 'American', or 'BAW'.",
                        other
                    )))
                    }
                };
                Ok(GreeksCurvePoint {
                    strike,
                    delta: greeks.delta,
                    gamma: greeks.gamma,
                    theta: greeks.theta,
                    vega: greeks.vega,
                    rho: greeks.rho,
                })
            })
            .collect()
    };

    Ok(Json(GreeksCurveResponse { points: points? }))
}

// ------------------------------------------------------------------
// Heston model handlers
// ------------------------------------------------------------------

/// Price a vanilla European option under user-supplied Heston parameters.
/// No calibration or database access required.
pub async fn price_heston_option(
    Json(req): Json<HestonOptionRequest>,
) -> Result<Json<PriceResponse>, AppError> {
    let option_type = parse_option_type(&req.option_type)?;
    let params = HestonParams::new(
        req.heston_params.v0,
        req.heston_params.kappa,
        req.heston_params.theta,
        req.heston_params.sigma,
        req.heston_params.rho,
    )?;
    let price = Heston::price(
        req.spot,
        req.strike,
        req.risk_free_rate,
        &params,
        req.time_to_maturity,
        option_type,
    )?;
    Ok(Json(PriceResponse {
        price,
        currency: "USD".to_string(),
    }))
}

/// Calibrate Heston parameters from the latest quotes stored in the
/// `options_data` table for the requested symbol.
pub async fn calibrate_heston(
    State(state): State<AppState>,
    Json(req): Json<CalibrateHestonRequest>,
) -> Result<Json<CalibrateHestonResponse>, AppError> {
    let pool = state.db.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailable(
            "Database not available; set DATABASE_URL to enable calibration".to_string(),
        )
    })?;

    let as_of = req.as_of.unwrap_or_else(Utc::now);
    let rows = db::fetch_latest_quotes(pool, &req.symbol, as_of)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to load quotes: {e}")))?;

    let quotes: Vec<MarketQuote> = rows
        .into_iter()
        .filter_map(|row| {
            let bid = row.bid?;
            let ask = row.ask?;
            if bid <= Decimal::ZERO || ask < bid {
                return None;
            }
            let mid = (bid + ask) / Decimal::TWO;
            let seconds_to_expiry = (row.expiry - as_of).num_seconds();
            let time_to_expiry = seconds_to_expiry as f64 / (365.0 * 86_400.0);
            if time_to_expiry <= 0.0 {
                return None;
            }
            let option_type = match row.option_type.to_lowercase().as_str() {
                "call" | "c" => OptionType::Call,
                "put" | "p" => OptionType::Put,
                _ => return None,
            };
            Some(MarketQuote {
                strike: row.strike,
                time_to_expiry,
                option_type,
                market_price: mid,
            })
        })
        .collect();

    if quotes.len() < 5 {
        return Err(AppError::BadRequest(format!(
            "Not enough usable quotes for '{}' as of {} (found {}, need at least 5)",
            req.symbol,
            as_of,
            quotes.len()
        )));
    }
    let quotes_used = quotes.len();

    let mut config = CalibrationConfig::default();
    if let Some(guess) = req.initial_guess {
        config.initial_guess =
            HestonParams::new(guess.v0, guess.kappa, guess.theta, guess.sigma, guess.rho)?;
    }
    let calibrator = HestonCalibrator::with_config(req.spot, req.risk_free_rate, config);

    // Calibration is CPU-bound; keep it off the async worker threads.
    let result = tokio::task::spawn_blocking(move || calibrator.calibrate(&quotes))
        .await
        .map_err(|e| AppError::Internal(format!("Calibration task failed: {e}")))??;

    Ok(Json(CalibrateHestonResponse {
        params: HestonParamsDto {
            v0: result.params.v0,
            kappa: result.params.kappa,
            theta: result.params.theta,
            sigma: result.params.sigma,
            rho: result.params.rho,
        },
        rmse: result.rmse,
        iterations: result.iterations,
        converged: result.converged,
        quotes_used,
    }))
}

// ------------------------------------------------------------------
// Generic product-driven handlers
// ------------------------------------------------------------------

pub async fn product_catalog() -> Json<ProductCatalogResponse> {
    Json(dispatch::product_catalog())
}

pub async fn generic_price(
    Json(req): Json<GenericAnalyticsRequest>,
) -> Result<Json<GenericPriceResponse>, AppError> {
    let instrument = dispatch::build_instrument(&req.product, &req.parameters)?;
    let price = dispatch::dispatch_price(instrument.as_ref())?;
    Ok(Json(GenericPriceResponse {
        price: price.amount(),
        currency: price.currency().to_string(),
    }))
}

pub async fn generic_greeks(
    Json(req): Json<GenericAnalyticsRequest>,
) -> Result<Json<GenericGreeksResponse>, AppError> {
    let instrument = dispatch::build_instrument(&req.product, &req.parameters)?;
    let greeks = dispatch::dispatch_greeks(instrument.as_ref())?;
    Ok(Json(GenericGreeksResponse {
        delta: greeks.delta,
        gamma: greeks.gamma,
        theta: greeks.theta,
        vega: greeks.vega,
        rho: greeks.rho,
    }))
}

pub async fn generic_second_order_greeks(
    Json(req): Json<GenericAnalyticsRequest>,
) -> Result<Json<GenericSecondOrderGreeksResponse>, AppError> {
    let instrument = dispatch::build_instrument(&req.product, &req.parameters)?;
    let sog = dispatch::dispatch_second_order_greeks(instrument.as_ref())?;
    Ok(Json(GenericSecondOrderGreeksResponse {
        vanna: sog.vanna,
        charm: sog.charm,
        vomma: sog.vomma,
        speed: sog.speed,
    }))
}

pub async fn generic_price_curve(
    Json(req): Json<GenericCurveRequest>,
) -> Result<Json<GenericPriceCurveResponse>, AppError> {
    let resp = dispatch::generic_price_curve(
        &req.product,
        &req.parameters,
        req.strikes,
        req.spots,
        req.fixed_strike,
    )?;
    Ok(Json(resp))
}

pub async fn generic_greeks_curve(
    Json(req): Json<GenericCurveRequest>,
) -> Result<Json<GenericGreeksCurveResponse>, AppError> {
    let resp = dispatch::generic_greeks_curve(
        &req.product,
        &req.parameters,
        req.strikes,
        req.spots,
        req.fixed_strike,
    )?;
    Ok(Json(resp))
}

pub async fn generic_second_order_greeks_curve(
    Json(req): Json<GenericCurveRequest>,
) -> Result<Json<GenericSecondOrderGreeksCurveResponse>, AppError> {
    let resp = dispatch::generic_second_order_greeks_curve(
        &req.product,
        &req.parameters,
        req.strikes,
        req.spots,
        req.fixed_strike,
    )?;
    Ok(Json(resp))
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
    ServiceUnavailable(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Pricing(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
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

//! Database access for the calibration pipeline.
//!
//! Connects to the TimescaleDB instance via `DATABASE_URL` and reads option
//! quotes from the `options_data` hypertable. The pool is optional: pricing
//! endpoints work without a database, only calibration requires one.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::postgres::{PgPool, PgPoolOptions};

/// Shared application state for the Axum router.
#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
}

/// Try to connect to Postgres using `DATABASE_URL`. Returns `None` (and logs
/// a warning) if the variable is unset or the connection fails, so the API
/// can still serve pricing requests.
pub async fn try_connect() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("DATABASE_URL not set; calibration endpoints will be unavailable");
            return None;
        }
    };

    match PgPoolOptions::new().max_connections(5).connect(&url).await {
        Ok(pool) => Some(pool),
        Err(e) => {
            eprintln!(
                "Failed to connect to database: {e}; calibration endpoints will be unavailable"
            );
            None
        }
    }
}

/// One option quote row from `options_data`.
#[derive(Debug, sqlx::FromRow)]
pub struct OptionQuoteRow {
    pub expiry: DateTime<Utc>,
    pub strike: Decimal,
    pub option_type: String,
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
}

/// Fetch the latest quote per (expiry, strike, option_type) for a symbol as
/// of the given timestamp, restricted to unexpired contracts.
pub async fn fetch_latest_quotes(
    pool: &PgPool,
    symbol: &str,
    as_of: DateTime<Utc>,
) -> Result<Vec<OptionQuoteRow>, sqlx::Error> {
    sqlx::query_as::<_, OptionQuoteRow>(
        r#"
        SELECT DISTINCT ON (expiry, strike, option_type)
               expiry, strike, option_type, bid, ask
        FROM options_data
        WHERE symbol = $1 AND time <= $2 AND expiry > $2
        ORDER BY expiry, strike, option_type, time DESC
        "#,
    )
    .bind(symbol)
    .bind(as_of)
    .fetch_all(pool)
    .await
}

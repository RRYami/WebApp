use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers;

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/price/european-option", post(handlers::price_european_option))
        .route("/price/american-option", post(handlers::price_american_option))
        .route("/greeks/european-option", post(handlers::greeks_european_option))
}

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers;

pub fn create_router() -> Router {
    Router::new()
        // Legacy endpoints (preserved for backward compatibility)
        .route("/health", get(handlers::health))
        .route("/price/european-option", post(handlers::price_european_option))
        .route("/price/american-option", post(handlers::price_american_option))
        .route("/price/baw-american-option", post(handlers::price_baw_american_option))
        .route("/price/curve", post(handlers::price_curve))
        .route("/greeks/european-option", post(handlers::greeks_european_option))
        .route("/greeks/american-option", post(handlers::greeks_american_option))
        .route("/greeks/curve", post(handlers::greeks_curve))
        .route("/greeks/second-order/european-option", post(handlers::second_order_greeks_european_option))
        .route("/greeks/second-order/american-option", post(handlers::second_order_greeks_american_option))
        // Generic product-driven endpoints
        .route("/products", get(handlers::product_catalog))
        .route("/analytics/price", post(handlers::generic_price))
        .route("/analytics/greeks", post(handlers::generic_greeks))
        .route("/analytics/second-order-greeks", post(handlers::generic_second_order_greeks))
        .route("/analytics/curve/price", post(handlers::generic_price_curve))
        .route("/analytics/curve/greeks", post(handlers::generic_greeks_curve))
        .route("/analytics/curve/second-order-greeks", post(handlers::generic_second_order_greeks_curve))
}

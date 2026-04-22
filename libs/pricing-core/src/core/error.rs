//! Error types for the pricing library.

use thiserror::Error;

/// Result type alias for the pricing library.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the pricing library.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    /// Invalid input parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Arithmetic overflow or underflow
    #[error("Arithmetic error: {0}")]
    Arithmetic(String),

    /// Currency mismatch
    #[error("Currency mismatch: expected {expected}, got {actual}")]
    CurrencyMismatch { expected: String, actual: String },

    /// Invalid date
    #[error("Invalid date: {0}")]
    InvalidDate(String),

    /// Pricing model error
    #[error("Pricing error: {0}")]
    Pricing(String),

    /// Not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create an invalid input error.
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        Error::InvalidInput(msg.into())
    }

    /// Create an arithmetic error.
    pub fn arithmetic<S: Into<String>>(msg: S) -> Self {
        Error::Arithmetic(msg.into())
    }

    /// Create a currency mismatch error.
    pub fn currency_mismatch<S: Into<String>>(expected: S, actual: S) -> Self {
        Error::CurrencyMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a pricing error.
    pub fn pricing<S: Into<String>>(msg: S) -> Self {
        Error::Pricing(msg.into())
    }
}

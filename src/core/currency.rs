//! Currency types and definitions.

use crate::core::error::{Error, Result};
use std::fmt;
use std::str::FromStr;

/// ISO 4217 currency code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurrencyCode([u8; 3]);

impl CurrencyCode {
    /// Create a new currency code from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not exactly 3 uppercase alphabetic characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use pricing_lib::core::currency::CurrencyCode;
    ///
    /// let usd = CurrencyCode::new("USD").unwrap();
    /// ```
    pub fn new(code: &str) -> Result<Self> {
        if code.len() != 3 {
            return Err(Error::invalid_input(format!(
                "Currency code must be exactly 3 characters, got '{}'",
                code
            )));
        }

        if !code
            .chars()
            .all(|c| c.is_ascii_alphabetic() && c.is_ascii_uppercase())
        {
            return Err(Error::invalid_input(format!(
                "Currency code must be uppercase letters, got '{}'",
                code
            )));
        }

        let bytes = code.as_bytes();
        Ok(CurrencyCode([bytes[0], bytes[1], bytes[2]]))
    }

    /// Get the currency code as a string slice.
    pub fn as_str(&self) -> &str {
        // SAFETY: We ensure the bytes are valid ASCII when constructing
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }

    /// Predefined USD currency code
    pub const USD: Self = CurrencyCode([b'U', b'S', b'D']);
    /// Predefined EUR currency code
    pub const EUR: Self = CurrencyCode([b'E', b'U', b'R']);
    /// Predefined GBP currency code
    pub const GBP: Self = CurrencyCode([b'G', b'B', b'P']);
    /// Predefined JPY currency code
    pub const JPY: Self = CurrencyCode([b'J', b'P', b'Y']);
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CurrencyCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        CurrencyCode::new(s)
    }
}

impl AsRef<str> for CurrencyCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Currency definition with additional metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Currency {
    code: CurrencyCode,
    name: String,
    symbol: String,
    decimal_places: u32,
}

impl Currency {
    /// Create a new currency definition.
    pub fn new<S: Into<String>>(
        code: CurrencyCode,
        name: S,
        symbol: S,
        decimal_places: u32,
    ) -> Self {
        Self {
            code,
            name: name.into(),
            symbol: symbol.into(),
            decimal_places,
        }
    }

    /// Get the currency code.
    pub fn code(&self) -> CurrencyCode {
        self.code
    }

    /// Get the currency name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the currency symbol.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get the number of decimal places.
    pub fn decimal_places(&self) -> u32 {
        self.decimal_places
    }

    /// Predefined USD currency
    pub fn usd() -> Self {
        Self::new(CurrencyCode::USD, "US Dollar", "$", 2)
    }

    /// Predefined EUR currency
    pub fn eur() -> Self {
        Self::new(CurrencyCode::EUR, "Euro", "€", 2)
    }

    /// Predefined GBP currency
    pub fn gbp() -> Self {
        Self::new(CurrencyCode::GBP, "British Pound", "£", 2)
    }

    /// Predefined JPY currency
    pub fn jpy() -> Self {
        Self::new(CurrencyCode::JPY, "Japanese Yen", "¥", 0)
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.code, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_code_new() {
        let usd = CurrencyCode::new("USD").unwrap();
        assert_eq!(usd.as_str(), "USD");

        let eur = CurrencyCode::from_str("EUR").unwrap();
        assert_eq!(eur.as_str(), "EUR");
    }

    #[test]
    fn test_currency_code_invalid() {
        assert!(CurrencyCode::new("US").is_err());
        assert!(CurrencyCode::new("USDD").is_err());
        assert!(CurrencyCode::new("usd").is_err());
        assert!(CurrencyCode::new("US1").is_err());
    }

    #[test]
    fn test_currency_constants() {
        assert_eq!(CurrencyCode::USD.as_str(), "USD");
        assert_eq!(CurrencyCode::EUR.as_str(), "EUR");
        assert_eq!(CurrencyCode::GBP.as_str(), "GBP");
        assert_eq!(CurrencyCode::JPY.as_str(), "JPY");
    }

    #[test]
    fn test_currency_display() {
        let usd = Currency::usd();
        assert_eq!(format!("{}", usd), "USD (US Dollar)");
    }
}

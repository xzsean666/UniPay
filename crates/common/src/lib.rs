use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CommonError {
    #[error("currency must be a three-letter uppercase ISO 4217 code")]
    InvalidCurrency,
    #[error("amount_minor must be greater than zero")]
    InvalidAmount,
    #[error("identifier cannot be empty")]
    EmptyIdentifier,
}

pub type CommonResult<T> = Result<T, CommonError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    pub fn new(value: impl Into<String>) -> CommonResult<Self> {
        let value = value.into();
        let valid = value.len() == 3
            && value
                .chars()
                .all(|character| character.is_ascii_uppercase());
        if !valid {
            return Err(CommonError::InvalidCurrency);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub currency: CurrencyCode,
    pub amount_minor: i64,
}

impl Money {
    pub fn new(currency: CurrencyCode, amount_minor: i64) -> CommonResult<Self> {
        if amount_minor <= 0 {
            return Err(CommonError::InvalidAmount);
        }
        Ok(Self {
            currency,
            amount_minor,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(value: impl Into<String>) -> CommonResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(CommonError::EmptyIdentifier);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub type MerchantOrderId = NonEmptyString;
pub type MerchantRefundId = NonEmptyString;

pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", Uuid::new_v4().simple())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_currency_code() {
        assert!(CurrencyCode::new("CNY").is_ok());
        assert!(CurrencyCode::new("cny").is_err());
        assert!(CurrencyCode::new("CN").is_err());
    }

    #[test]
    fn rejects_non_positive_money() {
        let currency = CurrencyCode::new("CNY").unwrap();
        assert!(Money::new(currency.clone(), 1).is_ok());
        assert!(Money::new(currency, 0).is_err());
    }
}

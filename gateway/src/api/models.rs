use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::error::{ApiError, ErrorCode};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Wechat,
    Alipay,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentChannel {
    Native,
    Web,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Closed,
    Refunding,
    PartiallyRefunded,
    Refunded,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefundStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Closed,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentActionType {
    QrCodeUrl,
    RedirectUrl,
    HtmlForm,
    SdkPayload,
    None,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Money {
    pub currency: String,
    pub amount_minor: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PaymentAction {
    #[serde(rename = "type")]
    pub action_type: PaymentActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreatePaymentRequest {
    pub provider: String,
    pub merchant_order_id: String,
    pub amount: Money,
    pub subject: String,
    #[serde(default)]
    pub description: Option<String>,
    pub channel: String,
    #[serde(default)]
    pub notify_url: Option<String>,
    #[serde(default)]
    pub expire_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateRefundRequest {
    pub provider: String,
    pub merchant_order_id: String,
    pub merchant_refund_id: String,
    pub amount: Money,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub notify_url: Option<String>,
    #[serde(default)]
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Payment {
    pub payment_id: String,
    pub provider: Provider,
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
    pub status: PaymentStatus,
    pub amount: Money,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_action: Option<PaymentAction>,
    pub expires_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Refund {
    pub refund_id: String,
    pub provider: Provider,
    pub merchant_order_id: String,
    pub merchant_refund_id: String,
    pub provider_refund_id: Option<String>,
    pub status: RefundStatus,
    pub amount: Money,
    pub succeeded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ValidatedCreatePayment {
    pub provider: Provider,
    pub merchant_order_id: String,
    pub amount: Money,
    pub subject: String,
    pub description: Option<String>,
    pub channel: PaymentChannel,
    pub notify_url: Option<String>,
    pub expire_at: Option<DateTime<FixedOffset>>,
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ValidatedCreateRefund {
    pub provider: Provider,
    pub merchant_order_id: String,
    pub merchant_refund_id: String,
    pub amount: Money,
    pub reason: Option<String>,
    pub notify_url: Option<String>,
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SuccessEnvelope<T> {
    pub success: bool,
    pub data: T,
    pub trace_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorEnvelope {
    pub success: bool,
    pub error: crate::error::ErrorDetail,
    pub trace_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebhookKind {
    Payment,
    Refund,
}

impl CreatePaymentRequest {
    pub fn validate(self) -> Result<ValidatedCreatePayment, ApiError> {
        let operation = "create_payment";
        let provider = parse_provider(&self.provider, operation)?;
        let channel = parse_channel(&self.channel, operation)?;
        validate_provider_channel(provider, channel, operation)?;
        validate_money(&self.amount, operation)?;
        validate_non_empty_len(&self.merchant_order_id, "merchant_order_id", 64, operation)?;
        validate_non_empty_len(&self.subject, "subject", 128, operation)?;
        validate_optional_len(self.description.as_deref(), "description", 256, operation)?;
        validate_optional_url(self.notify_url.as_deref(), "notify_url", operation)?;

        Ok(ValidatedCreatePayment {
            provider,
            merchant_order_id: self.merchant_order_id,
            amount: self.amount,
            subject: self.subject,
            description: self.description,
            channel,
            notify_url: self.notify_url,
            expire_at: self.expire_at,
            metadata: self.metadata,
        })
    }
}

impl CreateRefundRequest {
    pub fn validate(self) -> Result<ValidatedCreateRefund, ApiError> {
        let operation = "create_refund";
        let provider = parse_provider(&self.provider, operation)?;
        validate_money(&self.amount, operation)?;
        validate_non_empty_len(&self.merchant_order_id, "merchant_order_id", 64, operation)?;
        validate_non_empty_len(
            &self.merchant_refund_id,
            "merchant_refund_id",
            64,
            operation,
        )?;
        validate_optional_len(self.reason.as_deref(), "reason", 256, operation)?;
        validate_optional_url(self.notify_url.as_deref(), "notify_url", operation)?;

        Ok(ValidatedCreateRefund {
            provider,
            merchant_order_id: self.merchant_order_id,
            merchant_refund_id: self.merchant_refund_id,
            amount: self.amount,
            reason: self.reason,
            notify_url: self.notify_url,
            metadata: self.metadata,
        })
    }
}

impl FromStr for Provider {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "wechat" => Ok(Self::Wechat),
            "alipay" => Ok(Self::Alipay),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wechat => formatter.write_str("wechat"),
            Self::Alipay => formatter.write_str("alipay"),
        }
    }
}

impl FromStr for PaymentChannel {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "native" => Ok(Self::Native),
            "web" => Ok(Self::Web),
            _ => Err(()),
        }
    }
}

impl fmt::Display for PaymentChannel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native => formatter.write_str("native"),
            Self::Web => formatter.write_str("web"),
        }
    }
}

pub fn parse_provider(value: &str, operation: &str) -> Result<Provider, ApiError> {
    Provider::from_str(value).map_err(|()| ApiError::new(ErrorCode::InvalidProvider, operation))
}

pub fn parse_provider_query(value: Option<String>, operation: &str) -> Result<Provider, ApiError> {
    let value = value
        .filter(|provider| !provider.trim().is_empty())
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidProvider, operation))?;
    parse_provider(&value, operation)
}

fn parse_channel(value: &str, operation: &str) -> Result<PaymentChannel, ApiError> {
    PaymentChannel::from_str(value)
        .map_err(|()| ApiError::new(ErrorCode::InvalidChannel, operation))
}

fn validate_provider_channel(
    provider: Provider,
    channel: PaymentChannel,
    operation: &str,
) -> Result<(), ApiError> {
    let supported = matches!(
        (provider, channel),
        (Provider::Wechat, PaymentChannel::Native) | (Provider::Alipay, PaymentChannel::Web)
    );

    if supported {
        Ok(())
    } else {
        Err(ApiError::new(ErrorCode::InvalidChannel, operation).with_provider(provider))
    }
}

fn validate_money(money: &Money, operation: &str) -> Result<(), ApiError> {
    if money.amount_minor <= 0 {
        return Err(ApiError::new(ErrorCode::InvalidAmount, operation));
    }

    if money.currency != "CNY" {
        return Err(ApiError::new(ErrorCode::InvalidCurrency, operation));
    }

    Ok(())
}

fn validate_non_empty_len(
    value: &str,
    field: &str,
    max_len: usize,
    operation: &str,
) -> Result<(), ApiError> {
    if value.trim().is_empty() {
        return Err(ApiError::invalid_request(
            format!("{field} must not be empty."),
            operation,
        ));
    }

    if value.chars().count() > max_len {
        return Err(ApiError::invalid_request(
            format!("{field} must be at most {max_len} characters."),
            operation,
        ));
    }

    Ok(())
}

fn validate_optional_len(
    value: Option<&str>,
    field: &str,
    max_len: usize,
    operation: &str,
) -> Result<(), ApiError> {
    if let Some(value) = value
        && value.chars().count() > max_len
    {
        return Err(ApiError::invalid_request(
            format!("{field} must be at most {max_len} characters."),
            operation,
        ));
    }

    Ok(())
}

fn validate_optional_url(
    value: Option<&str>,
    field: &str,
    operation: &str,
) -> Result<(), ApiError> {
    if let Some(value) = value
        && Url::parse(value).is_err()
    {
        return Err(ApiError::invalid_request(
            format!("{field} must be an absolute URL."),
            operation,
        ));
    }

    Ok(())
}

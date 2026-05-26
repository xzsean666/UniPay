use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidRequest,
    InvalidAmount,
    InvalidProvider,
    InvalidChannel,
    InvalidCurrency,
    MissingIdempotencyKey,
    Unauthorized,
    Forbidden,
    PaymentNotFound,
    RefundNotFound,
    IdempotencyConflict,
    PaymentStateConflict,
    RefundStateConflict,
    DuplicateProviderOrder,
    RefundAmountExceeded,
    PaymentExpired,
    ProviderRejected,
    ProviderRateLimited,
    SignatureCreateFailed,
    SignatureVerifyFailed,
    WebhookPayloadInvalid,
    WebhookReplaySuspected,
    ProviderTimeout,
    ProviderUnavailable,
    ProviderBadResponse,
    TransportError,
    ConfigurationError,
    SecretUnavailable,
    DatabaseUnavailable,
    InternalError,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::InvalidAmount => "INVALID_AMOUNT",
            Self::InvalidProvider => "INVALID_PROVIDER",
            Self::InvalidChannel => "INVALID_CHANNEL",
            Self::InvalidCurrency => "INVALID_CURRENCY",
            Self::MissingIdempotencyKey => "MISSING_IDEMPOTENCY_KEY",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::PaymentNotFound => "PAYMENT_NOT_FOUND",
            Self::RefundNotFound => "REFUND_NOT_FOUND",
            Self::IdempotencyConflict => "IDEMPOTENCY_CONFLICT",
            Self::PaymentStateConflict => "PAYMENT_STATE_CONFLICT",
            Self::RefundStateConflict => "REFUND_STATE_CONFLICT",
            Self::DuplicateProviderOrder => "DUPLICATE_PROVIDER_ORDER",
            Self::RefundAmountExceeded => "REFUND_AMOUNT_EXCEEDED",
            Self::PaymentExpired => "PAYMENT_EXPIRED",
            Self::ProviderRejected => "PROVIDER_REJECTED",
            Self::ProviderRateLimited => "PROVIDER_RATE_LIMITED",
            Self::SignatureCreateFailed => "SIGNATURE_CREATE_FAILED",
            Self::SignatureVerifyFailed => "SIGNATURE_VERIFY_FAILED",
            Self::WebhookPayloadInvalid => "WEBHOOK_PAYLOAD_INVALID",
            Self::WebhookReplaySuspected => "WEBHOOK_REPLAY_SUSPECTED",
            Self::ProviderTimeout => "PROVIDER_TIMEOUT",
            Self::ProviderUnavailable => "PROVIDER_UNAVAILABLE",
            Self::ProviderBadResponse => "PROVIDER_BAD_RESPONSE",
            Self::TransportError => "TRANSPORT_ERROR",
            Self::ConfigurationError => "CONFIGURATION_ERROR",
            Self::SecretUnavailable => "SECRET_UNAVAILABLE",
            Self::DatabaseUnavailable => "DATABASE_UNAVAILABLE",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("{code:?}: {message}")]
pub struct UnipayError {
    pub code: ErrorCode,
    pub message: String,
    pub provider: Option<String>,
    pub operation: Option<String>,
    pub retryable: bool,
}

impl UnipayError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let retryable = matches!(
            code,
            ErrorCode::ProviderRateLimited
                | ErrorCode::ProviderTimeout
                | ErrorCode::ProviderUnavailable
                | ErrorCode::TransportError
                | ErrorCode::DatabaseUnavailable
        );
        Self {
            code,
            message: message.into(),
            provider: None,
            operation: None,
            retryable,
        }
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
}

pub type Result<T> = std::result::Result<T, UnipayError>;

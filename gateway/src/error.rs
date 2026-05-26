use axum::http::StatusCode;
use serde::Serialize;

use crate::api::models::Provider;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug)]
pub struct ApiError {
    code: ErrorCode,
    status: StatusCode,
    message: String,
    provider: Option<Provider>,
    retryable: bool,
    operation: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}

impl ApiError {
    pub fn new(code: ErrorCode, operation: impl Into<String>) -> Self {
        Self {
            code,
            status: code.http_status(),
            message: code.default_message().to_owned(),
            provider: None,
            retryable: code.default_retryable(),
            operation: Some(operation.into()),
        }
    }

    pub fn invalid_request(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidRequest, operation).with_message(message)
    }

    pub fn unauthorized() -> Self {
        Self::new(ErrorCode::Unauthorized, "authenticate")
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn to_detail(&self) -> ErrorDetail {
        ErrorDetail {
            code: self.code,
            message: self.message.clone(),
            provider: self.provider,
            retryable: self.retryable,
            operation: self.operation.clone(),
        }
    }
}

impl ErrorCode {
    pub fn http_status(self) -> StatusCode {
        match self {
            Self::InvalidRequest
            | Self::InvalidAmount
            | Self::InvalidProvider
            | Self::InvalidChannel
            | Self::InvalidCurrency
            | Self::MissingIdempotencyKey
            | Self::SignatureVerifyFailed
            | Self::WebhookPayloadInvalid
            | Self::WebhookReplaySuspected => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::PaymentNotFound | Self::RefundNotFound => StatusCode::NOT_FOUND,
            Self::IdempotencyConflict
            | Self::PaymentStateConflict
            | Self::RefundStateConflict
            | Self::DuplicateProviderOrder => StatusCode::CONFLICT,
            Self::RefundAmountExceeded | Self::PaymentExpired | Self::ProviderRejected => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::ProviderRateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::ProviderBadResponse => StatusCode::BAD_GATEWAY,
            Self::ProviderTimeout => StatusCode::GATEWAY_TIMEOUT,
            Self::ProviderUnavailable | Self::TransportError | Self::DatabaseUnavailable => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            Self::SignatureCreateFailed
            | Self::ConfigurationError
            | Self::SecretUnavailable
            | Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn default_retryable(self) -> bool {
        matches!(
            self,
            Self::ProviderRateLimited
                | Self::ProviderTimeout
                | Self::ProviderUnavailable
                | Self::TransportError
                | Self::DatabaseUnavailable
        )
    }

    pub fn default_message(self) -> &'static str {
        match self {
            Self::InvalidRequest => "Request body, query, or path is malformed.",
            Self::InvalidAmount => "Amount is zero, negative, malformed, or unsupported.",
            Self::InvalidProvider => "Provider is missing or unsupported.",
            Self::InvalidChannel => "Channel is not supported by provider.",
            Self::InvalidCurrency => "Currency is unsupported by provider or route.",
            Self::MissingIdempotencyKey => "Idempotency key is required.",
            Self::Unauthorized => "API key is missing or invalid.",
            Self::Forbidden => "Caller is authenticated but not allowed.",
            Self::PaymentNotFound => "Payment does not exist in UniPay ledger.",
            Self::RefundNotFound => "Refund does not exist in UniPay ledger.",
            Self::IdempotencyConflict => {
                "Same idempotency key was reused with different parameters."
            }
            Self::PaymentStateConflict => "Operation is invalid for current payment state.",
            Self::RefundStateConflict => "Operation is invalid for current refund state.",
            Self::DuplicateProviderOrder => "Provider reports merchant order id already used.",
            Self::RefundAmountExceeded => "Refund amount exceeds refundable amount.",
            Self::PaymentExpired => "Payment can no longer be completed.",
            Self::ProviderRejected => "Payment provider rejected the request.",
            Self::ProviderRateLimited => "Payment provider rate limit exceeded.",
            Self::SignatureCreateFailed => "UniPay failed to sign provider request.",
            Self::SignatureVerifyFailed => "Signature verification failed.",
            Self::WebhookPayloadInvalid => "Webhook body cannot be parsed after verification.",
            Self::WebhookReplaySuspected => "Webhook replay is suspected.",
            Self::ProviderTimeout => "Payment provider timed out.",
            Self::ProviderUnavailable => "Payment provider is temporarily unavailable.",
            Self::ProviderBadResponse => "Payment provider returned an invalid response.",
            Self::TransportError => "Provider network transport failed.",
            Self::ConfigurationError => "UniPay provider or gateway config is invalid.",
            Self::SecretUnavailable => "Required secret or key cannot be loaded.",
            Self::DatabaseUnavailable => "Ledger or idempotency store is unavailable.",
            Self::InternalError => "Unexpected UniPay failure.",
        }
    }
}

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    CreatePayment,
    QueryPayment,
    CreateRefund,
    QueryRefund,
    PaymentWebhook,
    RefundWebhook,
}

impl Operation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreatePayment => "create_payment",
            Self::QueryPayment => "query_payment",
            Self::CreateRefund => "create_refund",
            Self::QueryRefund => "query_refund",
            Self::PaymentWebhook => "payment_webhook",
            Self::RefundWebhook => "refund_webhook",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniPayErrorCode {
    InvalidRequest,
    InvalidAmount,
    InvalidChannel,
    InvalidCurrency,
    DuplicateProviderOrder,
    RefundAmountExceeded,
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
    InternalError,
}

impl UniPayErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::InvalidAmount => "INVALID_AMOUNT",
            Self::InvalidChannel => "INVALID_CHANNEL",
            Self::InvalidCurrency => "INVALID_CURRENCY",
            Self::DuplicateProviderOrder => "DUPLICATE_PROVIDER_ORDER",
            Self::RefundAmountExceeded => "REFUND_AMOUNT_EXCEEDED",
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
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderError {
    pub code: UniPayErrorCode,
    pub provider: &'static str,
    pub operation: Operation,
    pub retryable: bool,
    pub message: String,
    pub provider_code: Option<String>,
}

impl ProviderError {
    pub fn new(
        code: UniPayErrorCode,
        operation: Operation,
        retryable: bool,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            provider: "wechat",
            operation,
            retryable,
            message: message.into(),
            provider_code: None,
        }
    }

    pub fn with_provider_code(mut self, provider_code: impl Into<String>) -> Self {
        self.provider_code = Some(provider_code.into());
        self
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} during {}: {}",
            self.code.as_str(),
            self.operation.as_str(),
            self.message
        )
    }
}

impl std::error::Error for ProviderError {}

pub type ProviderResult<T> = Result<T, ProviderError>;

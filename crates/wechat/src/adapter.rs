use crate::config::WechatPayConfig;
use crate::error::{Operation, ProviderError, UniPayErrorCode};
use crate::mapping::wechat_amount_from_money;
use async_trait::async_trait;
use chrono::Utc;
use unipay_common::new_id;
use unipay_core::{
    CreatePaymentRequest, CreatePaymentResponse, CreateRefundRequest, PaymentAction,
    PaymentProvider, PaymentQuery, PaymentRecord, PaymentStatus, Provider, RefundQuery,
    RefundRecord, RefundResponse, RefundStatus, Result, UnipayError, WebhookEvent,
    WebhookEventKind, WebhookVerificationRequest,
};

#[derive(Debug, Clone)]
pub struct WechatRequestContext {
    pub trace_id: String,
}

#[derive(Debug, Clone)]
pub struct WechatPayAdapter {
    config: WechatPayConfig,
}

impl WechatPayAdapter {
    pub fn new(config: WechatPayConfig) -> Self {
        Self { config }
    }

    fn provider_error_to_core(error: ProviderError) -> UnipayError {
        UnipayError::new(error.code.into(), error.message)
            .with_provider("wechat")
            .with_operation(error.operation.as_str())
            .retryable(error.retryable)
    }
}

#[async_trait]
impl PaymentProvider for WechatPayAdapter {
    fn provider(&self) -> Provider {
        Provider::Wechat
    }

    async fn create_payment(&self, request: CreatePaymentRequest) -> Result<CreatePaymentResponse> {
        wechat_amount_from_money(&request.amount, Operation::CreatePayment)
            .map_err(Self::provider_error_to_core)?;
        let now = Utc::now();
        let code_url = format!(
            "weixin://wxpay/bizpayurl?mch_id={}&out_trade_no={}",
            self.config.merchant_id, request.merchant_order_id
        );

        Ok(CreatePaymentResponse {
            payment_id: new_id("pay"),
            provider: Provider::Wechat,
            merchant_order_id: request.merchant_order_id,
            provider_transaction_id: None,
            status: PaymentStatus::Pending,
            amount: request.amount,
            payment_action: PaymentAction::QrCodeUrl { value: code_url },
            expires_at: request.expire_at,
            created_at: now,
        })
    }

    async fn query_payment(&self, query: PaymentQuery) -> Result<PaymentRecord> {
        let now = Utc::now();
        Ok(PaymentRecord {
            payment_id: new_id("pay"),
            provider: Provider::Wechat,
            merchant_order_id: query.merchant_order_id,
            provider_transaction_id: query.provider_transaction_id,
            channel: unipay_core::PaymentChannel::Native,
            amount: unipay_common::Money::new(
                unipay_common::CurrencyCode::new("CNY").expect("static currency is valid"),
                1,
            )
            .expect("static amount is valid"),
            status: PaymentStatus::Unknown,
            subject: "provider query placeholder".to_owned(),
            description: None,
            payment_action: None,
            paid_at: None,
            expires_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    async fn create_refund(&self, request: CreateRefundRequest) -> Result<RefundResponse> {
        wechat_amount_from_money(&request.amount, Operation::CreateRefund)
            .map_err(Self::provider_error_to_core)?;
        Ok(RefundResponse {
            refund_id: new_id("rfd"),
            provider: Provider::Wechat,
            merchant_order_id: request.merchant_order_id,
            merchant_refund_id: request.merchant_refund_id,
            provider_refund_id: Some(new_id("wxrfd")),
            status: RefundStatus::Processing,
            amount: request.amount,
            created_at: Utc::now(),
        })
    }

    async fn query_refund(&self, query: RefundQuery) -> Result<RefundRecord> {
        let now = Utc::now();
        Ok(RefundRecord {
            refund_id: new_id("rfd"),
            payment_id: new_id("pay"),
            provider: Provider::Wechat,
            merchant_order_id: "unknown".to_owned(),
            merchant_refund_id: query.merchant_refund_id,
            provider_refund_id: query.provider_refund_id,
            amount: unipay_common::Money::new(
                unipay_common::CurrencyCode::new("CNY").expect("static currency is valid"),
                1,
            )
            .expect("static amount is valid"),
            status: RefundStatus::Unknown,
            reason: None,
            succeeded_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    async fn verify_payment_webhook(
        &self,
        request: WebhookVerificationRequest,
    ) -> Result<WebhookEvent> {
        require_configured_webhook_verifier(&request, "payment_webhook")?;
        let payload: serde_json::Value =
            serde_json::from_slice(&request.raw_body).map_err(|_| {
                UnipayError::new(
                    unipay_core::ErrorCode::WebhookPayloadInvalid,
                    "WeChat payment webhook payload is invalid JSON",
                )
                .with_provider("wechat")
                .with_operation("payment_webhook")
            })?;
        let deduplication_key = payload
            .get("id")
            .and_then(|value| value.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| unipay_signing_fallback_hash(&request.raw_body));

        Ok(WebhookEvent {
            provider: Provider::Wechat,
            kind: WebhookEventKind::Payment,
            deduplication_key,
            merchant_order_id: None,
            merchant_refund_id: None,
            provider_transaction_id: None,
            provider_refund_id: None,
            payment_status: Some(PaymentStatus::Unknown),
            refund_status: None,
            occurred_at: Some(request.received_at),
            payload,
        })
    }

    async fn verify_refund_webhook(
        &self,
        request: WebhookVerificationRequest,
    ) -> Result<WebhookEvent> {
        require_configured_webhook_verifier(&request, "refund_webhook")?;
        let payload: serde_json::Value =
            serde_json::from_slice(&request.raw_body).map_err(|_| {
                UnipayError::new(
                    unipay_core::ErrorCode::WebhookPayloadInvalid,
                    "WeChat refund webhook payload is invalid JSON",
                )
                .with_provider("wechat")
                .with_operation("refund_webhook")
            })?;
        let deduplication_key = payload
            .get("id")
            .and_then(|value| value.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| unipay_signing_fallback_hash(&request.raw_body));

        Ok(WebhookEvent {
            provider: Provider::Wechat,
            kind: WebhookEventKind::Refund,
            deduplication_key,
            merchant_order_id: None,
            merchant_refund_id: None,
            provider_transaction_id: None,
            provider_refund_id: None,
            payment_status: None,
            refund_status: Some(RefundStatus::Unknown),
            occurred_at: Some(request.received_at),
            payload,
        })
    }
}

fn require_configured_webhook_verifier(
    request: &WebhookVerificationRequest,
    operation: &'static str,
) -> Result<()> {
    let has_signature_headers = request.headers.contains_key("wechatpay-timestamp")
        && request.headers.contains_key("wechatpay-nonce")
        && request.headers.contains_key("wechatpay-signature")
        && request.headers.contains_key("wechatpay-serial");

    let message = if has_signature_headers {
        "WeChat Pay webhook signature verification is not configured"
    } else {
        "WeChat Pay webhook signature headers are missing"
    };

    Err(
        UnipayError::new(unipay_core::ErrorCode::SignatureVerifyFailed, message)
            .with_provider("wechat")
            .with_operation(operation),
    )
}

fn unipay_signing_fallback_hash(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl From<UniPayErrorCode> for unipay_core::ErrorCode {
    fn from(value: UniPayErrorCode) -> Self {
        match value {
            UniPayErrorCode::InvalidRequest => Self::InvalidRequest,
            UniPayErrorCode::InvalidAmount => Self::InvalidAmount,
            UniPayErrorCode::InvalidChannel => Self::InvalidChannel,
            UniPayErrorCode::InvalidCurrency => Self::InvalidCurrency,
            UniPayErrorCode::DuplicateProviderOrder => Self::DuplicateProviderOrder,
            UniPayErrorCode::RefundAmountExceeded => Self::RefundAmountExceeded,
            UniPayErrorCode::ProviderRejected => Self::ProviderRejected,
            UniPayErrorCode::ProviderRateLimited => Self::ProviderRateLimited,
            UniPayErrorCode::SignatureCreateFailed => Self::SignatureCreateFailed,
            UniPayErrorCode::SignatureVerifyFailed => Self::SignatureVerifyFailed,
            UniPayErrorCode::WebhookPayloadInvalid => Self::WebhookPayloadInvalid,
            UniPayErrorCode::WebhookReplaySuspected => Self::WebhookReplaySuspected,
            UniPayErrorCode::ProviderTimeout => Self::ProviderTimeout,
            UniPayErrorCode::ProviderUnavailable => Self::ProviderUnavailable,
            UniPayErrorCode::ProviderBadResponse => Self::ProviderBadResponse,
            UniPayErrorCode::TransportError => Self::TransportError,
            UniPayErrorCode::ConfigurationError => Self::ConfigurationError,
            UniPayErrorCode::SecretUnavailable => Self::SecretUnavailable,
            UniPayErrorCode::InternalError => Self::InternalError,
        }
    }
}

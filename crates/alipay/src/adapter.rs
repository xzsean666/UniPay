use crate::config::AlipayConfig;
use crate::mapping::alipay_amount_from_money;
use async_trait::async_trait;
use chrono::Utc;
use unipay_common::{new_id, CurrencyCode, Money};
use unipay_core::{
    CreatePaymentRequest, CreatePaymentResponse, CreateRefundRequest, ErrorCode, PaymentAction,
    PaymentChannel, PaymentProvider, PaymentQuery, PaymentRecord, PaymentStatus, Provider,
    RefundQuery, RefundRecord, RefundResponse, RefundStatus, Result, UnipayError, WebhookEvent,
    WebhookEventKind, WebhookVerificationRequest,
};

#[derive(Debug, Clone)]
pub struct AlipayAdapter {
    config: AlipayConfig,
}

impl AlipayAdapter {
    pub fn new(config: AlipayConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PaymentProvider for AlipayAdapter {
    fn provider(&self) -> Provider {
        Provider::Alipay
    }

    async fn create_payment(&self, request: CreatePaymentRequest) -> Result<CreatePaymentResponse> {
        let amount = alipay_amount_from_money(&request.amount)?;
        let form = format!(
            "<form method=\"post\" action=\"{}\"><input name=\"app_id\" value=\"{}\"><input name=\"out_trade_no\" value=\"{}\"><input name=\"total_amount\" value=\"{}\"></form>",
            self.config.api_base_url, self.config.app_id, request.merchant_order_id, amount
        );
        Ok(CreatePaymentResponse {
            payment_id: new_id("pay"),
            provider: Provider::Alipay,
            merchant_order_id: request.merchant_order_id,
            provider_transaction_id: None,
            status: PaymentStatus::Pending,
            amount: request.amount,
            payment_action: PaymentAction::HtmlForm { value: form },
            expires_at: request.expire_at,
            created_at: Utc::now(),
        })
    }

    async fn query_payment(&self, query: PaymentQuery) -> Result<PaymentRecord> {
        let now = Utc::now();
        Ok(PaymentRecord {
            payment_id: new_id("pay"),
            provider: Provider::Alipay,
            merchant_order_id: query.merchant_order_id,
            provider_transaction_id: query.provider_transaction_id,
            channel: PaymentChannel::Web,
            amount: Money::new(
                CurrencyCode::new("CNY").expect("static currency is valid"),
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
        let _ = alipay_amount_from_money(&request.amount)?;
        Ok(RefundResponse {
            refund_id: new_id("rfd"),
            provider: Provider::Alipay,
            merchant_order_id: request.merchant_order_id,
            merchant_refund_id: request.merchant_refund_id,
            provider_refund_id: Some(new_id("alirfd")),
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
            provider: Provider::Alipay,
            merchant_order_id: "unknown".to_owned(),
            merchant_refund_id: query.merchant_refund_id,
            provider_refund_id: query.provider_refund_id,
            amount: Money::new(
                CurrencyCode::new("CNY").expect("static currency is valid"),
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
        parse_webhook(request, WebhookEventKind::Payment)
    }

    async fn verify_refund_webhook(
        &self,
        request: WebhookVerificationRequest,
    ) -> Result<WebhookEvent> {
        require_configured_webhook_verifier(&request, "refund_webhook")?;
        parse_webhook(request, WebhookEventKind::Refund)
    }
}

fn require_configured_webhook_verifier(
    request: &WebhookVerificationRequest,
    operation: &'static str,
) -> Result<()> {
    let has_signature_material = request.headers.contains_key("alipay-signature")
        || request.headers.contains_key("sign")
        || request.headers.contains_key("x-alipay-signature")
        || std::str::from_utf8(&request.raw_body).is_ok_and(|body| body.contains("sign="));

    let message = if has_signature_material {
        "Alipay webhook signature verification is not configured"
    } else {
        "Alipay webhook signature material is missing"
    };

    Err(UnipayError::new(ErrorCode::SignatureVerifyFailed, message)
        .with_provider("alipay")
        .with_operation(operation))
}

fn parse_webhook(
    request: WebhookVerificationRequest,
    kind: WebhookEventKind,
) -> Result<WebhookEvent> {
    let payload: serde_json::Value = serde_json::from_slice(&request.raw_body).map_err(|_| {
        UnipayError::new(
            ErrorCode::WebhookPayloadInvalid,
            "Alipay webhook payload is invalid JSON",
        )
        .with_provider("alipay")
    })?;
    let deduplication_key = payload
        .get("notify_id")
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| fallback_hash(&request.raw_body));

    Ok(WebhookEvent {
        provider: Provider::Alipay,
        kind,
        deduplication_key,
        merchant_order_id: payload
            .get("out_trade_no")
            .and_then(|value| value.as_str())
            .map(str::to_owned),
        merchant_refund_id: payload
            .get("out_request_no")
            .and_then(|value| value.as_str())
            .map(str::to_owned),
        provider_transaction_id: payload
            .get("trade_no")
            .and_then(|value| value.as_str())
            .map(str::to_owned),
        provider_refund_id: None,
        payment_status: None,
        refund_status: None,
        occurred_at: Some(request.received_at),
        payload,
    })
}

fn fallback_hash(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

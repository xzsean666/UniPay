use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::api::models::{
    Payment, PaymentAction, PaymentActionType, PaymentStatus, Provider, Refund, RefundStatus,
    ValidatedCreatePayment, ValidatedCreateRefund, WebhookKind,
};
use crate::error::{ApiError, ErrorCode};

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub caller_id: String,
    pub trace_id: String,
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug)]
pub struct VerifiedWebhook {
    pub provider: Provider,
    pub kind: WebhookKind,
    pub deduplication_key: String,
    pub duplicate: bool,
}

#[async_trait]
pub trait GatewayService: Send + Sync {
    async fn create_payment(
        &self,
        context: RequestContext,
        request: ValidatedCreatePayment,
    ) -> Result<Payment, ApiError>;

    async fn query_payment(
        &self,
        context: RequestContext,
        provider: Provider,
        merchant_order_id: String,
    ) -> Result<Payment, ApiError>;

    async fn create_refund(
        &self,
        context: RequestContext,
        request: ValidatedCreateRefund,
    ) -> Result<Refund, ApiError>;

    async fn query_refund(
        &self,
        context: RequestContext,
        provider: Provider,
        merchant_refund_id: String,
    ) -> Result<Refund, ApiError>;

    async fn receive_webhook(
        &self,
        provider: Provider,
        kind: WebhookKind,
        headers: BTreeMap<String, String>,
        raw_body: Vec<u8>,
    ) -> Result<VerifiedWebhook, ApiError>;
}

#[derive(Default)]
pub struct InMemoryGatewayService {
    state: RwLock<GatewayMemoryState>,
}

#[derive(Default)]
struct GatewayMemoryState {
    payments: HashMap<PaymentKey, PaymentEntry>,
    refunds: HashMap<RefundKey, RefundEntry>,
    idempotency_records: HashMap<IdempotencyKey, IdempotencyEntry>,
    webhook_keys: HashSet<(Provider, String)>,
}

type PaymentKey = (String, Provider, String);
type RefundKey = (String, Provider, String);
type IdempotencyKey = (String, String, String);

#[derive(Clone, Debug)]
struct PaymentEntry {
    request_hash: String,
    payment: Payment,
}

#[derive(Clone, Debug)]
struct RefundEntry {
    request_hash: String,
    refund: Refund,
}

#[derive(Clone, Debug)]
struct IdempotencyEntry {
    request_hash: String,
    resource_id: String,
}

impl InMemoryGatewayService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl GatewayService for InMemoryGatewayService {
    async fn create_payment(
        &self,
        context: RequestContext,
        request: ValidatedCreatePayment,
    ) -> Result<Payment, ApiError> {
        let operation = "create_payment";
        let idempotency_key = required_idempotency_key(&context, operation)?;
        let request_hash = hash_serializable(&request, operation)?;
        let payment_key = (
            context.caller_id.clone(),
            request.provider,
            request.merchant_order_id.clone(),
        );
        let idempotency_map_key = idempotency_map_key(&context, operation, &idempotency_key);

        let mut state = self.state.write().expect("gateway memory state lock");
        if let Some(existing) = state.payments.get(&payment_key) {
            ensure_same_request_hash(&existing.request_hash, &request_hash, operation)?;
            let payment = existing.payment.clone();
            ensure_idempotency_record(
                &mut state,
                idempotency_map_key,
                &request_hash,
                &request.merchant_order_id,
                operation,
            )?;
            return Ok(payment);
        }
        ensure_idempotency_record(
            &mut state,
            idempotency_map_key,
            &request_hash,
            &request.merchant_order_id,
            operation,
        )?;

        let now = Utc::now();
        let payment_action = match request.provider {
            Provider::Wechat => PaymentAction {
                action_type: PaymentActionType::QrCodeUrl,
                value: Some(format!(
                    "weixin://wxpay/bizpayurl?out_trade_no={}",
                    request.merchant_order_id
                )),
                payload: None,
            },
            Provider::Alipay => PaymentAction {
                action_type: PaymentActionType::HtmlForm,
                value: Some(format!(
                    "<form method=\"post\" action=\"https://openapi.alipay.com/gateway.do\"><input name=\"out_trade_no\" value=\"{}\"></form>",
                    request.merchant_order_id
                )),
                payload: None,
            },
        };

        let payment = Payment {
            payment_id: scoped_resource_id(
                "pay",
                &context.caller_id,
                request.provider,
                &request.merchant_order_id,
            ),
            provider: request.provider,
            merchant_order_id: request.merchant_order_id,
            provider_transaction_id: None,
            status: PaymentStatus::Pending,
            amount: request.amount,
            payment_action: Some(payment_action),
            expires_at: request.expire_at.map(|time| time.with_timezone(&Utc)),
            paid_at: None,
            created_at: now,
            updated_at: Some(now),
        };

        state.payments.insert(
            payment_key,
            PaymentEntry {
                request_hash,
                payment: payment.clone(),
            },
        );
        Ok(payment)
    }

    async fn query_payment(
        &self,
        context: RequestContext,
        provider: Provider,
        merchant_order_id: String,
    ) -> Result<Payment, ApiError> {
        self.state
            .read()
            .expect("gateway memory state lock")
            .payments
            .get(&(context.caller_id, provider, merchant_order_id))
            .map(|entry| entry.payment.clone())
            .ok_or_else(|| ApiError::new(ErrorCode::PaymentNotFound, "query_payment"))
    }

    async fn create_refund(
        &self,
        context: RequestContext,
        request: ValidatedCreateRefund,
    ) -> Result<Refund, ApiError> {
        let operation = "create_refund";
        let idempotency_key = required_idempotency_key(&context, operation)?;
        let request_hash = hash_serializable(&request, operation)?;
        let payment_key = (
            context.caller_id.clone(),
            request.provider,
            request.merchant_order_id.clone(),
        );
        let refund_key = (
            context.caller_id.clone(),
            request.provider,
            request.merchant_refund_id.clone(),
        );
        let idempotency_map_key = idempotency_map_key(&context, operation, &idempotency_key);

        let mut state = self.state.write().expect("gateway memory state lock");
        if let Some(existing) = state.refunds.get(&refund_key) {
            ensure_same_request_hash(&existing.request_hash, &request_hash, operation)?;
            let refund = existing.refund.clone();
            ensure_idempotency_record(
                &mut state,
                idempotency_map_key,
                &request_hash,
                &request.merchant_refund_id,
                operation,
            )?;
            return Ok(refund);
        }
        ensure_idempotency_record(
            &mut state,
            idempotency_map_key,
            &request_hash,
            &request.merchant_refund_id,
            operation,
        )?;

        let Some(payment) = state
            .payments
            .get(&payment_key)
            .map(|entry| entry.payment.clone())
        else {
            return Err(ApiError::new(ErrorCode::PaymentNotFound, "create_refund")
                .with_provider(request.provider));
        };

        if !matches!(
            payment.status,
            PaymentStatus::Succeeded | PaymentStatus::PartiallyRefunded
        ) {
            return Err(ApiError::new(ErrorCode::PaymentStateConflict, operation)
                .with_provider(request.provider));
        }

        let refundable_amount = remaining_refundable_amount(&state, &context, &payment);
        if request.amount.currency != payment.amount.currency
            || request.amount.amount_minor > refundable_amount
        {
            return Err(ApiError::new(ErrorCode::RefundAmountExceeded, operation)
                .with_provider(request.provider));
        }

        let now = Utc::now();
        let refund = Refund {
            refund_id: scoped_resource_id(
                "rfd",
                &context.caller_id,
                request.provider,
                &request.merchant_refund_id,
            ),
            provider: request.provider,
            merchant_order_id: request.merchant_order_id,
            merchant_refund_id: request.merchant_refund_id,
            provider_refund_id: None,
            status: RefundStatus::Processing,
            amount: request.amount,
            succeeded_at: None,
            created_at: now,
            updated_at: Some(now),
        };

        state.refunds.insert(
            refund_key,
            RefundEntry {
                request_hash,
                refund: refund.clone(),
            },
        );
        Ok(refund)
    }

    async fn query_refund(
        &self,
        context: RequestContext,
        provider: Provider,
        merchant_refund_id: String,
    ) -> Result<Refund, ApiError> {
        self.state
            .read()
            .expect("gateway memory state lock")
            .refunds
            .get(&(context.caller_id, provider, merchant_refund_id))
            .map(|entry| entry.refund.clone())
            .ok_or_else(|| ApiError::new(ErrorCode::RefundNotFound, "query_refund"))
    }

    async fn receive_webhook(
        &self,
        provider: Provider,
        kind: WebhookKind,
        headers: BTreeMap<String, String>,
        raw_body: Vec<u8>,
    ) -> Result<VerifiedWebhook, ApiError> {
        if raw_body.is_empty() {
            return Err(
                ApiError::new(ErrorCode::WebhookPayloadInvalid, "receive_webhook")
                    .with_provider(provider),
            );
        }
        require_provider_webhook_signature(provider, &headers)?;

        let deduplication_key = headers
            .get("wechatpay-nonce")
            .or_else(|| headers.get("notify_id"))
            .cloned()
            .unwrap_or_else(|| short_hash(&raw_body));
        let key = (provider, deduplication_key.clone());
        let duplicate = !self
            .state
            .write()
            .expect("gateway memory state lock")
            .webhook_keys
            .insert(key);

        Ok(VerifiedWebhook {
            provider,
            kind,
            deduplication_key,
            duplicate,
        })
    }
}

fn required_idempotency_key(
    context: &RequestContext,
    operation: &'static str,
) -> Result<String, ApiError> {
    let Some(key) = context
        .idempotency_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
    else {
        return Err(ApiError::new(ErrorCode::MissingIdempotencyKey, operation));
    };

    if key.chars().count() > 128 {
        return Err(ApiError::invalid_request(
            "Idempotency-Key must be at most 128 characters.",
            operation,
        ));
    }

    Ok(key.to_owned())
}

fn idempotency_map_key(
    context: &RequestContext,
    operation: &'static str,
    key: &str,
) -> IdempotencyKey {
    (
        context.caller_id.clone(),
        operation.to_owned(),
        key.to_owned(),
    )
}

fn ensure_idempotency_record(
    state: &mut GatewayMemoryState,
    key: IdempotencyKey,
    request_hash: &str,
    resource_id: &str,
    operation: &'static str,
) -> Result<(), ApiError> {
    if let Some(existing) = state.idempotency_records.get(&key) {
        ensure_same_request_hash(&existing.request_hash, request_hash, operation)?;
        if existing.resource_id != resource_id {
            return Err(ApiError::new(ErrorCode::IdempotencyConflict, operation));
        }
        return Ok(());
    }

    state.idempotency_records.insert(
        key,
        IdempotencyEntry {
            request_hash: request_hash.to_owned(),
            resource_id: resource_id.to_owned(),
        },
    );
    Ok(())
}

fn ensure_same_request_hash(
    existing_hash: &str,
    request_hash: &str,
    operation: &'static str,
) -> Result<(), ApiError> {
    if existing_hash == request_hash {
        Ok(())
    } else {
        Err(ApiError::new(ErrorCode::IdempotencyConflict, operation))
    }
}

fn hash_serializable<T: Serialize>(value: &T, operation: &'static str) -> Result<String, ApiError> {
    let bytes = serde_json::to_vec(value).map_err(|_| {
        ApiError::new(ErrorCode::InternalError, operation)
            .with_message("UniPay failed to hash request for idempotency.")
    })?;
    Ok(full_hash(&bytes))
}

fn remaining_refundable_amount(
    state: &GatewayMemoryState,
    context: &RequestContext,
    payment: &Payment,
) -> i64 {
    let reserved_refund_amount: i64 = state
        .refunds
        .iter()
        .filter(|(key, entry)| {
            key.0 == context.caller_id
                && key.1 == payment.provider
                && entry.refund.merchant_order_id == payment.merchant_order_id
                && entry.refund.amount.currency == payment.amount.currency
                && refund_reserves_amount(entry.refund.status)
        })
        .map(|(_, entry)| entry.refund.amount.amount_minor)
        .sum();

    payment.amount.amount_minor - reserved_refund_amount
}

fn refund_reserves_amount(status: RefundStatus) -> bool {
    !matches!(status, RefundStatus::Failed | RefundStatus::Closed)
}

fn require_provider_webhook_signature(
    provider: Provider,
    headers: &BTreeMap<String, String>,
) -> Result<(), ApiError> {
    let has_required_signature_material = match provider {
        Provider::Wechat => {
            headers.contains_key("wechatpay-timestamp")
                && headers.contains_key("wechatpay-nonce")
                && headers.contains_key("wechatpay-signature")
                && headers.contains_key("wechatpay-serial")
        }
        Provider::Alipay => {
            headers.contains_key("alipay-signature")
                || headers.contains_key("sign")
                || headers.contains_key("x-alipay-signature")
        }
    };

    if has_required_signature_material {
        return Err(
            ApiError::new(ErrorCode::SignatureVerifyFailed, "receive_webhook")
                .with_provider(provider)
                .with_message(
                    "Provider webhook signature verification is not configured in this gateway.",
                ),
        );
    }

    Err(
        ApiError::new(ErrorCode::SignatureVerifyFailed, "receive_webhook")
            .with_provider(provider)
            .with_message("Provider webhook signature headers are missing."),
    )
}

fn short_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn full_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn scoped_resource_id(
    prefix: &str,
    caller_id: &str,
    provider: Provider,
    merchant_resource_id: &str,
) -> String {
    format!(
        "{prefix}_{}",
        short_hash(format!("{caller_id}:{provider}:{merchant_resource_id}").as_bytes())
    )
}

pub type SharedGatewayService = Arc<dyn GatewayService>;

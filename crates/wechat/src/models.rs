use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub currency: String,
    pub amount_minor: i64,
}

impl Money {
    pub fn cny(amount_minor: i64) -> Self {
        Self {
            currency: "CNY".to_string(),
            amount_minor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentChannel {
    Native,
    Web,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefundStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Closed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum PaymentAction {
    QrCodeUrl(String),
    RedirectUrl(String),
    HtmlForm(String),
    SdkPayload(serde_json::Value),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePaymentRequest {
    pub merchant_order_id: String,
    pub amount: Money,
    pub subject: String,
    pub description: Option<String>,
    pub channel: PaymentChannel,
    pub notify_url: Option<String>,
    pub expire_at: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePaymentResponse {
    pub provider: &'static str,
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
    pub status: PaymentStatus,
    pub amount: Money,
    pub payment_action: PaymentAction,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryPaymentRequest {
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentStatusResult {
    pub provider: &'static str,
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
    pub status: PaymentStatus,
    pub amount: Option<Money>,
    pub paid_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRefundRequest {
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
    pub merchant_refund_id: String,
    pub provider_refund_id: Option<String>,
    pub refund_amount: Money,
    pub payment_amount: Money,
    pub reason: Option<String>,
    pub notify_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundResponse {
    pub provider: &'static str,
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
    pub merchant_refund_id: String,
    pub provider_refund_id: Option<String>,
    pub status: RefundStatus,
    pub refund_amount: Money,
    pub succeeded_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRefundRequest {
    pub merchant_refund_id: String,
    pub provider_refund_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookEventKind {
    Payment,
    Refund,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebhookEvent {
    pub provider: &'static str,
    pub kind: WebhookEventKind,
    pub raw_event_type: String,
    pub deduplication_key: String,
    pub merchant_order_id: Option<String>,
    pub provider_transaction_id: Option<String>,
    pub merchant_refund_id: Option<String>,
    pub provider_refund_id: Option<String>,
    pub payment_status: Option<PaymentStatus>,
    pub refund_status: Option<RefundStatus>,
    pub amount: Option<Money>,
    pub occurred_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawWebhook {
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

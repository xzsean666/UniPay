use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use unipay_common::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Wechat,
    Alipay,
    Stripe,
    Paypal,
    ApplePay,
    GooglePay,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wechat => "wechat",
            Self::Alipay => "alipay",
            Self::Stripe => "stripe",
            Self::Paypal => "paypal",
            Self::ApplePay => "apple_pay",
            Self::GooglePay => "google_pay",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentChannel {
    Native,
    Web,
    H5,
    App,
    JsApi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl PaymentStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Closed | Self::Refunded
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefundStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Closed,
    Unknown,
}

impl RefundStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Closed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentAction {
    QrCodeUrl { value: String },
    RedirectUrl { value: String },
    HtmlForm { value: String },
    SdkPayload { payload: Value },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub provider: Provider,
    pub merchant_order_id: String,
    pub amount: Money,
    pub subject: String,
    pub description: Option<String>,
    pub channel: PaymentChannel,
    pub notify_url: Option<String>,
    pub expire_at: Option<DateTime<Utc>>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatePaymentResponse {
    pub payment_id: String,
    pub provider: Provider,
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
    pub status: PaymentStatus,
    pub amount: Money,
    pub payment_action: PaymentAction,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentQuery {
    pub provider: Provider,
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub payment_id: String,
    pub provider: Provider,
    pub merchant_order_id: String,
    pub provider_transaction_id: Option<String>,
    pub channel: PaymentChannel,
    pub amount: Money,
    pub status: PaymentStatus,
    pub subject: String,
    pub description: Option<String>,
    pub payment_action: Option<PaymentAction>,
    pub paid_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateRefundRequest {
    pub provider: Provider,
    pub merchant_order_id: String,
    pub merchant_refund_id: String,
    pub amount: Money,
    pub reason: Option<String>,
    pub notify_url: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefundResponse {
    pub refund_id: String,
    pub provider: Provider,
    pub merchant_order_id: String,
    pub merchant_refund_id: String,
    pub provider_refund_id: Option<String>,
    pub status: RefundStatus,
    pub amount: Money,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefundQuery {
    pub provider: Provider,
    pub merchant_refund_id: String,
    pub provider_refund_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefundRecord {
    pub refund_id: String,
    pub payment_id: String,
    pub provider: Provider,
    pub merchant_order_id: String,
    pub merchant_refund_id: String,
    pub provider_refund_id: Option<String>,
    pub amount: Money,
    pub status: RefundStatus,
    pub reason: Option<String>,
    pub succeeded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookVerificationRequest {
    pub provider: Provider,
    pub headers: BTreeMap<String, String>,
    pub raw_body: Vec<u8>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventKind {
    Payment,
    Refund,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub provider: Provider,
    pub kind: WebhookEventKind,
    pub deduplication_key: String,
    pub merchant_order_id: Option<String>,
    pub merchant_refund_id: Option<String>,
    pub provider_transaction_id: Option<String>,
    pub provider_refund_id: Option<String>,
    pub payment_status: Option<PaymentStatus>,
    pub refund_status: Option<RefundStatus>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub caller_id: String,
    pub key: String,
    pub operation: String,
    pub request_hash: String,
    pub resource_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRequestRecord {
    pub provider: Provider,
    pub operation: String,
    pub endpoint_path: String,
    pub request_hash: String,
    pub response_status: Option<u16>,
    pub provider_code: Option<String>,
    pub retryable: bool,
    pub trace_id: String,
    pub created_at: DateTime<Utc>,
}

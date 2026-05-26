use crate::models::*;
use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    fn provider(&self) -> Provider;

    async fn create_payment(&self, request: CreatePaymentRequest) -> Result<CreatePaymentResponse>;

    async fn query_payment(&self, query: PaymentQuery) -> Result<PaymentRecord>;

    async fn create_refund(&self, request: CreateRefundRequest) -> Result<RefundResponse>;

    async fn query_refund(&self, query: RefundQuery) -> Result<RefundRecord>;

    async fn verify_payment_webhook(
        &self,
        request: WebhookVerificationRequest,
    ) -> Result<WebhookEvent>;

    async fn verify_refund_webhook(
        &self,
        request: WebhookVerificationRequest,
    ) -> Result<WebhookEvent>;
}

#[async_trait]
pub trait PaymentStorage: Send + Sync {
    async fn insert_payment(&self, payment: PaymentRecord) -> Result<PaymentRecord>;

    async fn update_payment(&self, payment: PaymentRecord) -> Result<PaymentRecord>;

    async fn find_payment(
        &self,
        provider: Provider,
        merchant_order_id: &str,
    ) -> Result<Option<PaymentRecord>>;

    async fn insert_refund(&self, refund: RefundRecord) -> Result<RefundRecord>;

    async fn update_refund(&self, refund: RefundRecord) -> Result<RefundRecord>;

    async fn find_refund(
        &self,
        provider: Provider,
        merchant_refund_id: &str,
    ) -> Result<Option<RefundRecord>>;

    async fn list_refunds_for_payment(
        &self,
        provider: Provider,
        merchant_order_id: &str,
    ) -> Result<Vec<RefundRecord>>;

    async fn record_provider_request(&self, record: ProviderRequestRecord) -> Result<()>;

    async fn record_webhook_event(&self, event: WebhookEvent) -> Result<bool>;
}

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use unipay_core::{
    ErrorCode, PaymentRecord, PaymentStorage, Provider, ProviderRequestRecord, RefundRecord,
    Result, UnipayError, WebhookEvent,
};

#[derive(Default)]
pub struct InMemoryPaymentStorage {
    payments: RwLock<HashMap<(Provider, String), PaymentRecord>>,
    refunds: RwLock<HashMap<(Provider, String), RefundRecord>>,
    provider_requests: RwLock<Vec<ProviderRequestRecord>>,
    webhook_deduplication_keys: RwLock<HashSet<(Provider, String)>>,
    webhook_events: RwLock<Vec<WebhookEvent>>,
}

impl InMemoryPaymentStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn provider_request_count(&self) -> usize {
        self.provider_requests.read().len()
    }

    pub fn webhook_event_count(&self) -> usize {
        self.webhook_events.read().len()
    }
}

#[async_trait]
impl PaymentStorage for InMemoryPaymentStorage {
    async fn insert_payment(&self, payment: PaymentRecord) -> Result<PaymentRecord> {
        let key = (payment.provider, payment.merchant_order_id.clone());
        let mut payments = self.payments.write();
        if payments.contains_key(&key) {
            return Err(UnipayError::new(
                ErrorCode::IdempotencyConflict,
                "payment already exists for merchant order id",
            ));
        }
        payments.insert(key, payment.clone());
        Ok(payment)
    }

    async fn update_payment(&self, payment: PaymentRecord) -> Result<PaymentRecord> {
        let key = (payment.provider, payment.merchant_order_id.clone());
        self.payments.write().insert(key, payment.clone());
        Ok(payment)
    }

    async fn find_payment(
        &self,
        provider: Provider,
        merchant_order_id: &str,
    ) -> Result<Option<PaymentRecord>> {
        Ok(self
            .payments
            .read()
            .get(&(provider, merchant_order_id.to_owned()))
            .cloned())
    }

    async fn insert_refund(&self, refund: RefundRecord) -> Result<RefundRecord> {
        let key = (refund.provider, refund.merchant_refund_id.clone());
        let mut refunds = self.refunds.write();
        if refunds.contains_key(&key) {
            return Err(UnipayError::new(
                ErrorCode::IdempotencyConflict,
                "refund already exists for merchant refund id",
            ));
        }
        refunds.insert(key, refund.clone());
        Ok(refund)
    }

    async fn update_refund(&self, refund: RefundRecord) -> Result<RefundRecord> {
        let key = (refund.provider, refund.merchant_refund_id.clone());
        self.refunds.write().insert(key, refund.clone());
        Ok(refund)
    }

    async fn find_refund(
        &self,
        provider: Provider,
        merchant_refund_id: &str,
    ) -> Result<Option<RefundRecord>> {
        Ok(self
            .refunds
            .read()
            .get(&(provider, merchant_refund_id.to_owned()))
            .cloned())
    }

    async fn list_refunds_for_payment(
        &self,
        provider: Provider,
        merchant_order_id: &str,
    ) -> Result<Vec<RefundRecord>> {
        Ok(self
            .refunds
            .read()
            .values()
            .filter(|refund| {
                refund.provider == provider && refund.merchant_order_id == merchant_order_id
            })
            .cloned()
            .collect())
    }

    async fn record_provider_request(&self, record: ProviderRequestRecord) -> Result<()> {
        self.provider_requests.write().push(record);
        Ok(())
    }

    async fn record_webhook_event(&self, event: WebhookEvent) -> Result<bool> {
        let key = (event.provider, event.deduplication_key.clone());
        let mut keys = self.webhook_deduplication_keys.write();
        if !keys.insert(key) {
            return Ok(false);
        }
        self.webhook_events.write().push(event);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use unipay_common::{CurrencyCode, Money};
    use unipay_core::{PaymentChannel, PaymentStatus};

    #[tokio::test]
    async fn deduplicates_payments_by_provider_and_merchant_order() {
        let storage = InMemoryPaymentStorage::new();
        let money = Money::new(CurrencyCode::new("CNY").unwrap(), 100).unwrap();
        let payment = PaymentRecord {
            payment_id: "pay_1".to_owned(),
            provider: Provider::Wechat,
            merchant_order_id: "order_1".to_owned(),
            provider_transaction_id: None,
            channel: PaymentChannel::Native,
            amount: money,
            status: PaymentStatus::Pending,
            subject: "subject".to_owned(),
            description: None,
            payment_action: None,
            paid_at: None,
            expires_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        storage.insert_payment(payment.clone()).await.unwrap();
        assert!(storage.insert_payment(payment).await.is_err());
    }
}

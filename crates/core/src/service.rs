use crate::error::{ErrorCode, Result, UnipayError};
use crate::models::*;
use crate::traits::{PaymentProvider, PaymentStorage};
use std::collections::HashMap;
use std::sync::Arc;
use unipay_common::{new_id, now_utc};

#[derive(Default)]
pub struct ProviderRegistry {
    providers: HashMap<Provider, Arc<dyn PaymentProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Arc<dyn PaymentProvider>) {
        self.providers.insert(provider.provider(), provider);
    }

    pub fn get(&self, provider: Provider) -> Result<Arc<dyn PaymentProvider>> {
        self.providers.get(&provider).cloned().ok_or_else(|| {
            UnipayError::new(
                ErrorCode::InvalidProvider,
                format!("provider {} is not registered", provider.as_str()),
            )
        })
    }
}

pub struct PaymentService {
    registry: ProviderRegistry,
    storage: Arc<dyn PaymentStorage>,
}

impl PaymentService {
    pub fn new(registry: ProviderRegistry, storage: Arc<dyn PaymentStorage>) -> Self {
        Self { registry, storage }
    }

    pub async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse> {
        if let Some(existing) = self
            .storage
            .find_payment(request.provider, &request.merchant_order_id)
            .await?
        {
            ensure_existing_payment_matches_request(&existing, &request)?;
            return Ok(record_to_create_response(existing));
        }

        let now = now_utc();
        let local_record = PaymentRecord {
            payment_id: new_id("pay"),
            provider: request.provider,
            merchant_order_id: request.merchant_order_id.clone(),
            provider_transaction_id: None,
            channel: request.channel,
            amount: request.amount.clone(),
            status: PaymentStatus::Pending,
            subject: request.subject.clone(),
            description: request.description.clone(),
            payment_action: None,
            paid_at: None,
            expires_at: request.expire_at,
            created_at: now,
            updated_at: now,
        };
        self.storage.insert_payment(local_record.clone()).await?;

        let provider = self.registry.get(request.provider)?;
        let response = provider.create_payment(request).await?;
        let updated = PaymentRecord {
            provider_transaction_id: response.provider_transaction_id.clone(),
            status: response.status,
            payment_action: Some(response.payment_action.clone()),
            updated_at: now_utc(),
            ..local_record
        };
        let updated = self.storage.update_payment(updated).await?;
        Ok(record_to_create_response(updated))
    }

    pub async fn query_payment(&self, query: PaymentQuery) -> Result<PaymentRecord> {
        if let Some(local) = self
            .storage
            .find_payment(query.provider, &query.merchant_order_id)
            .await?
        {
            return Ok(local);
        }

        let provider = self.registry.get(query.provider)?;
        provider.query_payment(query).await
    }

    pub async fn create_refund(&self, request: CreateRefundRequest) -> Result<RefundResponse> {
        if let Some(existing) = self
            .storage
            .find_refund(request.provider, &request.merchant_refund_id)
            .await?
        {
            ensure_existing_refund_matches_request(&existing, &request)?;
            return Ok(record_to_refund_response(existing));
        }

        let payment = self
            .storage
            .find_payment(request.provider, &request.merchant_order_id)
            .await?
            .ok_or_else(|| {
                UnipayError::new(ErrorCode::PaymentNotFound, "payment was not found")
                    .with_provider(request.provider.as_str())
                    .with_operation("create_refund")
            })?;

        if payment.status != PaymentStatus::Succeeded
            && payment.status != PaymentStatus::PartiallyRefunded
        {
            return Err(UnipayError::new(
                ErrorCode::PaymentStateConflict,
                "payment is not refundable in its current state",
            )
            .with_provider(request.provider.as_str())
            .with_operation("create_refund"));
        }
        ensure_refund_amount_is_available(&*self.storage, &payment, &request).await?;

        let now = now_utc();
        let local_refund = RefundRecord {
            refund_id: new_id("rfd"),
            payment_id: payment.payment_id,
            provider: request.provider,
            merchant_order_id: request.merchant_order_id.clone(),
            merchant_refund_id: request.merchant_refund_id.clone(),
            provider_refund_id: None,
            amount: request.amount.clone(),
            status: RefundStatus::Pending,
            reason: request.reason.clone(),
            succeeded_at: None,
            created_at: now,
            updated_at: now,
        };
        self.storage.insert_refund(local_refund.clone()).await?;

        let provider = self.registry.get(request.provider)?;
        let response = provider.create_refund(request).await?;
        let updated = RefundRecord {
            provider_refund_id: response.provider_refund_id.clone(),
            status: response.status,
            updated_at: now_utc(),
            ..local_refund
        };
        let updated = self.storage.update_refund(updated).await?;
        Ok(record_to_refund_response(updated))
    }

    pub async fn query_refund(&self, query: RefundQuery) -> Result<RefundRecord> {
        if let Some(local) = self
            .storage
            .find_refund(query.provider, &query.merchant_refund_id)
            .await?
        {
            return Ok(local);
        }

        let provider = self.registry.get(query.provider)?;
        provider.query_refund(query).await
    }
}

fn record_to_create_response(record: PaymentRecord) -> CreatePaymentResponse {
    CreatePaymentResponse {
        payment_id: record.payment_id,
        provider: record.provider,
        merchant_order_id: record.merchant_order_id,
        provider_transaction_id: record.provider_transaction_id,
        status: record.status,
        amount: record.amount,
        payment_action: record.payment_action.unwrap_or(PaymentAction::None),
        expires_at: record.expires_at,
        created_at: record.created_at,
    }
}

fn record_to_refund_response(record: RefundRecord) -> RefundResponse {
    RefundResponse {
        refund_id: record.refund_id,
        provider: record.provider,
        merchant_order_id: record.merchant_order_id,
        merchant_refund_id: record.merchant_refund_id,
        provider_refund_id: record.provider_refund_id,
        status: record.status,
        amount: record.amount,
        created_at: record.created_at,
    }
}

fn ensure_existing_payment_matches_request(
    existing: &PaymentRecord,
    request: &CreatePaymentRequest,
) -> Result<()> {
    let same_request = existing.amount == request.amount
        && existing.channel == request.channel
        && existing.subject == request.subject
        && existing.description == request.description
        && existing.expires_at == request.expire_at;

    if same_request {
        Ok(())
    } else {
        Err(UnipayError::new(
            ErrorCode::IdempotencyConflict,
            "merchant order id was reused with different payment parameters",
        )
        .with_provider(request.provider.as_str())
        .with_operation("create_payment"))
    }
}

fn ensure_existing_refund_matches_request(
    existing: &RefundRecord,
    request: &CreateRefundRequest,
) -> Result<()> {
    let same_request = existing.merchant_order_id == request.merchant_order_id
        && existing.amount == request.amount
        && existing.reason == request.reason;

    if same_request {
        Ok(())
    } else {
        Err(UnipayError::new(
            ErrorCode::IdempotencyConflict,
            "merchant refund id was reused with different refund parameters",
        )
        .with_provider(request.provider.as_str())
        .with_operation("create_refund"))
    }
}

async fn ensure_refund_amount_is_available(
    storage: &dyn PaymentStorage,
    payment: &PaymentRecord,
    request: &CreateRefundRequest,
) -> Result<()> {
    if request.amount.currency != payment.amount.currency {
        return Err(UnipayError::new(
            ErrorCode::InvalidCurrency,
            "refund currency must match payment currency",
        )
        .with_provider(request.provider.as_str())
        .with_operation("create_refund"));
    }

    let reserved_amount = storage
        .list_refunds_for_payment(request.provider, &request.merchant_order_id)
        .await?
        .into_iter()
        .filter(|refund| refund.amount.currency == payment.amount.currency)
        .filter(|refund| refund_reserves_amount(refund.status))
        .map(|refund| refund.amount.amount_minor)
        .try_fold(0_i64, |total, amount| total.checked_add(amount))
        .ok_or_else(|| {
            UnipayError::new(
                ErrorCode::InternalError,
                "refund amount overflow while checking refundable balance",
            )
            .with_provider(request.provider.as_str())
            .with_operation("create_refund")
        })?;

    let requested_total = reserved_amount
        .checked_add(request.amount.amount_minor)
        .ok_or_else(|| {
            UnipayError::new(
                ErrorCode::InternalError,
                "refund amount overflow while checking requested amount",
            )
            .with_provider(request.provider.as_str())
            .with_operation("create_refund")
        })?;

    if requested_total > payment.amount.amount_minor {
        return Err(UnipayError::new(
            ErrorCode::RefundAmountExceeded,
            "refund amount exceeds remaining refundable payment amount",
        )
        .with_provider(request.provider.as_str())
        .with_operation("create_refund"));
    }

    Ok(())
}

fn refund_reserves_amount(status: RefundStatus) -> bool {
    !matches!(status, RefundStatus::Failed | RefundStatus::Closed)
}

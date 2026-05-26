use crate::error::{Operation, ProviderError, ProviderResult, UniPayErrorCode};
use unipay_common::Money;
use unipay_core::{PaymentStatus, RefundStatus};

pub fn map_wechat_trade_state(value: &str) -> PaymentStatus {
    match value {
        "SUCCESS" => PaymentStatus::Succeeded,
        "REFUND" => PaymentStatus::Refunding,
        "NOTPAY" => PaymentStatus::Pending,
        "CLOSED" | "REVOKED" => PaymentStatus::Closed,
        "USERPAYING" => PaymentStatus::Processing,
        "PAYERROR" => PaymentStatus::Failed,
        _ => PaymentStatus::Unknown,
    }
}

pub fn map_wechat_refund_status(value: &str) -> RefundStatus {
    match value {
        "SUCCESS" => RefundStatus::Succeeded,
        "CLOSED" => RefundStatus::Closed,
        "PROCESSING" => RefundStatus::Processing,
        "ABNORMAL" => RefundStatus::Unknown,
        _ => RefundStatus::Unknown,
    }
}

pub fn wechat_amount_from_money(money: &Money, operation: Operation) -> ProviderResult<i64> {
    if money.currency.as_str() != "CNY" {
        return Err(ProviderError::new(
            UniPayErrorCode::InvalidCurrency,
            operation,
            false,
            "WeChat MVP supports CNY only",
        ));
    }

    if money.amount_minor <= 0 {
        return Err(ProviderError::new(
            UniPayErrorCode::InvalidAmount,
            operation,
            false,
            "amount_minor must be positive",
        ));
    }

    Ok(money.amount_minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_payment_states() {
        assert_eq!(map_wechat_trade_state("SUCCESS"), PaymentStatus::Succeeded);
        assert_eq!(map_wechat_trade_state("NOTPAY"), PaymentStatus::Pending);
        assert_eq!(
            map_wechat_trade_state("UNKNOWN_NEW"),
            PaymentStatus::Unknown
        );
    }

    #[test]
    fn maps_refund_states() {
        assert_eq!(map_wechat_refund_status("SUCCESS"), RefundStatus::Succeeded);
        assert_eq!(
            map_wechat_refund_status("PROCESSING"),
            RefundStatus::Processing
        );
        assert_eq!(map_wechat_refund_status("OTHER"), RefundStatus::Unknown);
    }
}

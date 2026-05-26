use unipay_common::{CurrencyCode, Money};
use unipay_core::{ErrorCode, PaymentStatus, Result, UnipayError};

pub fn map_alipay_trade_status(value: &str) -> PaymentStatus {
    match value {
        "WAIT_BUYER_PAY" => PaymentStatus::Pending,
        "TRADE_CLOSED" => PaymentStatus::Closed,
        "TRADE_SUCCESS" | "TRADE_FINISHED" => PaymentStatus::Succeeded,
        _ => PaymentStatus::Unknown,
    }
}

pub fn alipay_amount_from_money(money: &Money) -> Result<String> {
    if money.currency.as_str() != "CNY" {
        return Err(
            UnipayError::new(ErrorCode::InvalidCurrency, "Alipay MVP supports CNY only")
                .with_provider("alipay"),
        );
    }
    if money.amount_minor <= 0 {
        return Err(
            UnipayError::new(ErrorCode::InvalidAmount, "amount_minor must be positive")
                .with_provider("alipay"),
        );
    }

    Ok(format!(
        "{}.{:02}",
        money.amount_minor / 100,
        money.amount_minor % 100
    ))
}

pub fn money_from_alipay_amount(value: &str) -> Result<Money> {
    let (major, minor) = value.split_once('.').unwrap_or((value, "0"));
    let major: i64 = major.parse().map_err(|_| {
        UnipayError::new(
            ErrorCode::InvalidAmount,
            "Alipay amount major unit is invalid",
        )
        .with_provider("alipay")
    })?;
    let mut minor = minor.to_owned();
    if minor.len() > 2 {
        return Err(UnipayError::new(
            ErrorCode::InvalidAmount,
            "Alipay amount has more than two decimal places",
        )
        .with_provider("alipay"));
    }
    while minor.len() < 2 {
        minor.push('0');
    }
    let minor: i64 = minor.parse().map_err(|_| {
        UnipayError::new(
            ErrorCode::InvalidAmount,
            "Alipay amount minor unit is invalid",
        )
        .with_provider("alipay")
    })?;
    Money::new(
        CurrencyCode::new("CNY").expect("static currency is valid"),
        major * 100 + minor,
    )
    .map_err(|_| UnipayError::new(ErrorCode::InvalidAmount, "Alipay amount is invalid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_minor_units_to_decimal_string() {
        let money = Money::new(CurrencyCode::new("CNY").unwrap(), 123).unwrap();
        assert_eq!(alipay_amount_from_money(&money).unwrap(), "1.23");
    }

    #[test]
    fn converts_decimal_string_to_minor_units() {
        assert_eq!(money_from_alipay_amount("1.20").unwrap().amount_minor, 120);
        assert_eq!(money_from_alipay_amount("1").unwrap().amount_minor, 100);
    }

    #[test]
    fn maps_trade_statuses() {
        assert_eq!(
            map_alipay_trade_status("WAIT_BUYER_PAY"),
            PaymentStatus::Pending
        );
        assert_eq!(
            map_alipay_trade_status("TRADE_SUCCESS"),
            PaymentStatus::Succeeded
        );
        assert_eq!(map_alipay_trade_status("OTHER"), PaymentStatus::Unknown);
    }
}

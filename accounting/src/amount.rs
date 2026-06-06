use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// 将 Decimal 转换为数据库存储的整数
///
/// 根据 commodity precision 进行缩放：precision=2 时 12.34 → 1234
pub fn to_db_amount(amount: Decimal, precision: u8) -> i64 {
    let scale = 10i64.pow(precision as u32);
    (amount * Decimal::from(scale)).round().to_i64().unwrap_or(0)
}

/// 将数据库存储的整数还原为 Decimal
///
/// precision=2 时 1234 → 12.34
pub fn from_db_amount(stored: i64, precision: u8) -> Decimal {
    let scale = 10i64.pow(precision as u32);
    Decimal::from(stored) / Decimal::from(scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_to_db_amount() {
        let d = Decimal::from_str("12.34").unwrap();
        assert_eq!(to_db_amount(d, 2), 1234i64);
    }

    #[test]
    fn test_to_db_amount_negative() {
        let d = Decimal::from_str("-5.00").unwrap();
        assert_eq!(to_db_amount(d, 2), -500i64);
    }

    #[test]
    fn test_from_db_amount() {
        assert_eq!(from_db_amount(1234i64, 2), Decimal::from_str("12.34").unwrap());
    }

    #[test]
    fn test_from_db_amount_zero() {
        assert_eq!(from_db_amount(0i64, 2), Decimal::ZERO);
    }
}

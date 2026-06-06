use chrono::{Datelike, NaiveDate};

/// 根据交易日期和还款日推断分期期数
///
/// 规则：交易日期在还款日之前 → 当期(1)，之后 → 下期(2)
pub fn infer_installment_index(tx_date: NaiveDate, repayment_day: u8) -> u32 {
    let day = tx_date.day() as u8;
    if day <= repayment_day {
        1
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_before_repayment_day() {
        let tx_date = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let repayment_day = 15u8;
        let index = infer_installment_index(tx_date, repayment_day);
        assert_eq!(index, 1);
    }

    #[test]
    fn test_after_repayment_day() {
        let tx_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let repayment_day = 15u8;
        let index = infer_installment_index(tx_date, repayment_day);
        assert_eq!(index, 2);
    }

    #[test]
    fn test_cross_month() {
        let tx_date = NaiveDate::from_ymd_opt(2024, 1, 25).unwrap();
        let repayment_day = 10u8;
        let index = infer_installment_index(tx_date, repayment_day);
        assert_eq!(index, 2);
    }
}

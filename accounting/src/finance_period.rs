use chrono::Datelike;
use chrono::NaiveDate;

/// 财务周期类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinancePeriod {
    /// 自然日（当天 00:00 ~ 23:59:59）
    Daily = 1,
    /// 自然周（周日起始）
    WeeklyFromSunday = 2,
    /// 自然周（周一起始）
    WeeklyFromMonday = 3,
    /// 自然月（每月1号起始）
    Monthly = 4,
    /// 自然年（每年1月1号起始）
    Yearly = 5,
}

impl FinancePeriod {
    /// 给定日期，返回当前周期的起止日期范围
    ///
    /// ## 示例
    ///
    /// ```plaintext
    /// Daily,  2026-06-26  → (2026-06-26, 2026-06-26)
    /// WeeklyFromMonday, 2026-06-26(周五) → (2026-06-22, 2026-06-28)
    /// WeeklyFromSunday, 2026-06-26(周五) → (2026-06-21, 2026-06-27)
    /// Monthly, 2026-06-26 → (2026-06-01, 2026-06-30)
    /// Yearly,  2026-06-26 → (2026-01-01, 2026-12-31)
    /// ```
    pub fn period_range(&self, date: NaiveDate) -> (NaiveDate, NaiveDate) {
        match self {
            FinancePeriod::Daily => (date, date),
            FinancePeriod::WeeklyFromSunday => {
                let weekday = date.weekday().num_days_from_sunday() as i64;
                let start = date - chrono::Duration::days(weekday);
                let end = start + chrono::Duration::days(6);
                (start, end)
            }
            FinancePeriod::WeeklyFromMonday => {
                let weekday = date.weekday().num_days_from_monday() as i64;
                let start = date - chrono::Duration::days(weekday);
                let end = start + chrono::Duration::days(6);
                (start, end)
            }
            FinancePeriod::Monthly => {
                let year = date.year();
                let month = date.month();
                let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                let end = start
                    .with_month(month + 1)
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
                    - chrono::Duration::days(1);
                (start, end)
            }
            FinancePeriod::Yearly => {
                let year = date.year();
                let start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
                (start, end)
            }
        }
    }

    /// 从整数创建 FinancePeriod
    pub fn from_i64(value: i64) -> Option<Self> {
        match value {
            1 => Some(FinancePeriod::Daily),
            2 => Some(FinancePeriod::WeeklyFromSunday),
            3 => Some(FinancePeriod::WeeklyFromMonday),
            4 => Some(FinancePeriod::Monthly),
            5 => Some(FinancePeriod::Yearly),
            _ => None,
        }
    }

    /// 转为整数
    pub fn as_i64(&self) -> i64 {
        *self as i64
    }
}

impl std::fmt::Display for FinancePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinancePeriod::Daily => write!(f, "Daily"),
            FinancePeriod::WeeklyFromSunday => write!(f, "WeeklyFromSunday"),
            FinancePeriod::WeeklyFromMonday => write!(f, "WeeklyFromMonday"),
            FinancePeriod::Monthly => write!(f, "Monthly"),
            FinancePeriod::Yearly => write!(f, "Yearly"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_period_range_daily() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = FinancePeriod::Daily.period_range(date);
        assert_eq!(start, date);
        assert_eq!(end, date);
    }

    #[test]
    fn test_period_range_weekly_from_monday_friday() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = FinancePeriod::WeeklyFromMonday.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 22).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 28).unwrap());
    }

    #[test]
    fn test_period_range_weekly_from_sunday_friday() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = FinancePeriod::WeeklyFromSunday.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 21).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 27).unwrap());
    }

    #[test]
    fn test_period_range_weekly_from_monday_on_monday() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 22).unwrap();
        let (start, end) = FinancePeriod::WeeklyFromMonday.period_range(date);
        assert_eq!(start, date);
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 28).unwrap());
    }

    #[test]
    fn test_period_range_weekly_from_sunday_on_sunday() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
        let (start, end) = FinancePeriod::WeeklyFromSunday.period_range(date);
        assert_eq!(start, date);
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 27).unwrap());
    }

    #[test]
    fn test_period_range_monthly() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = FinancePeriod::Monthly.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
    }

    #[test]
    fn test_period_range_monthly_december() {
        let date = NaiveDate::from_ymd_opt(2026, 12, 15).unwrap();
        let (start, end) = FinancePeriod::Monthly.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 12, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    #[test]
    fn test_period_range_yearly() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let (start, end) = FinancePeriod::Yearly.period_range(date);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    #[test]
    fn test_from_i64() {
        assert_eq!(FinancePeriod::from_i64(1), Some(FinancePeriod::Daily));
        assert_eq!(
            FinancePeriod::from_i64(2),
            Some(FinancePeriod::WeeklyFromSunday)
        );
        assert_eq!(
            FinancePeriod::from_i64(3),
            Some(FinancePeriod::WeeklyFromMonday)
        );
        assert_eq!(FinancePeriod::from_i64(4), Some(FinancePeriod::Monthly));
        assert_eq!(FinancePeriod::from_i64(5), Some(FinancePeriod::Yearly));
        assert_eq!(FinancePeriod::from_i64(0), None);
        assert_eq!(FinancePeriod::from_i64(6), None);
    }

    #[test]
    fn test_as_i64() {
        assert_eq!(FinancePeriod::Daily.as_i64(), 1);
        assert_eq!(FinancePeriod::WeeklyFromSunday.as_i64(), 2);
        assert_eq!(FinancePeriod::WeeklyFromMonday.as_i64(), 3);
        assert_eq!(FinancePeriod::Monthly.as_i64(), 4);
        assert_eq!(FinancePeriod::Yearly.as_i64(), 5);
    }

    #[test]
    fn test_roundtrip() {
        for value in 1..=5 {
            let period = FinancePeriod::from_i64(value).unwrap();
            assert_eq!(period.as_i64(), value);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(FinancePeriod::Daily.to_string(), "Daily");
        assert_eq!(
            FinancePeriod::WeeklyFromSunday.to_string(),
            "WeeklyFromSunday"
        );
        assert_eq!(
            FinancePeriod::WeeklyFromMonday.to_string(),
            "WeeklyFromMonday"
        );
        assert_eq!(FinancePeriod::Monthly.to_string(), "Monthly");
        assert_eq!(FinancePeriod::Yearly.to_string(), "Yearly");
    }
}

//! 资产趋势表

use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId};
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap};

/// 资产趋势点（某个时间桶结束时的总资产与总负债）
#[derive(Debug, Clone, PartialEq)]
pub struct NetWorthTrendPoint {
    /// 桶起始日期
    pub date: NaiveDate,
    /// 总资产（正余额之和）
    pub assets: Decimal,
    /// 总负债（负余额绝对值之和）
    pub liabilities: Decimal,
}

/// 资产趋势表服务
pub struct NetWorthTrendService {
    db: SqliteDatabase,
}

impl NetWorthTrendService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 生成资产趋势序列
    ///
    /// 返回全量历史按 period 分桶的总资产与总负债序列，按日期升序。
    pub async fn net_worth_trend(
        &self,
        period: FinancePeriod,
        commodity_id: CommodityId,
    ) -> Result<Vec<NetWorthTrendPoint>, AccountingError> {
        let deltas = self
            .db
            .posting_daily_delta_by_account(commodity_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(compute_trend(&deltas, period))
    }
}

/// 将 (账户, 日, 增量) 序列按 period 分桶、前缀累计并按符号拆分为资产/负债序列
fn compute_trend(
    deltas: &[(AccountId, NaiveDate, Decimal)],
    period: FinancePeriod,
) -> Vec<NetWorthTrendPoint> {
    if deltas.is_empty() {
        return Vec::new();
    }

    // 按 (桶起始日, 账户) 累加增量
    let mut bucket_deltas: BTreeMap<NaiveDate, HashMap<AccountId, Decimal>> = BTreeMap::new();
    for &(account, day, delta) in deltas {
        let bucket = period.period_range(day).0;
        *bucket_deltas
            .entry(bucket)
            .or_default()
            .entry(account)
            .or_insert(Decimal::ZERO) += delta;
    }

    // 枚举首尾桶之间的所有桶（含无分录的空桶）
    let first = *bucket_deltas.keys().next().unwrap();
    let last = *bucket_deltas.keys().next_back().unwrap();
    let mut buckets = Vec::new();
    let mut cur = first;
    while cur <= last {
        buckets.push(cur);
        let end = period.period_range(cur).1;
        cur = period
            .period_range(end + chrono::Duration::days(1))
            .0;
    }

    // 逐账户前缀和，按符号拆分
    let mut balances: HashMap<AccountId, Decimal> = HashMap::new();
    let mut points = Vec::new();
    for bucket in buckets {
        if let Some(deltas_in_bucket) = bucket_deltas.get(&bucket) {
            for (account, delta) in deltas_in_bucket {
                *balances.entry(*account).or_insert(Decimal::ZERO) += *delta;
            }
        }
        let mut assets = Decimal::ZERO;
        let mut liabilities = Decimal::ZERO;
        for balance in balances.values() {
            if balance.is_sign_negative() {
                liabilities += balance.abs();
            } else {
                assets += *balance;
            }
        }
        points.push(NetWorthTrendPoint {
            date: bucket,
            assets,
            liabilities,
        });
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_empty_deltas() {
        let points = compute_trend(&[], FinancePeriod::Monthly);
        assert!(points.is_empty());
    }

    #[test]
    fn test_cross_bucket_accumulation() {
        let a = AccountId(1);
        let deltas = vec![
            (a, d(2024, 6, 10), dec("3000")),
            (a, d(2024, 7, 5), dec("-1000")),
        ];
        let points = compute_trend(&deltas, FinancePeriod::Monthly);

        assert_eq!(points.len(), 2);
        assert_eq!(points[0].date, d(2024, 6, 1));
        assert_eq!(points[0].assets, dec("3000"));
        assert_eq!(points[1].date, d(2024, 7, 1));
        assert_eq!(points[1].assets, dec("2000"));
    }

    #[test]
    fn test_empty_bucket_carries_balance() {
        let a = AccountId(1);
        let deltas = vec![
            (a, d(2024, 1, 15), dec("5000")),
            (a, d(2024, 3, 20), dec("1000")),
        ];
        let points = compute_trend(&deltas, FinancePeriod::Monthly);

        assert_eq!(points.len(), 3);
        assert_eq!(points[0].date, d(2024, 1, 1));
        assert_eq!(points[0].assets, dec("5000"));
        // 空桶延续上一桶余额
        assert_eq!(points[1].date, d(2024, 2, 1));
        assert_eq!(points[1].assets, dec("5000"));
        assert_eq!(points[2].date, d(2024, 3, 1));
        assert_eq!(points[2].assets, dec("6000"));
    }

    #[test]
    fn test_sign_split_assets_liabilities() {
        let bank = AccountId(1);
        let credit_card = AccountId(2);
        let deltas = vec![
            (bank, d(2024, 6, 1), dec("50000")),
            (credit_card, d(2024, 6, 2), dec("-8000")),
        ];
        let points = compute_trend(&deltas, FinancePeriod::Monthly);

        assert_eq!(points.len(), 1);
        assert_eq!(points[0].assets, dec("50000"));
        assert_eq!(points[0].liabilities, dec("8000"));
    }

    #[test]
    fn test_account_flips_sign_across_buckets() {
        let a = AccountId(1);
        let deltas = vec![
            (a, d(2024, 1, 5), dec("1000")),
            (a, d(2024, 2, 5), dec("-3000")),
        ];
        let points = compute_trend(&deltas, FinancePeriod::Monthly);

        assert_eq!(points[0].assets, dec("1000"));
        assert_eq!(points[0].liabilities, dec("0"));
        // 余额翻转为负 → 计入负债
        assert_eq!(points[1].assets, dec("0"));
        assert_eq!(points[1].liabilities, dec("2000"));
    }

    #[test]
    fn test_weekly_bucketing() {
        let a = AccountId(1);
        // 2024-01-01 是周一
        let deltas = vec![
            (a, d(2024, 1, 3), dec("100")),
            (a, d(2024, 1, 10), dec("200")),
        ];
        let points = compute_trend(&deltas, FinancePeriod::WeeklyFromMonday);

        assert_eq!(points.len(), 2);
        assert_eq!(points[0].date, d(2024, 1, 1));
        assert_eq!(points[0].assets, dec("100"));
        assert_eq!(points[1].date, d(2024, 1, 8));
        assert_eq!(points[1].assets, dec("300"));
    }
}

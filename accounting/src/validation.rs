use crate::error::AccountingError;
use crate::posting::Posting;
use crate::account_type::AccountType;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 验证交易平衡性
///
/// 规则：
/// - 至少两个分录
/// - 同一 commodity 的金额之和为零
/// - 不同 commodity 的交易必须有 cost 字段建立等式
pub fn validate_transaction(postings: &[Posting]) -> Result<(), AccountingError> {
    if postings.len() < 2 {
        return Err(AccountingError::InvalidTransaction(
            "交易至少包含两个分录".to_string(),
        ));
    }

    // 按 commodity 分组求和
    let mut sums: HashMap<i64, Decimal> = HashMap::new();
    for p in postings {
        *sums.entry(p.commodity_id.0).or_insert_with(|| Decimal::ZERO) += p.amount;
    }

    // 检查是否所有 commodity 都能自平衡
    let unbalanced: Vec<_> = sums.iter().filter(|(_, v)| !v.is_zero()).collect();
    if unbalanced.is_empty() {
        return Ok(());
    }

    // 多币种情况：检查 cost 是否能建立等式
    let mut cost_sums: HashMap<i64, Decimal> = HashMap::new();
    for p in postings {
        if let Some(cost) = p.cost {
            let cost_commodity = p.cost_commodity_id.map(|c| c.0).unwrap_or(p.commodity_id.0);
            *cost_sums.entry(cost_commodity).or_insert_with(|| Decimal::ZERO) += cost;
        } else {
            *cost_sums.entry(p.commodity_id.0).or_insert_with(|| Decimal::ZERO) += p.amount;
        }
    }

    let unbalanced_costs: Vec<_> = cost_sums.iter().filter(|(_, v)| !v.is_zero()).collect();
    if unbalanced_costs.is_empty() {
        Ok(())
    } else {
        Err(AccountingError::InvalidTransaction(
            "交易不平衡".to_string(),
        ))
    }
}

/// 验证账户是否可以关闭
///
/// Asset 和 Liability 必须余额为零；Income 和 Expense 无限制
pub fn validate_account_close(
    account_type: AccountType,
    balances: &[(crate::id::CommodityId, Decimal)],
) -> Result<(), AccountingError> {
    match account_type {
        AccountType::Asset | AccountType::Liability | AccountType::Expense | AccountType::Equity => {
            let non_zero: Vec<_> = balances
                .iter()
                .filter(|(_, b)| !b.is_zero())
                .collect();
            if !non_zero.is_empty() {
                return Err(AccountingError::AccountNotEmpty(
                    "账户余额非零".to_string(),
                ));
            }
        }
        AccountType::Income => {
            // Income 账户关闭无限制
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{AccountId, CommodityId, PostingId, TransactionId};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn posting(account_id: i64, commodity_id: i64, amount: &str, cost: Option<&str>, cost_commodity: Option<i64>) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id: AccountId(account_id),
            commodity_id: CommodityId(commodity_id),
            amount: Decimal::from_str(amount).unwrap(),
            cost: cost.map(|c| Decimal::from_str(c).unwrap()),
            cost_commodity_id: cost_commodity.map(CommodityId),
            description: None,
            member_id: None,
            channel_id: None,
        }
    }

    #[test]
    fn test_empty_postings_fails() {
        let postings: Vec<Posting> = vec![];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_single_posting_fails() {
        let postings = vec![posting(1, 1, "100", None, None)];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_balanced_same_commodity_passes() {
        let postings = vec![
            posting(1, 1, "100", None, None),
            posting(2, 1, "-100", None, None),
        ];
        assert!(validate_transaction(&postings).is_ok());
    }

    #[test]
    fn test_unbalanced_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None),
            posting(2, 1, "-50", None, None),
        ];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_multi_commodity_with_cost_passes() {
        let postings = vec![
            posting(1, 1, "100", Some("70000"), Some(2)),
            posting(2, 2, "-70000", None, None),
        ];
        assert!(validate_transaction(&postings).is_ok());
    }

    #[test]
    fn test_multi_commodity_without_cost_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None),
            posting(2, 2, "-700", None, None),
        ];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_close_asset_with_zero_balance_ok() {
        let balances = vec![(CommodityId(1), Decimal::ZERO)];
        assert!(validate_account_close(AccountType::Asset, &balances).is_ok());
    }

    #[test]
    fn test_close_asset_with_non_zero_balance_fails() {
        let balances = vec![(CommodityId(1), Decimal::from_str("100").unwrap())];
        assert!(validate_account_close(AccountType::Asset, &balances).is_err());
    }

    #[test]
    fn test_close_income_unconditionally_ok() {
        let balances = vec![(CommodityId(1), Decimal::from_str("100").unwrap())];
        assert!(validate_account_close(AccountType::Income, &balances).is_ok());
    }
}

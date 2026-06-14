use crate::account_type::AccountType;
use crate::error::AccountingError;
use crate::posting::Posting;
use crate::transaction::TransactionKind;
use rust_decimal::Decimal;
use rust_i18n::t;
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
            t!("tx_at_least_two_postings").to_string(),
        ));
    }

    // 按 commodity 分组求和
    let mut sums: HashMap<i64, Decimal> = HashMap::new();
    for p in postings {
        *sums
            .entry(p.commodity_id.0)
            .or_insert_with(|| Decimal::ZERO) += p.amount;
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
            *cost_sums
                .entry(cost_commodity)
                .or_insert_with(|| Decimal::ZERO) += cost;
        } else {
            *cost_sums
                .entry(p.commodity_id.0)
                .or_insert_with(|| Decimal::ZERO) += p.amount;
        }
    }

    let unbalanced_costs: Vec<_> = cost_sums.iter().filter(|(_, v)| !v.is_zero()).collect();
    if unbalanced_costs.is_empty() {
        Ok(())
    } else {
        Err(AccountingError::InvalidTransaction(
            t!("tx_unbalanced").to_string(),
        ))
    }
}

/// 验证交易级 kind 与分录结构的一致性
///
/// 规则：
/// - Normal 交易中所有分录的 linked_posting_id 必须为 None
/// - Refund/Reimbursement 交易必须至少有一个分录的 linked_posting_id 不为 None
pub fn validate_kind_consistency(
    kind: TransactionKind,
    postings: &[Posting],
) -> Result<(), AccountingError> {
    let has_reversal = postings.iter().any(|p| p.linked_posting_id.is_some());
    match kind {
        TransactionKind::Normal => {
            if has_reversal {
                return Err(AccountingError::InvalidTransaction(
                    t!("normal_tx_no_reversal").to_string(),
                ));
            }
        }
        TransactionKind::Refund | TransactionKind::Reimbursement => {
            if !has_reversal {
                return Err(AccountingError::InvalidTransaction(
                    t!("refund_tx_must_have_reversal").to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// 验证冲减分录金额不为零
///
/// 更严格的方向验证（与原分录方向相反）在 service 层通过数据库查询完成。
pub fn validate_reversal_direction(postings: &[Posting]) -> Result<(), AccountingError> {
    for posting in postings {
        if posting.linked_posting_id.is_none() {
            continue;
        }
        if posting.amount.is_zero() {
            return Err(AccountingError::InvalidTransaction(
                t!("reversal_amount_cannot_be_zero").to_string(),
            ));
        }
    }
    Ok(())
}

/// 验证冲减金额不超过原分录剩余可冲减额度
///
/// `linked_amount`: 冲减分录金额
/// `original_amount`: 原分录金额
/// `existing_reversal_total`: 原分录已被其他冲减分录冲减的累计金额
pub fn validate_reversal_cap(
    linked_amount: Decimal,
    original_amount: Decimal,
    existing_reversal_total: Decimal,
) -> Result<(), AccountingError> {
    let used = existing_reversal_total.abs() + linked_amount.abs();
    let available = original_amount.abs();
    if used > available {
        return Err(AccountingError::InvalidTransaction(format!(
            "{}",
            t!(
                "reversal_cap_exceeded",
                reversed = existing_reversal_total.abs(),
                current = linked_amount.abs(),
                original = available
            )
        )));
    }
    Ok(())
}

/// 验证账户是否可以关闭
///
/// 仅 Asset 要求余额为零；Income、Expense、Equity、Liability 无限制
pub fn validate_account_close(
    account_type: AccountType,
    balances: &[(crate::id::CommodityId, Decimal)],
) -> Result<(), AccountingError> {
    match account_type {
        AccountType::Asset => {
            let non_zero: Vec<_> = balances.iter().filter(|(_, b)| !b.is_zero()).collect();
            if !non_zero.is_empty() {
                return Err(AccountingError::AccountNotEmpty(
                    t!("account_balance_non_zero").to_string(),
                ));
            }
        }
        AccountType::Liability
        | AccountType::Income
        | AccountType::Expense
        | AccountType::Equity => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{AccountId, CommodityId, PostingId, TransactionId};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn posting(
        account_id: i64,
        commodity_id: i64,
        amount: &str,
        cost: Option<&str>,
        cost_commodity: Option<i64>,
        linked_posting_id: Option<i64>,
    ) -> Posting {
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
            is_reimbursable: false,
            linked_posting_id: linked_posting_id.map(PostingId),
            reversal_total: Decimal::ZERO,
        }
    }

    #[test]
    fn test_empty_postings_fails() {
        let postings: Vec<Posting> = vec![];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_single_posting_fails() {
        let postings = vec![posting(1, 1, "100", None, None, None)];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_balanced_same_commodity_passes() {
        let postings = vec![
            posting(1, 1, "100", None, None, None),
            posting(2, 1, "-100", None, None, None),
        ];
        assert!(validate_transaction(&postings).is_ok());
    }

    #[test]
    fn test_unbalanced_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None, None),
            posting(2, 1, "-50", None, None, None),
        ];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_multi_commodity_with_cost_passes() {
        let postings = vec![
            posting(1, 1, "100", Some("70000"), Some(2), None),
            posting(2, 2, "-70000", None, None, None),
        ];
        assert!(validate_transaction(&postings).is_ok());
    }

    #[test]
    fn test_multi_commodity_without_cost_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None, None),
            posting(2, 2, "-700", None, None, None),
        ];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_normal_tx_with_reversal_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None, Some(1)),
            posting(2, 1, "-100", None, None, None),
        ];
        assert!(validate_kind_consistency(TransactionKind::Normal, &postings).is_err());
    }

    #[test]
    fn test_refund_tx_without_reversal_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None, None),
            posting(2, 1, "-100", None, None, None),
        ];
        assert!(validate_kind_consistency(TransactionKind::Refund, &postings).is_err());
    }

    #[test]
    fn test_refund_tx_with_reversal_passes() {
        let postings = vec![
            posting(1, 1, "-50", None, None, Some(1)),
            posting(2, 1, "50", None, None, None),
        ];
        assert!(validate_kind_consistency(TransactionKind::Refund, &postings).is_ok());
    }

    #[test]
    fn test_reversal_cap_exceeded_fails() {
        assert!(
            validate_reversal_cap(
                Decimal::from_str("60").unwrap(),
                Decimal::from_str("100").unwrap(),
                Decimal::from_str("50").unwrap(),
            )
            .is_err()
        );
    }

    #[test]
    fn test_reversal_cap_within_limit_passes() {
        assert!(
            validate_reversal_cap(
                Decimal::from_str("40").unwrap(),
                Decimal::from_str("100").unwrap(),
                Decimal::from_str("50").unwrap(),
            )
            .is_ok()
        );
    }

    #[test]
    fn test_reversal_direction_zero_rejected() {
        let postings = vec![
            posting(1, 1, "0", None, None, Some(1)),
            posting(2, 1, "0", None, None, None),
        ];
        assert!(validate_reversal_direction(&postings).is_err());
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

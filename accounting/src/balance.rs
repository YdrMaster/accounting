use crate::id::CommodityId;
use crate::posting::Posting;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 计算指定 commodity 的余额
pub fn calculate_balance(postings: &[Posting], commodity_id: CommodityId) -> Decimal {
    postings
        .iter()
        .filter(|p| p.commodity_id == commodity_id)
        .map(|p| p.amount)
        .sum()
}

/// 计算所有 commodity 的余额
pub fn calculate_all_balances(postings: &[Posting]) -> HashMap<CommodityId, Decimal> {
    let mut balances: HashMap<CommodityId, Decimal> = HashMap::new();
    for p in postings {
        *balances
            .entry(p.commodity_id)
            .or_insert_with(|| Decimal::ZERO) += p.amount;
    }
    balances
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{AccountId, CommodityId, PostingId, TransactionId};
    use crate::posting::Posting;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn make_posting(account_id: i64, commodity_id: i64, amount: &str) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id: AccountId(account_id),
            commodity_id: CommodityId(commodity_id),
            amount: Decimal::from_str(amount).unwrap(),
            cost: None,
            cost_commodity_id: None,
            description: None,
            member_id: None,
            channel_id: None,
        }
    }

    #[test]
    fn test_single_commodity_balance() {
        let postings = vec![
            make_posting(1, 1, "100"),
            make_posting(1, 1, "50"),
            make_posting(1, 1, "-30"),
        ];
        let balance = calculate_balance(&postings, CommodityId(1));
        assert_eq!(balance, Decimal::from_str("120").unwrap());
    }

    #[test]
    fn test_multi_commodity_balances() {
        let postings = vec![make_posting(1, 1, "100"), make_posting(1, 2, "50")];
        let balances = calculate_all_balances(&postings);
        assert_eq!(balances[&CommodityId(1)], Decimal::from_str("100").unwrap());
        assert_eq!(balances[&CommodityId(2)], Decimal::from_str("50").unwrap());
    }

    #[test]
    fn test_balance_zero() {
        let postings = vec![];
        let balance = calculate_balance(&postings, CommodityId(1));
        assert_eq!(balance, Decimal::ZERO);
    }
}

//! 资产负债表

use accounting::account::Account;
use accounting::error::AccountingError;
use accounting::id::{AccountId, CommodityId};
use accounting_sql::SqliteDatabase;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 账户余额项
#[derive(Debug, Clone)]
pub struct AccountBalance {
    /// 账户信息
    pub account: Account,
    /// 各商品余额列表
    pub balances: Vec<(CommodityId, Decimal)>,
}

/// 资产负债表
#[derive(Debug, Clone)]
pub struct BalanceSheet {
    /// 资产类账户余额
    pub assets: Vec<AccountBalance>,
}

/// 资产负债表服务
pub struct BalanceSheetService {
    db: SqliteDatabase,
}

impl BalanceSheetService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 生成资产负债表
    ///
    /// 使用单条 SQL 统计所有资产账户余额，然后按账户分组。
    pub async fn balance_sheet(&self) -> Result<BalanceSheet, AccountingError> {
        let rows = self
            .db
            .posting_sum_all_assets()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let accounts = self
            .db
            .account_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let account_map: HashMap<AccountId, Account> =
            accounts.into_iter().map(|a| (a.id, a)).collect();

        let mut asset_map: HashMap<AccountId, Vec<(CommodityId, Decimal)>> = HashMap::new();
        for (account_id, commodity_id, balance) in rows {
            asset_map
                .entry(account_id)
                .or_default()
                .push((commodity_id, balance));
        }

        let assets: Vec<AccountBalance> = asset_map
            .into_iter()
            .filter_map(|(account_id, balances)| {
                account_map.get(&account_id).map(|account| AccountBalance {
                    account: account.clone(),
                    balances,
                })
            })
            .collect();

        Ok(BalanceSheet { assets })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::id::{AccountId, CommodityId, PostingId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::{Transaction, TransactionKind};
    use accounting_sql::SqliteDatabase;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn sample_account(name: &str, parent_id: Option<AccountId>) -> Account {
        Account {
            id: AccountId(0),
            name: name.to_string(),
            parent_id,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    fn sample_posting(account_id: AccountId, amount: &str) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id,
            commodity_id: CommodityId(1),
            amount: Decimal::from_str(amount).unwrap(),
            cost: None,
            cost_commodity_id: None,
            description: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        }
    }

    async fn setup_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_balance_sheet_only_assets() {
        let db = setup_db().await;
        let service = BalanceSheetService::new(db.clone());

        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;
        let equity_id = db
            .account_get_by_name("Equity:OpeningBalances")
            .await
            .unwrap()
            .unwrap()
            .id;

        let bank = sample_account("Bank", Some(assets_id));
        let bank_id = db.account_create_with_closure(&bank).await.unwrap();

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Initial balance".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let tx_id = db.transaction_insert(&tx, &[]).await.unwrap();

        let mut p1 = sample_posting(bank_id, "100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(equity_id, "-100");
        p2.transaction_id = tx_id;
        db.posting_insert(&p1).await.unwrap();
        db.posting_insert(&p2).await.unwrap();

        let sheet = service.balance_sheet().await.unwrap();

        assert_eq!(sheet.assets.len(), 1);
        assert_eq!(sheet.assets[0].account.name, "Bank");
        assert_eq!(sheet.assets[0].balances.len(), 1);
        assert_eq!(
            sheet.assets[0].balances[0].1,
            Decimal::from_str("100").unwrap()
        );
    }

    #[tokio::test]
    async fn test_balance_sheet_excludes_zero_balance() {
        let db = setup_db().await;
        let service = BalanceSheetService::new(db.clone());

        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;

        let bank = sample_account("Bank", Some(assets_id));
        let bank_id = db.account_create_with_closure(&bank).await.unwrap();

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Zero balance".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let tx_id = db.transaction_insert(&tx, &[]).await.unwrap();

        let mut p1 = sample_posting(bank_id, "100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(bank_id, "-100");
        p2.transaction_id = tx_id;
        db.posting_insert(&p1).await.unwrap();
        db.posting_insert(&p2).await.unwrap();

        let sheet = service.balance_sheet().await.unwrap();
        assert!(sheet.assets.is_empty());
    }
}

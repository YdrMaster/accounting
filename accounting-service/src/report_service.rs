use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::error::AccountingError;
use accounting::id::{AccountId, CommodityId};
use accounting_sql::database::Database;
use accounting_sql::transaction::Transaction;
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
    /// 负债类账户余额
    pub liabilities: Vec<AccountBalance>,
    /// 权益类账户余额
    pub equity: Vec<AccountBalance>,
}

/// 损益表
#[derive(Debug, Clone)]
pub struct IncomeStatement {
    /// 收入类账户余额
    pub income: Vec<AccountBalance>,
    /// 支出类账户余额
    pub expenses: Vec<AccountBalance>,
}

/// 报告服务
pub struct ReportService<D: Database> {
    db: D,
}

impl<D: Database> ReportService<D> {
    /// 创建服务实例
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 获取账户余额（含子账户聚合）
    pub async fn get_balance(
        &self,
        account_id: AccountId,
    ) -> Result<HashMap<CommodityId, Decimal>, AccountingError> {
        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 通过闭包表聚合查询余额
        let totals = tx
            .posting_repo()
            .sum_with_ancestors(&tx.conn(), account_id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(totals.into_iter().collect())
    }

    /// 资产负债表
    pub async fn balance_sheet(&self) -> Result<BalanceSheet, AccountingError> {
        let conn = self.db.connection();
        let accounts = self
            .db
            .account_repo()
            .list(&conn)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut assets = Vec::new();
        let mut liabilities = Vec::new();
        let mut equity = Vec::new();

        for account in accounts {
            let balances = self
                .db
                .posting_repo()
                .sum_by_account(&conn, account.id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if balances.iter().all(|(_, b)| b.is_zero()) {
                continue;
            }
            let item = AccountBalance {
                account: account.clone(),
                balances,
            };
            match account.account_type {
                AccountType::Asset => assets.push(item),
                AccountType::Liability => liabilities.push(item),
                AccountType::Equity => equity.push(item),
                _ => {}
            }
        }

        Ok(BalanceSheet {
            assets,
            liabilities,
            equity,
        })
    }

    /// 损益表
    pub async fn income_statement(&self) -> Result<IncomeStatement, AccountingError> {
        let conn = self.db.connection();
        let accounts = self
            .db
            .account_repo()
            .list(&conn)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut income = Vec::new();
        let mut expenses = Vec::new();

        for account in accounts {
            let balances = self
                .db
                .posting_repo()
                .sum_by_account(&conn, account.id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if balances.iter().all(|(_, b)| b.is_zero()) {
                continue;
            }
            let item = AccountBalance {
                account: account.clone(),
                balances,
            };
            match account.account_type {
                AccountType::Income => income.push(item),
                AccountType::Expense => expenses.push(item),
                _ => {}
            }
        }

        Ok(IncomeStatement { income, expenses })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::account_type::AccountType;
    use accounting::id::{AccountId, CommodityId, PostingId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::Transaction;
    use accounting_sql::impls::sqlite::SqliteDatabase;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn sample_account(name: &str, account_type: AccountType) -> Account {
        Account {
            id: AccountId(0),
            full_name: name.to_string(),
            account_type,
            parent_id: None,
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
            commodity_id: CommodityId(1), // CNY seed
            amount: Decimal::from_str(amount).unwrap(),
            cost: None,
            cost_commodity_id: None,
            description: None,
            member_id: None,
            channel_id: None,
        }
    }

    #[tokio::test]
    async fn test_get_balance() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let report_service = ReportService::new(db);

        let a1 = sample_account("Assets:I", AccountType::Asset);
        let a2 = sample_account("Assets:J", AccountType::Asset);
        let id1 = report_service
            .db
            .account_repo()
            .create(&report_service.db.connection(), &a1)
            .unwrap();
        let id2 = report_service
            .db
            .account_repo()
            .create(&report_service.db.connection(), &a2)
            .unwrap();

        // 直接通过 repo 插入交易和分录
        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Report test".to_string(),
            member_id: None,
            is_template: false,
        };
        let tx_id = report_service
            .db
            .transaction_repo()
            .insert(&report_service.db.connection(), &tx, &[])
            .unwrap();

        let mut p1 = sample_posting(id1, "100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(id2, "-100");
        p2.transaction_id = tx_id;

        report_service
            .db
            .posting_repo()
            .insert(&report_service.db.connection(), &p1)
            .unwrap();
        report_service
            .db
            .posting_repo()
            .insert(&report_service.db.connection(), &p2)
            .unwrap();

        let balance = report_service.get_balance(id1).await.unwrap();
        assert_eq!(balance[&CommodityId(1)], Decimal::from_str("100").unwrap());
    }
}

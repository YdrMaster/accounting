//! 资金流量表

use accounting::account::Account;
use accounting::error::AccountingError;
use accounting::finance_period::FinancePeriod;
use accounting::id::{AccountId, CommodityId, TagId};
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 资金流量项
#[derive(Debug, Clone)]
pub struct CashFlowItem {
    /// 账户信息
    pub account: Account,
    /// 流入（正金额之和）
    pub inflow: Decimal,
    /// 流出（负金额绝对值之和）
    pub outflow: Decimal,
    /// 净额 = inflow - outflow
    pub net: Decimal,
}

/// 总资产汇总
#[derive(Debug, Clone)]
pub struct CashFlowTotal {
    /// 总流入
    pub inflow: Decimal,
    /// 总流出
    pub outflow: Decimal,
    /// 总净额
    pub net: Decimal,
}

/// 资金流量表
#[derive(Debug, Clone)]
pub struct CashFlowReport {
    /// 周期起始日期
    pub period_start: NaiveDate,
    /// 周期结束日期
    pub period_end: NaiveDate,
    /// 各资产账户流量明细
    pub items: Vec<CashFlowItem>,
    /// 总资产汇总
    pub total: CashFlowTotal,
}

/// 资金流量表服务
pub struct CashFlowService {
    db: SqliteDatabase,
}

impl CashFlowService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 生成资金流量表
    ///
    /// 统计指定周期内每个资产账户的流入、流出、净额，以及总资产汇总。
    pub async fn cash_flow_report(
        &self,
        date: NaiveDate,
        period: FinancePeriod,
        commodity_id: CommodityId,
    ) -> Result<CashFlowReport, AccountingError> {
        let (period_start, period_end) = period.period_range(date);

        let accounts = self
            .db
            .account_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let asset_accounts: Vec<Account> = self.get_asset_accounts(&accounts).await?;

        let account_ids: Vec<AccountId> = asset_accounts.iter().map(|a| a.id).collect();
        let exclude_tag_ids = self.get_exclude_budget_tag_ids().await?;

        let sums = self
            .db
            .posting_sum_by_period(
                &account_ids,
                period_start,
                period_end,
                &exclude_tag_ids,
                commodity_id,
            )
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let sum_map: HashMap<AccountId, Decimal> = sums.into_iter().collect();
        let account_map: HashMap<AccountId, Account> =
            asset_accounts.into_iter().map(|a| (a.id, a)).collect();

        let mut items: Vec<CashFlowItem> = sum_map
            .into_iter()
            .filter_map(|(account_id, net)| {
                account_map.get(&account_id).map(|account| {
                    let inflow = if net.is_sign_positive() {
                        net
                    } else {
                        Decimal::ZERO
                    };
                    let outflow = if net.is_sign_negative() {
                        net.abs()
                    } else {
                        Decimal::ZERO
                    };
                    CashFlowItem {
                        account: account.clone(),
                        inflow,
                        outflow,
                        net,
                    }
                })
            })
            .collect();

        items.sort_by(|a, b| a.account.name.cmp(&b.account.name));

        let total = CashFlowTotal {
            inflow: items.iter().map(|i| i.inflow).sum(),
            outflow: items.iter().map(|i| i.outflow).sum(),
            net: items.iter().map(|i| i.net).sum(),
        };

        Ok(CashFlowReport {
            period_start,
            period_end,
            items,
            total,
        })
    }

    async fn get_asset_accounts(
        &self,
        accounts: &[Account],
    ) -> Result<Vec<Account>, AccountingError> {
        let mut asset_accounts = Vec::new();
        for account in accounts {
            let root_name = self
                .db
                .account_find_root_name(account.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if root_name == "Assets" || root_name == "资产" {
                asset_accounts.push(account.clone());
            }
        }
        Ok(asset_accounts)
    }

    async fn get_exclude_budget_tag_ids(&self) -> Result<Vec<TagId>, AccountingError> {
        let tags = self
            .db
            .tag_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let exclude_names = ["exclude-from-budget", "不计预算"];
        Ok(tags
            .iter()
            .filter(|t| exclude_names.contains(&t.name.as_str()))
            .map(|t| t.id)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::id::{AccountId, MemberId, PostingId, TransactionId};
    use accounting::member::Member;
    use accounting::posting::Posting;
    use accounting::transaction::{Transaction, TransactionKind};
    use accounting_sql::SqliteDatabase;
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

    async fn create_test_member(db: &SqliteDatabase) -> MemberId {
        db.member_create(&Member {
            id: MemberId(0),
            name: "Test".to_string(),
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_cash_flow_report() {
        let db = setup_db().await;
        let service = CashFlowService::new(db.clone());
        let member_id = create_test_member(&db).await;

        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;
        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;

        let bank = sample_account("Bank", Some(assets_id));
        let food = sample_account("Food", Some(expenses_id));
        let bank_id = db.account_create_with_closure(&bank).await.unwrap();
        let food_id = db.account_create_with_closure(&food).await.unwrap();

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 15)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Expense".to_string(),
            kind: TransactionKind::Normal,
            member_id,
        };
        let tx_id = db.transaction_insert(&tx, &[]).await.unwrap();

        let mut p1 = sample_posting(bank_id, "-100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(food_id, "100");
        p2.transaction_id = tx_id;
        db.posting_insert(&p1).await.unwrap();
        db.posting_insert(&p2).await.unwrap();

        let report = service
            .cash_flow_report(
                NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                FinancePeriod::Monthly,
                CommodityId(1),
            )
            .await
            .unwrap();

        assert_eq!(
            report.period_start,
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()
        );
        assert_eq!(
            report.period_end,
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()
        );
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].account.name, "Bank");
        assert_eq!(report.items[0].outflow, Decimal::from_str("100").unwrap());
        assert_eq!(report.items[0].net, Decimal::from_str("-100").unwrap());
        assert_eq!(report.total.outflow, Decimal::from_str("100").unwrap());
        assert_eq!(report.total.net, Decimal::from_str("-100").unwrap());
    }
}

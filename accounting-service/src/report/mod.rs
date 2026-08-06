//! 报表模块

use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::error::AccountingError;
use accounting::id::{AccountId, CommodityId};
use accounting_sql::SqliteDatabase;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

/// 资产负债表
pub mod balance_sheet;
/// 预算执行表
pub mod budget;
/// 资金流量表
pub mod cash_flow;
/// 按天收支汇总
pub mod daily_summary;
/// 资产趋势表
pub mod net_worth_trend;
/// 攒钱计划状态表
pub mod saving_plan;

/// 加载所有账户（id → Account）
pub(crate) async fn load_accounts(
    db: &SqliteDatabase,
) -> Result<HashMap<AccountId, Account>, AccountingError> {
    let list = db
        .account_list()
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    Ok(list.into_iter().map(|a| (a.id, a)).collect())
}

/// 加载所有币种 ID
pub(crate) async fn load_commodity_ids(
    db: &SqliteDatabase,
) -> Result<HashSet<CommodityId>, AccountingError> {
    let list = db
        .commodity_list()
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    Ok(list.into_iter().map(|c| c.id).collect())
}

/// 加载所有账户的类型（由根账户名推导；根账户名固定按 en 解析，
/// 无法推导类型的账户不出现在结果中，校验时视为账户不存在）
pub(crate) async fn load_account_types(
    db: &SqliteDatabase,
) -> Result<HashMap<AccountId, AccountType>, AccountingError> {
    let accounts = load_accounts(db).await?;
    let ids: Vec<AccountId> = accounts.keys().copied().collect();
    let root_names = db
        .account_root_names_by_ids(&ids, accounting::name::lang::EN)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    Ok(root_names
        .into_iter()
        .filter_map(|(id, root_name)| {
            AccountType::from_str(&root_name)
                .ok()
                .map(|account_type| (id, account_type))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting_sql::SqliteDatabase;

    fn bare_account(parent_id: Option<AccountId>) -> Account {
        Account {
            id: AccountId(0),
            parent_id,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    async fn account_id_by_name(db: &SqliteDatabase, name: &str) -> AccountId {
        db.account_get_by_name(name).await.unwrap().unwrap().id
    }

    /// load_account_types 的批量解析结果必须与此前的逐账户解析完全等价：
    /// 多类型混合账户映射正确，根账户名无法推导类型的账户不出现在结果中。
    #[tokio::test]
    async fn test_load_account_types_equivalent_to_per_account_resolution() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();

        // 多类型混合：Assets / Expenses 下各挂子账户
        let assets_id = account_id_by_name(&db, "Assets").await;
        let expenses_id = account_id_by_name(&db, "Expenses").await;
        let bank_id = db
            .account_create_with_name(&bare_account(Some(assets_id)), "Bank", "en")
            .await
            .unwrap();
        let food_id = db
            .account_create_with_name(&bare_account(Some(expenses_id)), "Food", "en")
            .await
            .unwrap();
        // 非系统根账户（根名 "Foo" 无法推导类型）及其子账户
        let custom_root_id = db
            .account_create_with_name(&bare_account(None), "Foo", "en")
            .await
            .unwrap();
        let custom_child_id = db
            .account_create_with_name(&bare_account(Some(custom_root_id)), "Bar", "en")
            .await
            .unwrap();

        let batch = load_account_types(&db).await.unwrap();

        // 逐账户解析（变更前的实现方式）作为等价基准
        let accounts = load_accounts(&db).await.unwrap();
        let mut expected = HashMap::new();
        for id in accounts.keys() {
            let root_name = db
                .account_find_root_name(*id, accounting::name::lang::EN)
                .await
                .unwrap();
            if let Ok(account_type) = AccountType::from_str(&root_name) {
                expected.insert(*id, account_type);
            }
        }

        assert_eq!(batch, expected);
        // 多类型混合映射正确
        assert_eq!(batch.get(&bank_id), Some(&AccountType::Asset));
        assert_eq!(batch.get(&food_id), Some(&AccountType::Expense));
        // 无法推导类型的账户（含其子账户）不出现在结果中
        assert!(!batch.contains_key(&custom_root_id));
        assert!(!batch.contains_key(&custom_child_id));
    }
}

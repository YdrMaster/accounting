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
    let mut types = HashMap::new();
    for id in accounts.keys() {
        let root_name = db
            .account_find_root_name(*id, accounting::name::lang::EN)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        if let Ok(account_type) = AccountType::from_str(&root_name) {
            types.insert(*id, account_type);
        }
    }
    Ok(types)
}

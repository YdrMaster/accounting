use accounting::error::AccountingError;
use accounting::id::{AccountId, BudgetId, ChannelId, CommodityId, MemberId};
use accounting_sql::SqliteDatabase;

/// 解析成员名称到 MemberId
pub async fn resolve_member(
    db: &SqliteDatabase,
    name: &str,
) -> Result<MemberId, AccountingError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AccountingError::InvalidTransaction(
            "成员名称不能为空".to_string(),
        ));
    }
    let member = db
        .member_get_by_name(name)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    member
        .map(|m| m.id)
        .ok_or_else(|| AccountingError::InvalidTransaction(format!("成员 '{}' 不存在", name)))
}

/// 解析账户路径到 AccountId
pub async fn resolve_account(
    db: &SqliteDatabase,
    path: &str,
) -> Result<AccountId, AccountingError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(AccountingError::InvalidTransaction(
            "账户路径不能为空".to_string(),
        ));
    }
    let account = db
        .account_get_by_name(path)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    account
        .map(|a| a.id)
        .ok_or_else(|| AccountingError::AccountNotFound(path.to_string()))
}

/// 解析渠道名称到 ChannelId
pub async fn resolve_channel(
    db: &SqliteDatabase,
    name: &str,
) -> Result<ChannelId, AccountingError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AccountingError::InvalidTransaction(
            "渠道名称不能为空".to_string(),
        ));
    }
    let channel = db
        .channel_get_by_name(name)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    channel
        .map(|c| c.id)
        .ok_or_else(|| AccountingError::InvalidTransaction(format!("渠道 '{}' 不存在", name)))
}

/// 解析币种符号到 CommodityId
pub async fn resolve_commodity(
    db: &SqliteDatabase,
    symbol: &str,
) -> Result<CommodityId, AccountingError> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return Err(AccountingError::InvalidTransaction(
            "币种符号不能为空".to_string(),
        ));
    }
    let commodity = db
        .commodity_get_by_symbol(symbol)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    commodity
        .map(|c| c.id)
        .ok_or_else(|| AccountingError::CommodityNotFound(symbol.to_string()))
}

/// 解析预算表名称到 BudgetId
pub async fn resolve_budget(
    db: &SqliteDatabase,
    name: &str,
) -> Result<BudgetId, AccountingError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AccountingError::InvalidTransaction(
            "预算表名称不能为空".to_string(),
        ));
    }
    let budget = db
        .budget_get_by_name(name)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    budget
        .map(|b| b.id)
        .ok_or_else(|| AccountingError::InvalidTransaction(format!("预算表 '{}' 不存在", name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("zh-CN")).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_resolve_member_not_found() {
        let db = setup_db().await;
        let err = resolve_member(&db, "Alice").await.unwrap_err();
        assert!(matches!(err, AccountingError::InvalidTransaction(_)));
        assert!(err.to_string().contains("Alice"));
    }

    #[tokio::test]
    async fn test_resolve_member_empty() {
        let db = setup_db().await;
        let err = resolve_member(&db, "   ").await.unwrap_err();
        assert!(matches!(err, AccountingError::InvalidTransaction(_)));
    }

    #[tokio::test]
    async fn test_resolve_account_found() {
        let db = setup_db().await;
        let id = resolve_account(&db, "资产").await.unwrap();
        assert!(id.0 > 0);
    }

    #[tokio::test]
    async fn test_resolve_account_not_found() {
        let db = setup_db().await;
        let err = resolve_account(&db, "Assets:NotExist").await.unwrap_err();
        assert!(matches!(err, AccountingError::AccountNotFound(_)));
    }

    #[tokio::test]
    async fn test_resolve_commodity_found() {
        let db = setup_db().await;
        let id = resolve_commodity(&db, "CNY").await.unwrap();
        assert!(id.0 > 0);
    }

    #[tokio::test]
    async fn test_resolve_channel_found() {
        let db = setup_db().await;
        let id = resolve_channel(&db, "支付宝").await.unwrap();
        assert!(id.0 > 0);
    }
}

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

use crate::error::DbError;
use crate::transaction::SqliteTransaction;

/// SQLite 数据库入口
#[derive(Clone)]
pub struct SqliteDatabase {
    pool: SqlitePool,
}

/// 生成七类实体的批量显示名解析包装（回退链见 `names::EntityNames`）
macro_rules! display_name_wrappers {
    ($($fn_name:ident: $id_ty:ident => $names:ident,)*) => {
        $(
            /// 批量解析显示名，返回 实体 ID → 显示名 的映射（无名字的实体不在映射中）
            pub async fn $fn_name(
                &self,
                ids: &[accounting::id::$id_ty],
                lang: &str,
            ) -> Result<std::collections::HashMap<accounting::id::$id_ty, String>, DbError> {
                let mut conn = self.acquire().await?;
                let raw: Vec<i64> = ids.iter().map(|id| id.0).collect();
                let map = crate::names::$names
                    .batch_resolve_display(&mut conn, &raw, lang)
                    .await?;
                Ok(map
                    .into_iter()
                    .map(|(k, v)| (accounting::id::$id_ty(k), v))
                    .collect())
            }
        )*
    };
}

impl SqliteDatabase {
    /// 打开文件数据库并自动初始化 schema
    pub async fn open(path: &str) -> Result<Self, DbError> {
        let path = std::path::Path::new(path);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| DbError::Database(e.to_string()))?
                .join(path)
        };
        let path = path.to_str().ok_or_else(|| {
            DbError::Database("database path contains invalid UTF-8 characters".to_string())
        })?;

        let options = SqliteConnectOptions::from_str(path)
            .map_err(|e| DbError::Database(e.to_string()))?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        let mut conn = pool
            .acquire()
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        crate::schema::initialize_schema(&mut conn).await?;
        Ok(Self { pool })
    }

    /// 打开内存数据库并自动初始化 schema
    pub async fn open_in_memory() -> Result<Self, DbError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(|e| DbError::Database(e.to_string()))?
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        let mut conn = pool
            .acquire()
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        crate::schema::initialize_schema(&mut conn).await?;
        Ok(Self { pool })
    }

    /// 开始事务
    pub async fn transaction(&self) -> Result<SqliteTransaction<'static>, DbError> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        Ok(SqliteTransaction::new(tx))
    }

    /// 初始化数据库种子数据（语言不再是库的属性）
    pub async fn initialize(&self) -> Result<(), DbError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        // 检查是否已初始化（commodities 表有数据说明已 seed）
        let seeded: Option<i64> = sqlx::query_scalar("SELECT 1 FROM commodities LIMIT 1")
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        if seeded.is_none() {
            crate::schema::insert_seed_data(&mut conn).await?;
        }
        Ok(())
    }

    /// 检查数据库是否已初始化（commodities 表有数据说明已 seed）
    pub async fn is_initialized(&self) -> Result<bool, DbError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        let seeded: Option<i64> = sqlx::query_scalar("SELECT 1 FROM commodities LIMIT 1")
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        Ok(seeded.is_some())
    }

    // === 显示名解析（回退链：所选语言 → en → zh-CN → 插入序） ===

    display_name_wrappers! {
        account_display_names: AccountId => ACCOUNT_NAMES,
        tag_display_names: TagId => TAG_NAMES,
        channel_display_names: ChannelId => CHANNEL_NAMES,
        commodity_display_names: CommodityId => COMMODITY_NAMES,
        member_display_names: MemberId => MEMBER_NAMES,
        budget_display_names: BudgetId => BUDGET_NAMES,
        saving_plan_display_names: SavingPlanId => SAVING_PLAN_NAMES,
    }

    // === Account ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Commodity ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Member ===
    // 委托方法已由 repo/facade.rs 的清单宏生成
    // 委托方法已由 repo/facade.rs 的清单宏生成

    /// 账户改名：先校验改名合法性再执行（复合流程，不入清单宏）
    pub async fn account_rename(
        &self,
        id: accounting::id::AccountId,
        new_name: &str,
        lang: &str,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_ensure_rename_allowed(&mut conn, id).await?;
        crate::repo::account::account_rename(&mut conn, id, new_name, lang).await
    }

    // === Channel ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === ChannelPath ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Tag ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Attachment ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Transaction ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Posting ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Settings ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Budget ===

    // === SavingPlan ===

    // === Account Mapping ===
    // 委托方法已由 repo/facade.rs 的清单宏生成

    // === Budget Statistics ===
    // 委托方法已由 repo/facade.rs 的清单宏生成
    // sum_by_account_with_descendants 已由 repo/facade.rs 的清单宏生成

    pub(crate) async fn acquire(
        &self,
    ) -> Result<sqlx::pool::PoolConnection<sqlx::Sqlite>, DbError> {
        self.pool
            .acquire()
            .await
            .map_err(|e| DbError::Database(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_open_creates_missing_file() {
        let path = "../target/test_open_creates_missing_file.db";
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{path}-wal"));
        let _ = std::fs::remove_file(format!("{path}-shm"));

        let db = SqliteDatabase::open(path).await.unwrap();
        drop(db);

        assert!(std::path::Path::new(path).exists());

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{path}-wal"));
        let _ = std::fs::remove_file(format!("{path}-shm"));
    }

    #[tokio::test]
    async fn test_initialize_is_idempotent() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();

        db.initialize().await.unwrap();
        // Second call should be a no-op (idempotent)
        db.initialize().await.unwrap();

        let root_names: Vec<String> =
            sqlx::query_scalar("SELECT an.name FROM accounts a JOIN account_names an ON an.account_id = a.id WHERE a.parent_id IS NULL AND an.lang = 'en' AND an.is_display = 1 ORDER BY an.name")
                .fetch_all(&db.pool)
                .await
                .unwrap();

        assert_eq!(root_names, vec!["Assets", "Equity", "Expenses", "Income"]);
    }

    fn bare_account(parent_id: Option<accounting::id::AccountId>) -> accounting::account::Account {
        accounting::account::Account {
            id: accounting::id::AccountId(0),
            parent_id,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    #[tokio::test]
    async fn test_account_root_names_by_ids_wrapper() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();

        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;
        let fees_id = db
            .account_get_by_name("Expenses:Fees")
            .await
            .unwrap()
            .unwrap()
            .id;
        let bank_id = db
            .account_create_with_name(&bare_account(Some(assets_id)), "Bank", "en")
            .await
            .unwrap();

        let en = db
            .account_root_names_by_ids(&[bank_id, fees_id], "en")
            .await
            .unwrap();
        assert_eq!(
            en,
            vec![
                (fees_id, "Expenses".to_string()),
                (bank_id, "Assets".to_string()),
            ]
        );

        // 事务包装路径
        let mut tx = db.transaction().await.unwrap();
        let zh = tx
            .account_root_names_by_ids(&[bank_id, fees_id], "zh-CN")
            .await
            .unwrap();
        assert_eq!(
            zh,
            vec![(fees_id, "支出".to_string()), (bank_id, "资产".to_string()),]
        );

        // 空输入
        assert!(
            db.account_root_names_by_ids(&[], "en")
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_account_rename_rejects_system_root_en() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;

        let err = db
            .account_rename(assets_id, "MyAssets", "en")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("system root account"),
            "unexpected error: {err}"
        );

        // 根账户名保持不变
        let names = db.account_display_names(&[assets_id], "en").await.unwrap();
        assert_eq!(names.get(&assets_id).unwrap(), "Assets");
    }

    #[tokio::test]
    async fn test_account_rename_rejects_system_root_zh_cn() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let expenses_id = db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id;

        let err = db
            .account_rename(expenses_id, "花销", "zh-CN")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("system root account"),
            "unexpected error: {err}"
        );

        // 事务路径同样拒绝
        let mut tx = db.transaction().await.unwrap();
        let err = tx
            .account_rename(expenses_id, "花销", "zh-CN")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("system root account"),
            "unexpected error: {err}"
        );
        drop(tx);

        // 根账户中文名保持不变
        let names = db
            .account_display_names(&[expenses_id], "zh-CN")
            .await
            .unwrap();
        assert_eq!(names.get(&expenses_id).unwrap(), "支出");
    }

    #[tokio::test]
    async fn test_account_rename_non_root_ok() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;
        let bank_id = db
            .account_create_with_name(&bare_account(Some(assets_id)), "Bank", "en")
            .await
            .unwrap();

        db.account_rename(bank_id, "招商银行", "zh-CN")
            .await
            .unwrap();
        let names = db.account_display_names(&[bank_id], "zh-CN").await.unwrap();
        assert_eq!(names.get(&bank_id).unwrap(), "招商银行");
    }

    #[tokio::test]
    async fn test_saving_plan_wrappers() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();

        let assets_id = db.account_get_by_name("Assets").await.unwrap().unwrap().id;
        let account = accounting::account::Account {
            id: accounting::id::AccountId(0),
            parent_id: Some(assets_id),
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let account_id = db
            .account_create_with_name(&account, "Alipay", "en")
            .await
            .unwrap();

        let deadline = chrono::NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        let plan_id = db
            .saving_plan_create(
                "Trip Fund",
                None,
                Some(deadline),
                accounting::id::CommodityId(1),
                rust_decimal::Decimal::from_str("5000").unwrap(),
                &[account_id],
                "en",
            )
            .await
            .unwrap();

        let plan = db.saving_plan_get(plan_id).await.unwrap().unwrap();
        assert_eq!(plan.period, None);
        assert_eq!(plan.deadline, Some(deadline));
        assert_eq!(
            plan.target_amount,
            rust_decimal::Decimal::from_str("5000").unwrap()
        );

        // 显示名批量解析
        let names = db
            .saving_plan_display_names(&[plan_id], "zh-CN")
            .await
            .unwrap();
        assert_eq!(names.get(&plan_id).unwrap(), "Trip Fund");

        // 账户集合
        let accounts = db.saving_plan_get_accounts(plan_id).await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_id, account_id);

        // 余额聚合包装（无分录 → 0）
        let balance = db
            .account_balance_by_ids(
                &[account_id],
                accounting::id::CommodityId(1),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(balance, rust_decimal::Decimal::ZERO);

        // 分组余额包装（无分录 → 每个账户 0，顺序与输入一致）
        let balances = db
            .account_balances_by_ids(
                &[account_id],
                accounting::id::CommodityId(1),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(balances, vec![(account_id, rust_decimal::Decimal::ZERO)]);

        // list_all_with_accounts + 删除
        let all = db.saving_plan_list_all_with_accounts().await.unwrap();
        assert_eq!(all.len(), 1);
        db.saving_plan_delete(plan_id).await.unwrap();
        assert!(db.saving_plan_get(plan_id).await.unwrap().is_none());
    }
}

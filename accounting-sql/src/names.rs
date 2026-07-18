//! 名字按语言管理：六张同构名字表共享的查询与写路径逻辑
//!
//! - 显示名解析回退链：所选语言显示名 → en 显示名 → zh-CN 显示名 → 其余按 rowid 插入序第一条
//! - 命名空间唯一性：账户按父账户作用域（根账户全局），其余实体全局；不区分大小写
//! - 改名：按 (entity, lang) 定位显示名单条记录；系统名字（is_system=1）不可改文本，
//!   改名时降级为非显示名并插入用户自定义显示名

use sqlx::SqliteConnection;
use std::collections::HashMap;

use accounting::name::lang;

use crate::error::DbError;

/// 把仅由编译期常量（表名/列名）拼接而成的 SQL 标记为安全。
/// 所有运行时值一律走绑定参数，不参与拼接。
fn safe_sql(text: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(text)
}

/// 某类实体的名字表操作句柄。
///
/// 六张名字表（account_names、tag_names、channel_names、commodity_names、
/// member_names、budget_names）结构同构，共享这一套实现。
pub struct EntityNames {
    /// 名字表名
    table: &'static str,
    /// 实体外键列名
    entity_col: &'static str,
    /// 唯一性命名空间是否按父账户作用域（仅账户；其余实体为全局命名空间）
    parent_scoped: bool,
}

/// 账户名字表（命名空间为父账户作用域）
pub const ACCOUNT_NAMES: EntityNames = EntityNames::new("account_names", "account_id", true);
/// 标签名字表
pub const TAG_NAMES: EntityNames = EntityNames::new("tag_names", "tag_id", false);
/// 渠道名字表
pub const CHANNEL_NAMES: EntityNames = EntityNames::new("channel_names", "channel_id", false);
/// 币种名字表
pub const COMMODITY_NAMES: EntityNames = EntityNames::new("commodity_names", "commodity_id", false);
/// 成员名字表
pub const MEMBER_NAMES: EntityNames = EntityNames::new("member_names", "member_id", false);
/// 预算名字表
pub const BUDGET_NAMES: EntityNames = EntityNames::new("budget_names", "budget_id", false);

/// 显示名回退链的 ORDER BY 片段（`?2` 为所选语言）。
///
/// 注意：batch_resolve_display 中以绑定参数内联了同一段 CASE，两处必须保持同步。
const FALLBACK_ORDER_SQL: &str = "
    CASE
        WHEN lang = ?2 AND is_display = 1 THEN 1
        WHEN lang = 'en' AND is_display = 1 THEN 2
        WHEN lang = 'zh-CN' AND is_display = 1 THEN 3
        ELSE 4
    END,
    rowid
";

impl EntityNames {
    const fn new(table: &'static str, entity_col: &'static str, parent_scoped: bool) -> Self {
        Self {
            table,
            entity_col,
            parent_scoped,
        }
    }

    /// 解析单个实体的显示名（回退链：所选语言 → en → zh-CN → 其余按插入序）。
    ///
    /// 实体没有任何名字时返回 None。
    pub async fn resolve_display(
        &self,
        conn: &mut SqliteConnection,
        entity_id: i64,
        display_lang: &str,
    ) -> Result<Option<String>, DbError> {
        let sql = format!(
            "SELECT name FROM {} WHERE {} = ?1 ORDER BY {} LIMIT 1",
            self.table, self.entity_col, FALLBACK_ORDER_SQL
        );
        let name: Option<String> = sqlx::query_scalar(safe_sql(sql))
            .bind(entity_id)
            .bind(lang::normalize(display_lang))
            .fetch_optional(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        Ok(name)
    }

    /// 批量解析显示名，返回 entity_id → 显示名 的映射（无名字的实体不在映射中）。
    ///
    /// 单次查询取回所有候选名字，按回退链排序后每个实体取第一条，避免 N+1。
    pub async fn batch_resolve_display(
        &self,
        conn: &mut SqliteConnection,
        entity_ids: &[i64],
        display_lang: &str,
    ) -> Result<HashMap<i64, String>, DbError> {
        let mut map = HashMap::new();
        if entity_ids.is_empty() {
            return Ok(map);
        }

        let mut builder = sqlx::QueryBuilder::new("SELECT ");
        builder.push(self.entity_col);
        builder.push(", name FROM ");
        builder.push(self.table);
        builder.push(" WHERE ");
        builder.push(self.entity_col);
        builder.push(" IN (");
        let mut separated = builder.separated(", ");
        for id in entity_ids {
            separated.push_bind(id);
        }
        // 回退链排序，与 FALLBACK_ORDER_SQL 的 CASE 保持一致
        builder.push(") ORDER BY ");
        builder.push(self.entity_col);
        builder.push(", CASE WHEN lang = ");
        builder.push_bind(lang::normalize(display_lang));
        builder.push(
            " AND is_display = 1 THEN 1 WHEN lang = 'en' AND is_display = 1 THEN 2 WHEN lang = 'zh-CN' AND is_display = 1 THEN 3 ELSE 4 END, rowid",
        );

        let rows: Vec<(i64, String)> = builder
            .build_query_as()
            .fetch_all(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;

        // 每个实体的第一行即回退链最优名字
        for (entity_id, name) in rows {
            map.entry(entity_id).or_insert(name);
        }
        Ok(map)
    }

    /// 插入一条名字记录（lang 归一化后写入）。
    pub async fn insert(
        &self,
        conn: &mut SqliteConnection,
        entity_id: i64,
        name_lang: &str,
        name: &str,
        is_system: bool,
        is_display: bool,
    ) -> Result<(), DbError> {
        let sql = format!(
            "INSERT INTO {} ({}, lang, name, is_system, is_display) VALUES (?1, ?2, ?3, ?4, ?5)",
            self.table, self.entity_col
        );
        sqlx::query(safe_sql(sql))
            .bind(entity_id)
            .bind(lang::normalize(name_lang))
            .bind(name)
            .bind(is_system as i32)
            .bind(is_display as i32)
            .execute(conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        Ok(())
    }

    /// 校验名字在命名空间内可用（不区分大小写，含系统内置名字）。
    ///
    /// - 账户：命名空间为父账户（parent_id 为 None 表示根账户全局命名空间）
    /// - 其余实体：全局命名空间（parent_id 参数忽略）
    /// - exclude_entity_id 用于改名时排除实体自身的名字
    pub async fn ensure_available(
        &self,
        conn: &mut SqliteConnection,
        parent_id: Option<i64>,
        exclude_entity_id: Option<i64>,
        name: &str,
    ) -> Result<(), DbError> {
        let count: i64 = if self.parent_scoped {
            let sql = format!(
                "SELECT COUNT(*) FROM {} n JOIN accounts a ON a.id = n.{}
                 WHERE a.parent_id IS ?1 AND n.name = ?2 AND n.{} IS NOT ?3",
                self.table, self.entity_col, self.entity_col
            );
            sqlx::query_scalar(safe_sql(sql))
                .bind(parent_id)
                .bind(name)
                .bind(exclude_entity_id)
                .fetch_one(conn)
                .await
                .map_err(|e| DbError::Database(e.to_string()))?
        } else {
            let sql = format!(
                "SELECT COUNT(*) FROM {} WHERE name = ?1 AND {} IS NOT ?2",
                self.table, self.entity_col
            );
            sqlx::query_scalar(safe_sql(sql))
                .bind(name)
                .bind(exclude_entity_id)
                .fetch_one(conn)
                .await
                .map_err(|e| DbError::Database(e.to_string()))?
        };

        if count > 0 {
            return Err(DbError::Database(format!("名字 \"{}\" 已存在", name)));
        }
        Ok(())
    }

    /// 改名：把 (entity, lang) 的显示名改为 new_name；该语言尚无显示名时插入新显示名。
    ///
    /// - 当前显示名是系统名字（is_system=1）时，系统名文本不可改：
    ///   将其设为非显示名，插入用户自定义名字作为新显示名（系统名仍可命中）
    /// - 新名字须在命名空间内唯一（不区分大小写；parent_id 仅对账户有意义）
    pub async fn rename_display(
        &self,
        conn: &mut SqliteConnection,
        entity_id: i64,
        parent_id: Option<i64>,
        display_lang: &str,
        new_name: &str,
    ) -> Result<(), DbError> {
        self.ensure_available(conn, parent_id, Some(entity_id), new_name)
            .await?;

        // 目标名字已是该实体在该语言的名字 → 仅切换显示标记
        let sql = format!(
            "SELECT id FROM {} WHERE {} = ?1 AND lang = ?2 AND name = ?3",
            self.table, self.entity_col
        );
        let existing: Option<i64> = sqlx::query_scalar(safe_sql(sql))
            .bind(entity_id)
            .bind(lang::normalize(display_lang))
            .bind(new_name)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        if let Some(target_id) = existing {
            let sql = format!(
                "UPDATE {} SET is_display = 0 WHERE {} = ?1 AND lang = ?2 AND is_display = 1",
                self.table, self.entity_col
            );
            sqlx::query(safe_sql(sql))
                .bind(entity_id)
                .bind(lang::normalize(display_lang))
                .execute(&mut *conn)
                .await
                .map_err(|e| DbError::Database(e.to_string()))?;
            let sql = format!("UPDATE {} SET is_display = 1 WHERE id = ?1", self.table);
            sqlx::query(safe_sql(sql))
                .bind(target_id)
                .execute(conn)
                .await
                .map_err(|e| DbError::Database(e.to_string()))?;
            return Ok(());
        }

        let sql = format!(
            "SELECT id, is_system FROM {} WHERE {} = ?1 AND lang = ?2 AND is_display = 1",
            self.table, self.entity_col
        );
        let row: Option<(i64, i32)> = sqlx::query_as(safe_sql(sql))
            .bind(entity_id)
            .bind(lang::normalize(display_lang))
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;

        match row {
            Some((name_id, is_system)) if is_system != 0 => {
                let sql = format!("UPDATE {} SET is_display = 0 WHERE id = ?1", self.table);
                sqlx::query(safe_sql(sql))
                    .bind(name_id)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| DbError::Database(e.to_string()))?;
                self.insert(conn, entity_id, display_lang, new_name, false, true)
                    .await
            }
            Some((name_id, _)) => {
                let sql = format!("UPDATE {} SET name = ?1 WHERE id = ?2", self.table);
                sqlx::query(safe_sql(sql))
                    .bind(new_name)
                    .bind(name_id)
                    .execute(conn)
                    .await
                    .map_err(|e| DbError::Database(e.to_string()))?;
                Ok(())
            }
            None => {
                self.insert(conn, entity_id, display_lang, new_name, false, true)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    async fn setup() -> SqliteConnection {
        let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
            .await
            .unwrap();
        crate::schema::initialize_schema(&mut conn).await.unwrap();
        crate::schema::insert_seed_data(&mut conn).await.unwrap();
        conn
    }

    /// 创建一个账户实体（不带名字），返回 id
    async fn insert_account(conn: &mut SqliteConnection, parent_id: Option<i64>) -> i64 {
        sqlx::query_scalar(
            "INSERT INTO accounts (parent_id, is_system) VALUES (?1, 0) RETURNING id",
        )
        .bind(parent_id)
        .fetch_one(conn)
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_fallback_chain_four_levels() {
        let mut conn = setup().await;

        // 第 1 级：有所选语言显示名
        let a1 = insert_account(&mut conn, None).await;
        ACCOUNT_NAMES
            .insert(&mut conn, a1, "en", "Cash", false, true)
            .await
            .unwrap();
        ACCOUNT_NAMES
            .insert(&mut conn, a1, "zh-CN", "现金", false, true)
            .await
            .unwrap();

        // 第 2 级：只有 en 显示名，以中文查询回退到英文
        let a2 = insert_account(&mut conn, None).await;
        ACCOUNT_NAMES
            .insert(&mut conn, a2, "en", "Wallet", false, true)
            .await
            .unwrap();

        // 第 3 级：只有 zh-CN 显示名，以英文查询回退到中文
        let a3 = insert_account(&mut conn, None).await;
        ACCOUNT_NAMES
            .insert(&mut conn, a3, "zh-CN", "钱包", false, true)
            .await
            .unwrap();

        // 第 4 级：只有 und 名字，回退到插入序第一条
        let a4 = insert_account(&mut conn, None).await;
        ACCOUNT_NAMES
            .insert(&mut conn, a4, "und", "餐饮美食", false, true)
            .await
            .unwrap();

        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, a1, "zh-CN")
                .await
                .unwrap(),
            Some("现金".to_string())
        );
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, a1, "en")
                .await
                .unwrap(),
            Some("Cash".to_string())
        );
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, a2, "zh-CN")
                .await
                .unwrap(),
            Some("Wallet".to_string())
        );
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, a3, "en")
                .await
                .unwrap(),
            Some("钱包".to_string())
        );
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, a4, "en")
                .await
                .unwrap(),
            Some("餐饮美食".to_string())
        );

        // 批量解析与单条结果一致
        let map = ACCOUNT_NAMES
            .batch_resolve_display(&mut conn, &[a1, a2, a3, a4], "zh-CN")
            .await
            .unwrap();
        assert_eq!(map.get(&a1).unwrap(), "现金");
        assert_eq!(map.get(&a2).unwrap(), "Wallet");
        assert_eq!(map.get(&a3).unwrap(), "钱包");
        assert_eq!(map.get(&a4).unwrap(), "餐饮美食");

        let map_en = ACCOUNT_NAMES
            .batch_resolve_display(&mut conn, &[a1, a2, a3, a4], "en")
            .await
            .unwrap();
        assert_eq!(map_en.get(&a1).unwrap(), "Cash");
        assert_eq!(map_en.get(&a3).unwrap(), "钱包");
    }

    #[tokio::test]
    async fn test_lang_normalized_on_insert_and_resolve() {
        let mut conn = setup().await;
        let id = insert_account(&mut conn, None).await;
        // zh 归一为 zh-CN 存储
        ACCOUNT_NAMES
            .insert(&mut conn, id, "zh", "现金", false, true)
            .await
            .unwrap();
        let stored: String =
            sqlx::query_scalar("SELECT lang FROM account_names WHERE account_id = ?1")
                .bind(id)
                .fetch_one(&mut conn)
                .await
                .unwrap();
        assert_eq!(stored, "zh-CN");
        // 以 zh-* 查询同样命中
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, id, "zh-TW")
                .await
                .unwrap(),
            Some("现金".to_string())
        );
    }

    #[tokio::test]
    async fn test_ensure_available_account_parent_scope() {
        let mut conn = setup().await;
        let assets = insert_account(&mut conn, None).await;
        let expenses = insert_account(&mut conn, None).await;

        // 与 Assets 下系统名 Cash 撞（NOCASE + 系统名）→ 拒绝
        let assets_id: i64 = sqlx::query_scalar(
            "SELECT a.id FROM accounts a JOIN account_names an ON an.account_id = a.id
             WHERE a.parent_id IS NULL AND an.name = 'Assets' AND an.is_system = 1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        let expenses_id: i64 = sqlx::query_scalar(
            "SELECT a.id FROM accounts a JOIN account_names an ON an.account_id = a.id
             WHERE a.parent_id IS NULL AND an.name = 'Expenses' AND an.is_system = 1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert!(
            ACCOUNT_NAMES
                .ensure_available(&mut conn, Some(assets_id), None, "cash")
                .await
                .is_err(),
            "同父级撞系统名（NOCASE）应拒绝"
        );
        // 不同父级允许同名
        assert!(
            ACCOUNT_NAMES
                .ensure_available(&mut conn, Some(expenses_id), None, "Cash")
                .await
                .is_ok(),
            "不同父级应允许同名"
        );
        // 根命名空间撞系统根账户名 → 拒绝
        assert!(
            ACCOUNT_NAMES
                .ensure_available(&mut conn, None, None, "assets")
                .await
                .is_err(),
            "根命名空间撞系统名应拒绝"
        );
        // 普通可用名字
        assert!(
            ACCOUNT_NAMES
                .ensure_available(&mut conn, Some(assets), None, "招商")
                .await
                .is_ok()
        );
        assert!(
            ACCOUNT_NAMES
                .ensure_available(&mut conn, Some(expenses), None, "招商")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_ensure_available_global_scope() {
        let mut conn = setup().await;
        // 标签全局命名空间：撞系统名（NOCASE）→ 拒绝
        assert!(
            TAG_NAMES
                .ensure_available(&mut conn, None, None, "PENDING")
                .await
                .is_err()
        );
        assert!(
            TAG_NAMES
                .ensure_available(&mut conn, None, None, "待处理")
                .await
                .is_err()
        );
        assert!(
            TAG_NAMES
                .ensure_available(&mut conn, None, None, "旅行")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_rename_display_updates_user_name() {
        let mut conn = setup().await;
        let id = insert_account(&mut conn, None).await;
        ACCOUNT_NAMES
            .insert(&mut conn, id, "en", "OldName", false, true)
            .await
            .unwrap();

        ACCOUNT_NAMES
            .rename_display(&mut conn, id, None, "en", "NewName")
            .await
            .unwrap();
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, id, "en")
                .await
                .unwrap(),
            Some("NewName".to_string())
        );

        // 改另一种语言：该语言无显示名 → 插入新显示名，英文显示名不受影响
        ACCOUNT_NAMES
            .rename_display(&mut conn, id, None, "zh-CN", "新名字")
            .await
            .unwrap();
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, id, "zh-CN")
                .await
                .unwrap(),
            Some("新名字".to_string())
        );
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, id, "en")
                .await
                .unwrap(),
            Some("NewName".to_string())
        );
    }

    #[tokio::test]
    async fn test_rename_display_demotes_system_name() {
        let mut conn = setup().await;
        let assets_id: i64 = sqlx::query_scalar(
            "SELECT a.id FROM accounts a JOIN account_names an ON an.account_id = a.id
             WHERE a.parent_id IS NULL AND an.name = 'Assets' AND an.is_system = 1",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();

        // 系统名不可改文本：改名 = 系统名降级 + 用户自定义名成为显示名
        ACCOUNT_NAMES
            .rename_display(&mut conn, assets_id, None, "en", "MyAssets")
            .await
            .unwrap();
        assert_eq!(
            ACCOUNT_NAMES
                .resolve_display(&mut conn, assets_id, "en")
                .await
                .unwrap(),
            Some("MyAssets".to_string())
        );
        // 系统名文本保留、仍作为非显示名存在（任意名字可命中）
        let (name, is_display): (String, i32) = sqlx::query_as(
            "SELECT name, is_display FROM account_names
             WHERE account_id = ?1 AND lang = 'en' AND is_system = 1",
        )
        .bind(assets_id)
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(name, "Assets");
        assert_eq!(is_display, 0);
    }

    #[tokio::test]
    async fn test_rename_display_rejects_occupied_name() {
        let mut conn = setup().await;
        let id = insert_account(&mut conn, None).await;
        ACCOUNT_NAMES
            .insert(&mut conn, id, "en", "MyRoot", false, true)
            .await
            .unwrap();

        // 改为根命名空间内已存在的名字（NOCASE）→ 拒绝
        let result = ACCOUNT_NAMES
            .rename_display(&mut conn, id, None, "en", "assets")
            .await;
        assert!(result.is_err());
        // 改为自身现有名字（大小写不同）→ 允许
        assert!(
            ACCOUNT_NAMES
                .rename_display(&mut conn, id, None, "en", "myroot")
                .await
                .is_ok()
        );
    }
}

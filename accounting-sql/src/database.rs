use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

use crate::error::DbError;
use crate::transaction::SqliteTransaction;

/// SQLite 数据库入口
#[derive(Clone)]
pub struct SqliteDatabase {
    pool: SqlitePool,
}

/// 默认语言
pub const DEFAULT_LANG: &str = "zh-CN";

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

    /// 初始化数据库种子数据
    ///
    /// 若数据库已写入过种子数据（`settings.language` 已存在），则直接返回已保存的语言，
    /// 避免再次运行时按另一种语言追加第二套默认账户。
    ///
    /// `lang` 为 `None` 时使用 [`DEFAULT_LANG`](DEFAULT_LANG)。
    pub async fn initialize(&self, lang: Option<&str>) -> Result<String, DbError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| DbError::Database(e.to_string()))?;
        if let Some(saved) = crate::repo::get_setting(&mut conn, "language").await? {
            Ok(saved)
        } else {
            let lang = lang.unwrap_or(DEFAULT_LANG);
            crate::schema::insert_seed_data(&mut conn, lang).await?;
            crate::repo::set_setting(&mut conn, "language", lang).await?;
            Ok(lang.to_string())
        }
    }

    // === Account ===

    pub async fn account_create(
        &self,
        account: &accounting::account::Account,
    ) -> Result<accounting::id::AccountId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_create(&mut conn, account).await
    }

    pub async fn account_create_with_closure(
        &self,
        account: &accounting::account::Account,
    ) -> Result<accounting::id::AccountId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_create_with_closure(&mut conn, account).await
    }

    pub async fn account_get(
        &self,
        id: accounting::id::AccountId,
    ) -> Result<Option<accounting::account::Account>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_get(&mut conn, id).await
    }

    pub async fn account_get_by_name(
        &self,
        name: &str,
    ) -> Result<Option<accounting::account::Account>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_get_by_name(&mut conn, name).await
    }

    pub async fn account_get_by_parent_and_name(
        &self,
        parent_id: Option<accounting::id::AccountId>,
        name: &str,
    ) -> Result<Option<accounting::account::Account>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_get_by_parent_and_name(&mut conn, parent_id, name).await
    }

    pub async fn account_list(&self) -> Result<Vec<accounting::account::Account>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_list(&mut conn).await
    }

    pub async fn account_list_children(
        &self,
        parent_id: accounting::id::AccountId,
    ) -> Result<Vec<accounting::account::Account>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_list_children(&mut conn, parent_id).await
    }

    pub async fn account_rename(
        &self,
        id: accounting::id::AccountId,
        new_name: &str,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_rename(&mut conn, id, new_name).await
    }

    pub async fn account_close(&self, id: accounting::id::AccountId) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_close(&mut conn, id).await
    }

    pub async fn account_reopen(&self, id: accounting::id::AccountId) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_reopen(&mut conn, id).await
    }

    pub async fn account_delete(&self, id: accounting::id::AccountId) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_delete(&mut conn, id).await
    }

    pub async fn account_get_owners(
        &self,
        account_id: accounting::id::AccountId,
    ) -> Result<Vec<accounting::id::MemberId>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_get_owners(&mut conn, account_id).await
    }

    pub async fn account_set_owners(
        &self,
        account_id: accounting::id::AccountId,
        member_ids: &[accounting::id::MemberId],
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_set_owners(&mut conn, account_id, member_ids).await
    }

    pub async fn account_find_root_name(
        &self,
        account_id: accounting::id::AccountId,
    ) -> Result<String, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_find_root_name(&mut conn, account_id).await
    }

    pub async fn account_find_root_id(
        &self,
        account_id: accounting::id::AccountId,
    ) -> Result<accounting::id::AccountId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_find_root_id(&mut conn, account_id).await
    }

    pub async fn account_get_or_create_by_path(
        &self,
        path: &str,
    ) -> Result<accounting::id::AccountId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_get_or_create_by_path(&mut conn, path).await
    }

    pub async fn account_update_by_path(
        &self,
        path: &str,
        closed_at: Option<chrono::NaiveDate>,
        billing_day: Option<u8>,
        repayment_day: Option<u8>,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_update_by_path(
            &mut conn,
            path,
            closed_at,
            billing_day,
            repayment_day,
        )
        .await
    }

    pub async fn account_list_owners(
        &self,
    ) -> Result<Vec<(accounting::id::AccountId, accounting::id::MemberId)>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_list_owners(&mut conn).await
    }

    pub async fn account_rebuild_ancestors(&self) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account::account_rebuild_ancestors(&mut conn).await
    }

    // === Commodity ===

    pub async fn commodity_get_by_symbol(
        &self,
        symbol: &str,
    ) -> Result<Option<accounting::commodity::Commodity>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::commodity::commodity_get_by_symbol(&mut conn, symbol).await
    }

    pub async fn commodity_list(&self) -> Result<Vec<accounting::commodity::Commodity>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::commodity::commodity_list(&mut conn).await
    }

    pub async fn commodity_create(
        &self,
        commodity: &accounting::commodity::Commodity,
    ) -> Result<accounting::id::CommodityId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::commodity::commodity_create(&mut conn, commodity).await
    }

    pub async fn commodity_upsert_by_symbol(
        &self,
        symbol: &str,
        name: &str,
        precision: u8,
    ) -> Result<accounting::id::CommodityId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::commodity::commodity_upsert_by_symbol(&mut conn, symbol, name, precision).await
    }

    // === Member ===

    pub async fn member_create(
        &self,
        member: &accounting::member::Member,
    ) -> Result<accounting::id::MemberId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::member::member_create(&mut conn, member).await
    }

    pub async fn member_get(
        &self,
        id: accounting::id::MemberId,
    ) -> Result<Option<accounting::member::Member>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::member::member_get(&mut conn, id).await
    }

    pub async fn member_list(&self) -> Result<Vec<accounting::member::Member>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::member::member_list(&mut conn).await
    }

    pub async fn member_delete(&self, id: accounting::id::MemberId) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::member::member_delete(&mut conn, id).await
    }

    pub async fn member_get_or_create_by_name(
        &self,
        name: &str,
    ) -> Result<accounting::id::MemberId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::member::member_get_or_create_by_name(&mut conn, name).await
    }

    // === Channel ===

    pub async fn channel_create(
        &self,
        channel: &accounting::channel::Channel,
    ) -> Result<accounting::id::ChannelId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel::channel_create(&mut conn, channel).await
    }

    pub async fn channel_get(
        &self,
        id: accounting::id::ChannelId,
    ) -> Result<Option<accounting::channel::Channel>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel::channel_get(&mut conn, id).await
    }

    pub async fn channel_get_by_name(
        &self,
        name: &str,
    ) -> Result<Option<accounting::channel::Channel>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel::channel_get_by_name(&mut conn, name).await
    }

    pub async fn channel_list(&self) -> Result<Vec<accounting::channel::Channel>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel::channel_list(&mut conn).await
    }

    pub async fn channel_count_transactions_by_id(
        &self,
        channel_id: accounting::id::ChannelId,
    ) -> Result<i64, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel::channel_count_transactions_by_id(&mut conn, channel_id).await
    }

    pub async fn channel_force_delete_by_id(
        &self,
        channel_id: accounting::id::ChannelId,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel::channel_force_delete_by_id(&mut conn, channel_id).await
    }

    pub async fn channel_update(
        &self,
        id: accounting::id::ChannelId,
        account_id: Option<accounting::id::AccountId>,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel::channel_update(&mut conn, id, account_id).await
    }

    pub async fn channel_upsert_by_name(
        &self,
        name: &str,
        description: Option<&str>,
        account_id: Option<accounting::id::AccountId>,
    ) -> Result<accounting::id::ChannelId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel::channel_upsert_by_name(&mut conn, name, description, account_id).await
    }

    // === ChannelPath ===

    pub async fn channel_path_create(
        &self,
        transaction_id: accounting::id::TransactionId,
        node: &accounting::channel_path::ChannelPathNode,
    ) -> Result<accounting::id::ChannelPathId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel_path::channel_path_create(&mut conn, transaction_id, node).await
    }

    pub async fn channel_path_create_batch(
        &self,
        transaction_id: accounting::id::TransactionId,
        nodes: &[accounting::channel_path::ChannelPathNode],
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel_path::channel_path_create_batch(&mut conn, transaction_id, nodes).await
    }

    pub async fn channel_path_list_by_transaction(
        &self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<Vec<accounting::channel_path::ChannelPath>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel_path::channel_path_list_by_transaction(&mut conn, transaction_id).await
    }

    pub async fn channel_path_delete_by_transaction(
        &self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel_path::channel_path_delete_by_transaction(&mut conn, transaction_id)
            .await
    }

    pub async fn channel_path_find_transactions_by_channel(
        &self,
        channel_id: accounting::id::ChannelId,
    ) -> Result<Vec<accounting::id::TransactionId>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel_path::channel_path_find_transactions_by_channel(&mut conn, channel_id)
            .await
    }

    pub async fn channel_path_count_by_channel(
        &self,
        channel_id: accounting::id::ChannelId,
    ) -> Result<i64, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel_path::channel_path_count_by_channel(&mut conn, channel_id).await
    }

    pub async fn channel_path_update_reconciled(
        &self,
        id: accounting::id::ChannelPathId,
        reconciled: bool,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel_path::channel_path_update_reconciled(&mut conn, id, reconciled).await
    }

    pub async fn channel_path_get(
        &self,
        id: accounting::id::ChannelPathId,
    ) -> Result<Option<accounting::channel_path::ChannelPath>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::channel_path::channel_path_get(&mut conn, id).await
    }

    // === Tag ===

    pub async fn tag_get_by_name(
        &self,
        name: &str,
    ) -> Result<Option<accounting::tag::Tag>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::tag::tag_get_by_name(&mut conn, name).await
    }

    pub async fn tag_list(&self) -> Result<Vec<accounting::tag::Tag>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::tag::tag_list(&mut conn).await
    }

    pub async fn tag_create(
        &self,
        tag: &accounting::tag::Tag,
    ) -> Result<accounting::id::TagId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::tag::tag_create(&mut conn, tag).await
    }

    pub async fn tag_delete(&self, name: &str) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::tag::tag_delete(&mut conn, name).await
    }

    pub async fn tag_upsert_by_name(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<accounting::id::TagId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::tag::tag_upsert_by_name(&mut conn, name, description).await
    }

    pub async fn tag_names_by_transactions(
        &self,
        transaction_ids: &[accounting::id::TransactionId],
    ) -> Result<std::collections::HashMap<accounting::id::TransactionId, Vec<String>>, DbError>
    {
        let mut conn = self.acquire().await?;
        crate::repo::tag::tag_names_by_transactions(&mut conn, transaction_ids).await
    }

    // === Attachment ===

    pub async fn attachment_create(
        &self,
        attachment: &accounting::attachment::Attachment,
    ) -> Result<accounting::id::AttachmentId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::attachment::attachment_create(&mut conn, attachment).await
    }

    pub async fn attachment_list_by_transaction(
        &self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<Vec<accounting::attachment::Attachment>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::attachment::attachment_list_by_transaction(&mut conn, transaction_id).await
    }

    pub async fn attachment_delete(&self, id: accounting::id::AttachmentId) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::attachment::attachment_delete(&mut conn, id).await
    }

    // === Transaction ===

    pub async fn transaction_insert(
        &self,
        tx: &accounting::transaction::Transaction,
        tag_ids: &[accounting::id::TagId],
    ) -> Result<accounting::id::TransactionId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::transaction::transaction_insert(&mut conn, tx, tag_ids).await
    }

    pub async fn transaction_get(
        &self,
        id: accounting::id::TransactionId,
    ) -> Result<Option<accounting::transaction::Transaction>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::transaction::transaction_get(&mut conn, id).await
    }

    pub async fn transaction_list(
        &self,
        filter: &accounting::transaction_filter::TransactionFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<accounting::transaction::Transaction>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::transaction::transaction_list(&mut conn, filter, limit, offset).await
    }

    pub async fn transaction_count(
        &self,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<usize, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::transaction::transaction_count(&mut conn, filter).await
    }

    pub async fn transaction_delete(
        &self,
        id: accounting::id::TransactionId,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::transaction::transaction_delete(&mut conn, id).await
    }

    pub async fn transaction_update(
        &self,
        tx: &accounting::transaction::Transaction,
        tag_ids: &[accounting::id::TagId],
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::transaction::transaction_update(&mut conn, tx, tag_ids).await
    }

    // === Posting ===

    pub async fn posting_insert(
        &self,
        posting: &accounting::posting::Posting,
    ) -> Result<accounting::id::PostingId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_insert(&mut conn, posting).await
    }

    pub async fn posting_get(
        &self,
        id: accounting::id::PostingId,
    ) -> Result<Option<accounting::posting::Posting>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_get(&mut conn, id).await
    }

    pub async fn posting_list_by_transaction(
        &self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<Vec<accounting::posting::Posting>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_list_by_transaction(&mut conn, transaction_id).await
    }

    pub async fn posting_list_by_account(
        &self,
        account_id: accounting::id::AccountId,
    ) -> Result<Vec<accounting::posting::Posting>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_list_by_account(&mut conn, account_id).await
    }

    pub async fn posting_has_postings(
        &self,
        account_id: accounting::id::AccountId,
    ) -> Result<bool, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_has_postings(&mut conn, account_id).await
    }

    pub async fn posting_sum_by_account(
        &self,
        account_id: accounting::id::AccountId,
    ) -> Result<Vec<(accounting::id::CommodityId, rust_decimal::Decimal)>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_sum_by_account(&mut conn, account_id).await
    }

    pub async fn posting_delete_by_transaction(
        &self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_delete_by_transaction(&mut conn, transaction_id).await
    }

    pub async fn posting_sum_with_ancestors(
        &self,
        ancestor_id: accounting::id::AccountId,
    ) -> Result<Vec<(accounting::id::CommodityId, rust_decimal::Decimal)>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_sum_with_ancestors(&mut conn, ancestor_id).await
    }

    pub async fn posting_sum_by_tag(
        &self,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<
        Vec<(
            accounting::id::TagId,
            accounting::id::CommodityId,
            String,
            rust_decimal::Decimal,
        )>,
        DbError,
    > {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_sum_by_tag(&mut conn, filter).await
    }

    pub async fn posting_sum_by_member(
        &self,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<
        Vec<(
            accounting::id::MemberId,
            accounting::id::CommodityId,
            String,
            rust_decimal::Decimal,
        )>,
        DbError,
    > {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_sum_by_member(&mut conn, filter).await
    }

    pub async fn posting_sum_by_channel(
        &self,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<
        Vec<(
            accounting::id::ChannelId,
            accounting::id::CommodityId,
            String,
            rust_decimal::Decimal,
        )>,
        DbError,
    > {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_sum_by_channel(&mut conn, filter).await
    }

    pub async fn posting_summary(
        &self,
        start: Option<chrono::NaiveDate>,
        end: Option<chrono::NaiveDate>,
    ) -> Result<crate::repo::posting::PostingSummary, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::posting_summary(&mut conn, start, end).await
    }

    // === Settings ===

    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::get_setting(&mut conn, key).await
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::set_setting(&mut conn, key, value).await
    }

    // === Budget ===

    pub async fn budget_create(
        &self,
        name: &str,
        period: accounting::budget::BudgetPeriod,
        commodity_id: accounting::id::CommodityId,
        limits: &[(accounting::id::AccountId, rust_decimal::Decimal)],
    ) -> Result<accounting::id::BudgetId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::budget::budget_create(&mut conn, name, period, commodity_id, limits).await
    }

    pub async fn budget_get(
        &self,
        id: accounting::id::BudgetId,
    ) -> Result<Option<accounting::budget::Budget>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::budget::budget_get(&mut conn, id).await
    }

    pub async fn budget_list(&self) -> Result<Vec<accounting::budget::Budget>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::budget::budget_list(&mut conn).await
    }

    pub async fn budget_update(
        &self,
        budget_id: accounting::id::BudgetId,
        name: &str,
        period: accounting::budget::BudgetPeriod,
        commodity_id: accounting::id::CommodityId,
        limits: &[(accounting::id::AccountId, rust_decimal::Decimal)],
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::budget::budget_update(&mut conn, budget_id, name, period, commodity_id, limits)
            .await
    }

    pub async fn budget_delete(&self, id: accounting::id::BudgetId) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::budget::budget_delete(&mut conn, id).await
    }

    pub async fn budget_get_limits(
        &self,
        budget_id: accounting::id::BudgetId,
    ) -> Result<Vec<accounting::budget::BudgetLimit>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::budget::budget_get_limits(&mut conn, budget_id).await
    }

    pub async fn budget_list_all_with_limits(
        &self,
    ) -> Result<
        Vec<(
            accounting::budget::Budget,
            Vec<accounting::budget::BudgetLimit>,
        )>,
        DbError,
    > {
        let mut conn = self.acquire().await?;
        crate::repo::budget::budget_list_all_with_limits(&mut conn).await
    }

    pub async fn budget_upsert_by_name(
        &self,
        name: &str,
        period: accounting::budget::BudgetPeriod,
        commodity_id: accounting::id::CommodityId,
        limits: &[(accounting::id::AccountId, rust_decimal::Decimal)],
    ) -> Result<accounting::id::BudgetId, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::budget::budget_upsert_by_name(&mut conn, name, period, commodity_id, limits)
            .await
    }

    // === Account Mapping ===

    pub async fn account_mapping_upsert(
        &self,
        mapping: &accounting::account_mapping::AccountMapping,
    ) -> Result<(), DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account_mapping::mapping_upsert(&mut conn, mapping).await
    }

    pub async fn account_mapping_find(
        &self,
        member_id: accounting::id::MemberId,
        channel_id: accounting::id::ChannelId,
        category: &str,
    ) -> Result<Option<accounting::account_mapping::AccountMapping>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account_mapping::mapping_find(&mut conn, member_id, channel_id, category).await
    }

    pub async fn account_mapping_list(
        &self,
        member_id: accounting::id::MemberId,
        channel_id: accounting::id::ChannelId,
    ) -> Result<Vec<accounting::account_mapping::AccountMapping>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account_mapping::mapping_list(&mut conn, member_id, channel_id).await
    }

    pub async fn account_mapping_delete(
        &self,
        member_id: accounting::id::MemberId,
        channel_id: accounting::id::ChannelId,
        category: &str,
    ) -> Result<bool, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account_mapping::mapping_delete(&mut conn, member_id, channel_id, category)
            .await
    }

    pub async fn account_mapping_count_by_account(
        &self,
        account_id: accounting::id::AccountId,
    ) -> Result<i64, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account_mapping::mapping_count_by_account(&mut conn, account_id).await
    }

    pub async fn account_mapping_list_all(
        &self,
    ) -> Result<Vec<accounting::account_mapping::AccountMapping>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::account_mapping::mapping_list_all(&mut conn).await
    }

    // === Budget Statistics ===

    pub async fn sum_by_account_with_descendants(
        &self,
        account_ids: &[accounting::id::AccountId],
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        exclude_tag_ids: &[accounting::id::TagId],
        commodity_id: accounting::id::CommodityId,
    ) -> Result<Vec<(accounting::id::AccountId, rust_decimal::Decimal)>, DbError> {
        let mut conn = self.acquire().await?;
        crate::repo::posting::sum_by_account_with_descendants(
            &mut conn,
            account_ids,
            start_date,
            end_date,
            exclude_tag_ids,
            commodity_id,
        )
        .await
    }

    async fn acquire(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Sqlite>, DbError> {
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
        let _ = std::fs::remove_file(format!("{}-wal", path));
        let _ = std::fs::remove_file(format!("{}-shm", path));

        let db = SqliteDatabase::open(path).await.unwrap();
        drop(db);

        assert!(std::path::Path::new(path).exists());

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}-wal", path));
        let _ = std::fs::remove_file(format!("{}-shm", path));
    }

    #[tokio::test]
    async fn test_initialize_is_idempotent_across_languages() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();

        db.initialize(Some("zh-CN")).await.unwrap();
        // 模拟 API 服务在另一种语言下再次调用 initialize
        db.initialize(Some("en")).await.unwrap();

        let root_names: Vec<String> =
            sqlx::query_scalar("SELECT name FROM accounts WHERE parent_id IS NULL ORDER BY name")
                .fetch_all(&db.pool)
                .await
                .unwrap();

        assert_eq!(root_names, vec!["导入", "支出", "收入", "权益", "资产"]);

        let lang = db.get_setting("language").await.unwrap();
        assert_eq!(lang, Some("zh-CN".to_string()));
    }
}

use sqlx::{Sqlite, Transaction};

use crate::error::DbError;

/// SQLite 事务
pub struct SqliteTransaction<'a> {
    tx: Transaction<'a, Sqlite>,
}

impl<'a> SqliteTransaction<'a> {
    pub(crate) fn new(tx: Transaction<'a, Sqlite>) -> Self {
        Self { tx }
    }

    /// 提交事务
    pub async fn commit(self) -> Result<(), DbError> {
        self.tx
            .commit()
            .await
            .map_err(|e| DbError::Database(e.to_string()))
    }

    // === Account ===

    pub async fn account_create(
        &mut self,
        account: &accounting::account::Account,
    ) -> Result<accounting::id::AccountId, DbError> {
        crate::repo::account::account_create(&mut self.tx, account).await
    }

    pub async fn account_create_with_closure(
        &mut self,
        account: &accounting::account::Account,
    ) -> Result<accounting::id::AccountId, DbError> {
        crate::repo::account::account_create_with_closure(&mut self.tx, account).await
    }

    pub async fn account_get(
        &mut self,
        id: accounting::id::AccountId,
    ) -> Result<Option<accounting::account::Account>, DbError> {
        crate::repo::account::account_get(&mut self.tx, id).await
    }

    pub async fn account_get_by_name(
        &mut self,
        name: &str,
    ) -> Result<Option<accounting::account::Account>, DbError> {
        crate::repo::account::account_get_by_name(&mut self.tx, name).await
    }

    pub async fn account_get_by_parent_and_name(
        &mut self,
        parent_id: Option<accounting::id::AccountId>,
        name: &str,
    ) -> Result<Option<accounting::account::Account>, DbError> {
        crate::repo::account::account_get_by_parent_and_name(&mut self.tx, parent_id, name).await
    }

    pub async fn account_list(&mut self) -> Result<Vec<accounting::account::Account>, DbError> {
        crate::repo::account::account_list(&mut self.tx).await
    }

    pub async fn account_list_children(
        &mut self,
        parent_id: accounting::id::AccountId,
    ) -> Result<Vec<accounting::account::Account>, DbError> {
        crate::repo::account::account_list_children(&mut self.tx, parent_id).await
    }

    pub async fn account_rename(
        &mut self,
        id: accounting::id::AccountId,
        new_name: &str,
    ) -> Result<(), DbError> {
        crate::repo::account::account_rename(&mut self.tx, id, new_name).await
    }

    pub async fn account_close(&mut self, id: accounting::id::AccountId) -> Result<(), DbError> {
        crate::repo::account::account_close(&mut self.tx, id).await
    }

    pub async fn account_reopen(&mut self, id: accounting::id::AccountId) -> Result<(), DbError> {
        crate::repo::account::account_reopen(&mut self.tx, id).await
    }

    pub async fn account_delete(&mut self, id: accounting::id::AccountId) -> Result<(), DbError> {
        crate::repo::account::account_delete(&mut self.tx, id).await
    }

    pub async fn account_get_owners(
        &mut self,
        account_id: accounting::id::AccountId,
    ) -> Result<Vec<accounting::id::MemberId>, DbError> {
        crate::repo::account::account_get_owners(&mut self.tx, account_id).await
    }

    pub async fn account_set_owners(
        &mut self,
        account_id: accounting::id::AccountId,
        member_ids: &[accounting::id::MemberId],
    ) -> Result<(), DbError> {
        crate::repo::account::account_set_owners(&mut self.tx, account_id, member_ids).await
    }

    pub async fn account_find_root_name(
        &mut self,
        account_id: accounting::id::AccountId,
    ) -> Result<String, DbError> {
        crate::repo::account::account_find_root_name(&mut self.tx, account_id).await
    }

    pub async fn account_find_root_id(
        &mut self,
        account_id: accounting::id::AccountId,
    ) -> Result<accounting::id::AccountId, DbError> {
        crate::repo::account::account_find_root_id(&mut self.tx, account_id).await
    }

    pub async fn account_get_or_create_by_path(
        &mut self,
        path: &str,
    ) -> Result<accounting::id::AccountId, DbError> {
        crate::repo::account::account_get_or_create_by_path(&mut self.tx, path).await
    }

    pub async fn account_update_by_path(
        &mut self,
        path: &str,
        closed_at: Option<chrono::NaiveDate>,
        billing_day: Option<u8>,
        repayment_day: Option<u8>,
    ) -> Result<(), DbError> {
        crate::repo::account::account_update_by_path(
            &mut self.tx,
            path,
            closed_at,
            billing_day,
            repayment_day,
        )
        .await
    }

    pub async fn account_list_owners(
        &mut self,
    ) -> Result<Vec<(accounting::id::AccountId, accounting::id::MemberId)>, DbError> {
        crate::repo::account::account_list_owners(&mut self.tx).await
    }

    pub async fn account_rebuild_ancestors(&mut self) -> Result<(), DbError> {
        crate::repo::account::account_rebuild_ancestors(&mut self.tx).await
    }

    // === Commodity ===

    pub async fn commodity_get_by_symbol(
        &mut self,
        symbol: &str,
    ) -> Result<Option<accounting::commodity::Commodity>, DbError> {
        crate::repo::commodity::commodity_get_by_symbol(&mut self.tx, symbol).await
    }

    pub async fn commodity_list(
        &mut self,
    ) -> Result<Vec<accounting::commodity::Commodity>, DbError> {
        crate::repo::commodity::commodity_list(&mut self.tx).await
    }

    pub async fn commodity_create(
        &mut self,
        commodity: &accounting::commodity::Commodity,
    ) -> Result<accounting::id::CommodityId, DbError> {
        crate::repo::commodity::commodity_create(&mut self.tx, commodity).await
    }

    pub async fn commodity_upsert_by_symbol(
        &mut self,
        symbol: &str,
        name: &str,
        precision: u8,
    ) -> Result<accounting::id::CommodityId, DbError> {
        crate::repo::commodity::commodity_upsert_by_symbol(&mut self.tx, symbol, name, precision)
            .await
    }

    // === Member ===

    pub async fn member_create(
        &mut self,
        member: &accounting::member::Member,
    ) -> Result<accounting::id::MemberId, DbError> {
        crate::repo::member::member_create(&mut self.tx, member).await
    }

    pub async fn member_get(
        &mut self,
        id: accounting::id::MemberId,
    ) -> Result<Option<accounting::member::Member>, DbError> {
        crate::repo::member::member_get(&mut self.tx, id).await
    }

    pub async fn member_list(&mut self) -> Result<Vec<accounting::member::Member>, DbError> {
        crate::repo::member::member_list(&mut self.tx).await
    }

    pub async fn member_delete(&mut self, id: accounting::id::MemberId) -> Result<(), DbError> {
        crate::repo::member::member_delete(&mut self.tx, id).await
    }

    pub async fn member_get_or_create_by_name(
        &mut self,
        name: &str,
    ) -> Result<accounting::id::MemberId, DbError> {
        crate::repo::member::member_get_or_create_by_name(&mut self.tx, name).await
    }

    // === Channel ===

    pub async fn channel_create(
        &mut self,
        channel: &accounting::channel::Channel,
    ) -> Result<accounting::id::ChannelId, DbError> {
        crate::repo::channel::channel_create(&mut self.tx, channel).await
    }

    pub async fn channel_get(
        &mut self,
        id: accounting::id::ChannelId,
    ) -> Result<Option<accounting::channel::Channel>, DbError> {
        crate::repo::channel::channel_get(&mut self.tx, id).await
    }

    pub async fn channel_get_by_name(
        &mut self,
        name: &str,
    ) -> Result<Option<accounting::channel::Channel>, DbError> {
        crate::repo::channel::channel_get_by_name(&mut self.tx, name).await
    }

    pub async fn channel_resolve_by_name(
        &mut self,
        name: &str,
    ) -> Result<Option<accounting::channel::Channel>, DbError> {
        crate::repo::channel::channel_resolve_by_name(&mut self.tx, name).await
    }

    pub async fn channel_list(&mut self) -> Result<Vec<accounting::channel::Channel>, DbError> {
        crate::repo::channel::channel_list(&mut self.tx).await
    }

    pub async fn channel_count_transactions_by_id(
        &mut self,
        channel_id: accounting::id::ChannelId,
    ) -> Result<i64, DbError> {
        crate::repo::channel::channel_count_transactions_by_id(&mut self.tx, channel_id).await
    }

    pub async fn channel_force_delete_by_id(
        &mut self,
        channel_id: accounting::id::ChannelId,
    ) -> Result<(), DbError> {
        crate::repo::channel::channel_force_delete_by_id(&mut self.tx, channel_id).await
    }

    pub async fn channel_update(
        &mut self,
        id: accounting::id::ChannelId,
        account_id: Option<accounting::id::AccountId>,
    ) -> Result<(), DbError> {
        crate::repo::channel::channel_update(&mut self.tx, id, account_id).await
    }

    pub async fn channel_upsert_by_name(
        &mut self,
        name: &str,
        description: Option<&str>,
        account_id: Option<accounting::id::AccountId>,
    ) -> Result<accounting::id::ChannelId, DbError> {
        crate::repo::channel::channel_upsert_by_name(&mut self.tx, name, description, account_id)
            .await
    }

    // === ChannelPath ===

    pub async fn channel_path_create(
        &mut self,
        transaction_id: accounting::id::TransactionId,
        node: &accounting::channel_path::ChannelPathNode,
    ) -> Result<accounting::id::ChannelPathId, DbError> {
        crate::repo::channel_path::channel_path_create(&mut self.tx, transaction_id, node).await
    }

    pub async fn channel_path_create_batch(
        &mut self,
        transaction_id: accounting::id::TransactionId,
        nodes: &[accounting::channel_path::ChannelPathNode],
    ) -> Result<(), DbError> {
        crate::repo::channel_path::channel_path_create_batch(&mut self.tx, transaction_id, nodes)
            .await
    }

    pub async fn channel_path_list_by_transaction(
        &mut self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<Vec<accounting::channel_path::ChannelPath>, DbError> {
        crate::repo::channel_path::channel_path_list_by_transaction(&mut self.tx, transaction_id)
            .await
    }

    pub async fn channel_path_delete_by_transaction(
        &mut self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<(), DbError> {
        crate::repo::channel_path::channel_path_delete_by_transaction(&mut self.tx, transaction_id)
            .await
    }

    pub async fn channel_path_find_transactions_by_channel(
        &mut self,
        channel_id: accounting::id::ChannelId,
    ) -> Result<Vec<accounting::id::TransactionId>, DbError> {
        crate::repo::channel_path::channel_path_find_transactions_by_channel(
            &mut self.tx,
            channel_id,
        )
        .await
    }

    pub async fn channel_path_count_by_channel(
        &mut self,
        channel_id: accounting::id::ChannelId,
    ) -> Result<i64, DbError> {
        crate::repo::channel_path::channel_path_count_by_channel(&mut self.tx, channel_id).await
    }

    pub async fn channel_path_update_status(
        &mut self,
        id: accounting::id::ChannelPathId,
        status: accounting::channel_path::ChannelPathStatus,
    ) -> Result<(), DbError> {
        crate::repo::channel_path::channel_path_update_status(&mut self.tx, id, status).await
    }

    pub async fn channel_path_get(
        &mut self,
        id: accounting::id::ChannelPathId,
    ) -> Result<Option<accounting::channel_path::ChannelPath>, DbError> {
        crate::repo::channel_path::channel_path_get(&mut self.tx, id).await
    }

    // === Tag ===

    pub async fn tag_get_by_name(
        &mut self,
        name: &str,
    ) -> Result<Option<accounting::tag::Tag>, DbError> {
        crate::repo::tag::tag_get_by_name(&mut self.tx, name).await
    }

    pub async fn tag_list(&mut self) -> Result<Vec<accounting::tag::Tag>, DbError> {
        crate::repo::tag::tag_list(&mut self.tx).await
    }

    pub async fn tag_create(
        &mut self,
        tag: &accounting::tag::Tag,
    ) -> Result<accounting::id::TagId, DbError> {
        crate::repo::tag::tag_create(&mut self.tx, tag).await
    }

    pub async fn tag_delete(&mut self, name: &str) -> Result<(), DbError> {
        crate::repo::tag::tag_delete(&mut self.tx, name).await
    }

    pub async fn tag_upsert_by_name(
        &mut self,
        name: &str,
        description: Option<&str>,
    ) -> Result<accounting::id::TagId, DbError> {
        crate::repo::tag::tag_upsert_by_name(&mut self.tx, name, description).await
    }

    // === Attachment ===

    pub async fn attachment_create(
        &mut self,
        attachment: &accounting::attachment::Attachment,
    ) -> Result<accounting::id::AttachmentId, DbError> {
        crate::repo::attachment::attachment_create(&mut self.tx, attachment).await
    }

    pub async fn attachment_list_by_transaction(
        &mut self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<Vec<accounting::attachment::Attachment>, DbError> {
        crate::repo::attachment::attachment_list_by_transaction(&mut self.tx, transaction_id).await
    }

    pub async fn attachment_delete(
        &mut self,
        id: accounting::id::AttachmentId,
    ) -> Result<(), DbError> {
        crate::repo::attachment::attachment_delete(&mut self.tx, id).await
    }

    // === Transaction ===

    pub async fn transaction_insert(
        &mut self,
        tx: &accounting::transaction::Transaction,
        tag_ids: &[accounting::id::TagId],
    ) -> Result<accounting::id::TransactionId, DbError> {
        crate::repo::transaction::transaction_insert(&mut self.tx, tx, tag_ids).await
    }

    pub async fn transaction_get(
        &mut self,
        id: accounting::id::TransactionId,
    ) -> Result<Option<accounting::transaction::Transaction>, DbError> {
        crate::repo::transaction::transaction_get(&mut self.tx, id).await
    }

    pub async fn transaction_list(
        &mut self,
        filter: &accounting::transaction_filter::TransactionFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<accounting::transaction::Transaction>, DbError> {
        crate::repo::transaction::transaction_list(&mut self.tx, filter, limit, offset).await
    }

    pub async fn transaction_count(
        &mut self,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<usize, DbError> {
        crate::repo::transaction::transaction_count(&mut self.tx, filter).await
    }

    pub async fn transaction_delete(
        &mut self,
        id: accounting::id::TransactionId,
    ) -> Result<(), DbError> {
        crate::repo::transaction::transaction_delete(&mut self.tx, id).await
    }

    pub async fn transaction_update(
        &mut self,
        tx: &accounting::transaction::Transaction,
        tag_ids: &[accounting::id::TagId],
    ) -> Result<(), DbError> {
        crate::repo::transaction::transaction_update(&mut self.tx, tx, tag_ids).await
    }

    // === Posting ===

    pub async fn posting_insert(
        &mut self,
        posting: &accounting::posting::Posting,
    ) -> Result<accounting::id::PostingId, DbError> {
        crate::repo::posting::posting_insert(&mut self.tx, posting).await
    }

    pub async fn posting_get(
        &mut self,
        id: accounting::id::PostingId,
    ) -> Result<Option<accounting::posting::Posting>, DbError> {
        crate::repo::posting::posting_get(&mut self.tx, id).await
    }

    pub async fn posting_list_by_transaction(
        &mut self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<Vec<accounting::posting::Posting>, DbError> {
        crate::repo::posting::posting_list_by_transaction(&mut self.tx, transaction_id).await
    }

    pub async fn posting_list_by_account(
        &mut self,
        account_id: accounting::id::AccountId,
    ) -> Result<Vec<accounting::posting::Posting>, DbError> {
        crate::repo::posting::posting_list_by_account(&mut self.tx, account_id).await
    }

    pub async fn posting_has_postings(
        &mut self,
        account_id: accounting::id::AccountId,
    ) -> Result<bool, DbError> {
        crate::repo::posting::posting_has_postings(&mut self.tx, account_id).await
    }

    pub async fn posting_sum_by_account(
        &mut self,
        account_id: accounting::id::AccountId,
    ) -> Result<Vec<(accounting::id::CommodityId, rust_decimal::Decimal)>, DbError> {
        crate::repo::posting::posting_sum_by_account(&mut self.tx, account_id).await
    }

    pub async fn posting_delete_by_transaction(
        &mut self,
        transaction_id: accounting::id::TransactionId,
    ) -> Result<(), DbError> {
        crate::repo::posting::posting_delete_by_transaction(&mut self.tx, transaction_id).await
    }

    pub async fn posting_sum_with_ancestors(
        &mut self,
        ancestor_id: accounting::id::AccountId,
    ) -> Result<Vec<(accounting::id::CommodityId, rust_decimal::Decimal)>, DbError> {
        crate::repo::posting::posting_sum_with_ancestors(&mut self.tx, ancestor_id).await
    }

    pub async fn posting_sum_by_tag(
        &mut self,
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
        crate::repo::posting::posting_sum_by_tag(&mut self.tx, filter).await
    }

    pub async fn posting_sum_by_member(
        &mut self,
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
        crate::repo::posting::posting_sum_by_member(&mut self.tx, filter).await
    }

    pub async fn posting_sum_by_channel(
        &mut self,
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
        crate::repo::posting::posting_sum_by_channel(&mut self.tx, filter).await
    }

    // === Settings ===

    pub async fn get_setting(&mut self, key: &str) -> Result<Option<String>, DbError> {
        crate::repo::get_setting(&mut self.tx, key).await
    }

    pub async fn set_setting(&mut self, key: &str, value: &str) -> Result<(), DbError> {
        crate::repo::set_setting(&mut self.tx, key, value).await
    }

    // === Account Mapping ===

    pub async fn account_mapping_upsert(
        &mut self,
        mapping: &accounting::account_mapping::AccountMapping,
    ) -> Result<(), DbError> {
        crate::repo::account_mapping::mapping_upsert(&mut self.tx, mapping).await
    }

    pub async fn account_mapping_list_all(
        &mut self,
    ) -> Result<Vec<accounting::account_mapping::AccountMapping>, DbError> {
        crate::repo::account_mapping::mapping_list_all(&mut self.tx).await
    }

    // === Budget ===

    pub async fn budget_upsert_by_name(
        &mut self,
        name: &str,
        period: accounting::finance_period::FinancePeriod,
        commodity_id: accounting::id::CommodityId,
        limits: &[(accounting::id::AccountId, rust_decimal::Decimal)],
    ) -> Result<accounting::id::BudgetId, DbError> {
        crate::repo::budget::budget_upsert_by_name(&mut self.tx, name, period, commodity_id, limits)
            .await
    }

    pub async fn budget_list_all_with_limits(
        &mut self,
    ) -> Result<
        Vec<(
            accounting::budget::Budget,
            Vec<accounting::budget::BudgetLimit>,
        )>,
        DbError,
    > {
        crate::repo::budget::budget_list_all_with_limits(&mut self.tx).await
    }
}

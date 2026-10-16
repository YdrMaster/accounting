//! 门面清单：`SqliteDatabase` / `SqliteTransaction` 对 repo 函数的委托方法。
//!
//! 清单是门面契约的唯一来源——repo 新增函数在对应侧登记一行即可暴露；
//! 宏只做逐字转发，不含任何参数映射。同一调用内的条目形态必须齐整：
//! 普通条目（方法名 == repo 函数名）与 `as` 改名条目分两次调用。

use crate::database::SqliteDatabase;
use crate::error::DbError;
use crate::transaction::SqliteTransaction;

macro_rules! facade {
    // db 侧，repo 根模块（如 get_setting/set_setting）
    (db: repo_root {
        $( $(#[$meta:meta])* $name:ident ( $($pname:ident : $pty:ty),* $(,)? ) -> $ret:ty ; )*
    }) => {
        impl SqliteDatabase {
            $(
                $(#[$meta])*
                pub async fn $name(&self, $($pname: $pty),*) -> Result<$ret, DbError> {
                    let mut conn = self.acquire().await?;
                    crate::repo::$name(&mut conn, $($pname),*).await
                }
            )*
        }
    };
    // tx 侧，repo 根模块
    (tx: repo_root {
        $( $(#[$meta:meta])* $name:ident ( $($pname:ident : $pty:ty),* $(,)? ) -> $ret:ty ; )*
    }) => {
        impl SqliteTransaction<'_> {
            $(
                $(#[$meta])*
                pub async fn $name(&mut self, $($pname: $pty),*) -> Result<$ret, DbError> {
                    crate::repo::$name(&mut self.tx, $($pname),*).await
                }
            )*
        }
    };
    // db 侧，普通条目
    (db: $domain:ident {
        $( $(#[$meta:meta])* $name:ident ( $($pname:ident : $pty:ty),* $(,)? ) -> $ret:ty ; )*
    }) => {
        impl SqliteDatabase {
            $(
                $(#[$meta])*
                pub async fn $name(&self, $($pname: $pty),*) -> Result<$ret, DbError> {
                    let mut conn = self.acquire().await?;
                    crate::repo::$domain::$name(&mut conn, $($pname),*).await
                }
            )*
        }
    };
    // db 侧，改名条目
    (db: $domain:ident {
        $( $(#[$meta:meta])* $name:ident ( $($pname:ident : $pty:ty),* $(,)? ) -> $ret:ty as $repo:ident ; )*
    }) => {
        impl SqliteDatabase {
            $(
                $(#[$meta])*
                pub async fn $name(&self, $($pname: $pty),*) -> Result<$ret, DbError> {
                    let mut conn = self.acquire().await?;
                    crate::repo::$domain::$repo(&mut conn, $($pname),*).await
                }
            )*
        }
    };
    // tx 侧，普通条目
    (tx: $domain:ident {
        $( $(#[$meta:meta])* $name:ident ( $($pname:ident : $pty:ty),* $(,)? ) -> $ret:ty ; )*
    }) => {
        impl SqliteTransaction<'_> {
            $(
                $(#[$meta])*
                pub async fn $name(&mut self, $($pname: $pty),*) -> Result<$ret, DbError> {
                    crate::repo::$domain::$name(&mut self.tx, $($pname),*).await
                }
            )*
        }
    };
    // tx 侧，改名条目
    (tx: $domain:ident {
        $( $(#[$meta:meta])* $name:ident ( $($pname:ident : $pty:ty),* $(,)? ) -> $ret:ty as $repo:ident ; )*
    }) => {
        impl SqliteTransaction<'_> {
            $(
                $(#[$meta])*
                pub async fn $name(&mut self, $($pname: $pty),*) -> Result<$ret, DbError> {
                    crate::repo::$domain::$repo(&mut self.tx, $($pname),*).await
                }
            )*
        }
    };
}

// === Member ===

facade! { db: member {
    member_create(member: &accounting::member::Member) -> accounting::id::MemberId;
    member_get(id: accounting::id::MemberId) -> Option<accounting::member::Member>;
    member_list() -> Vec<accounting::member::Member>;
    member_delete(id: accounting::id::MemberId) -> ();
    member_get_or_create_by_name(name: &str, lang: &str) -> accounting::id::MemberId;
    member_get_by_name(name: &str) -> Option<accounting::member::Member>;
    member_rename(id: accounting::id::MemberId, new_name: &str, lang: &str) -> ();
}}

facade! { tx: member {
    member_create(member: &accounting::member::Member) -> accounting::id::MemberId;
    member_get(id: accounting::id::MemberId) -> Option<accounting::member::Member>;
    member_list() -> Vec<accounting::member::Member>;
    member_delete(id: accounting::id::MemberId) -> ();
    member_get_or_create_by_name(name: &str, lang: &str) -> accounting::id::MemberId;
}}

facade! { db: posting {
    /// 每个指定账户各自（含后代）截至 as_of 的余额，按账户分组返回（分配算法口径，不去重）
    account_balances_by_ids(account_ids: &[accounting::id::AccountId],commodity_id: accounting::id::CommodityId,as_of: chrono::NaiveDate) -> Vec<(accounting::id::AccountId,rust_decimal::Decimal)>;
    /// 账户集合（含后代）截至 as_of 的余额合计（攒钱计划判定口径）
    account_balance_by_ids(account_ids: &[accounting::id::AccountId],commodity_id: accounting::id::CommodityId,as_of: chrono::NaiveDate) -> rust_decimal::Decimal;
}}

facade! { db: channel_path {

    channel_path_get(id: accounting::id::ChannelPathId) -> Option<accounting::channel_path::ChannelPath>;

    channel_path_update_status(id: accounting::id::ChannelPathId,status: accounting::channel_path::ChannelPathStatus) -> ();

    channel_path_find_transactions_by_channel(channel_id: accounting::id::ChannelId) -> Vec<accounting::id::TransactionId>;

    channel_path_delete_by_transaction(transaction_id: accounting::id::TransactionId) -> ();

    channel_path_list_by_transaction(transaction_id: accounting::id::TransactionId) -> Vec<accounting::channel_path::ChannelPath>;

    channel_path_create_batch(transaction_id: accounting::id::TransactionId,nodes: &[accounting::channel_path::ChannelPathNode]) -> ();

    channel_path_create(transaction_id: accounting::id::TransactionId,node: &accounting::channel_path::ChannelPathNode) -> accounting::id::ChannelPathId;
}}

facade! { db: channel {
    /// 渠道改名：把渠道在 `lang` 语言下的显示名改为 `new_name`
    channel_rename(id: accounting::id::ChannelId,new_name: &str,lang: &str) -> ();

    channel_upsert_by_name(name: &str,description: Option<&str>,account_id: Option<accounting::id::AccountId>,lang: &str) -> accounting::id::ChannelId;

    channel_update(id: accounting::id::ChannelId,description: Option<&str>,account_id: Option<accounting::id::AccountId>) -> ();

    channel_force_delete_by_id(channel_id: accounting::id::ChannelId) -> ();

    channel_in_use(channel_id: accounting::id::ChannelId) -> bool;
    /// 获取渠道的全部名字（任意语言）
    channel_names_by_id(id: accounting::id::ChannelId) -> Vec<String>;

    channel_list() -> Vec<accounting::channel::Channel>;

    channel_resolve_by_name(name: &str) -> Option<accounting::channel::Channel>;

    channel_get_by_name(name: &str) -> Option<accounting::channel::Channel>;

    channel_get(id: accounting::id::ChannelId) -> Option<accounting::channel::Channel>;

    channel_create(channel: &accounting::channel::Channel) -> accounting::id::ChannelId;
}}

facade! { db: commodity {

    commodity_upsert_by_symbol(symbol: &str,name: &str,precision: u8,lang: &str) -> accounting::id::CommodityId;

    commodity_create(commodity: &accounting::commodity::Commodity) -> accounting::id::CommodityId;

    commodity_created_at_map() -> std::collections::HashMap<accounting::id::CommodityId,chrono::NaiveDate>;

    commodity_list() -> Vec<accounting::commodity::Commodity>;

    commodity_get_by_symbol(symbol: &str) -> Option<accounting::commodity::Commodity>;
}}

facade! { db: account {

    account_created_at_map() -> std::collections::HashMap<accounting::id::AccountId,chrono::NaiveDate>;

    account_rebuild_ancestors() -> ();

    account_list_owners() -> Vec<(accounting::id::AccountId,accounting::id::MemberId)>;

    account_update_by_path(path: &str,closed_at: Option<chrono::NaiveDate>,billing_day: Option<u8>,repayment_day: Option<u8>) -> ();

    account_get_or_create_by_path(path: &str,lang: &str) -> accounting::id::AccountId;

    account_find_root_id(account_id: accounting::id::AccountId) -> accounting::id::AccountId;
    /// 批量解析每个输入账户的根账户在指定语言的系统显示名（单条 SQL）。
    account_root_names_by_ids(account_ids: &[accounting::id::AccountId],lang: &str) -> Vec<(accounting::id::AccountId,String)>;

    account_find_root_name(account_id: accounting::id::AccountId,lang: &str) -> String;

    account_set_owners(account_id: accounting::id::AccountId,member_ids: &[accounting::id::MemberId]) -> ();

    account_get_owners(account_id: accounting::id::AccountId) -> Vec<accounting::id::MemberId>;

    account_display_name(id: accounting::id::AccountId,lang: &str) -> Option<String>;

    account_is_descendant_of(account_id: accounting::id::AccountId,ancestor_id: accounting::id::AccountId) -> bool;

    account_update_parent(id: accounting::id::AccountId,new_parent_id: accounting::id::AccountId) -> ();

    account_delete(id: accounting::id::AccountId) -> ();

    account_reopen(id: accounting::id::AccountId) -> ();

    account_close(id: accounting::id::AccountId) -> ();

    account_update_fields(id: accounting::id::AccountId,billing_day: Option<u8>,repayment_day: Option<u8>) -> ();


    account_list_children(parent_id: accounting::id::AccountId) -> Vec<accounting::account::Account>;

    account_list() -> Vec<accounting::account::Account>;

    account_get_by_parent_and_name(parent_id: Option<accounting::id::AccountId>,name: &str) -> Option<accounting::account::Account>;

    account_get_by_name(name: &str) -> Option<accounting::account::Account>;

    account_get(id: accounting::id::AccountId) -> Option<accounting::account::Account>;

    account_create_with_name(account: &accounting::account::Account,name: &str,lang: &str) -> accounting::id::AccountId;

    account_create_with_closure(account: &accounting::account::Account) -> accounting::id::AccountId;

    account_create(account: &accounting::account::Account) -> accounting::id::AccountId;
}}

facade! { tx: posting {
    /// 每个指定账户各自（含后代）截至 as_of 的余额，按账户分组返回（分配算法口径，不去重）
    account_balances_by_ids(account_ids: &[accounting::id::AccountId],commodity_id: accounting::id::CommodityId,as_of: chrono::NaiveDate) -> Vec<(accounting::id::AccountId,rust_decimal::Decimal)>;
}}

facade! { tx: channel_path {

    channel_path_get(id: accounting::id::ChannelPathId) -> Option<accounting::channel_path::ChannelPath>;

    channel_path_update_status(id: accounting::id::ChannelPathId,status: accounting::channel_path::ChannelPathStatus) -> ();

    channel_path_find_transactions_by_channel(channel_id: accounting::id::ChannelId) -> Vec<accounting::id::TransactionId>;

    channel_path_delete_by_transaction(transaction_id: accounting::id::TransactionId) -> ();

    channel_path_list_by_transaction(transaction_id: accounting::id::TransactionId) -> Vec<accounting::channel_path::ChannelPath>;

    channel_path_create_batch(transaction_id: accounting::id::TransactionId,nodes: &[accounting::channel_path::ChannelPathNode]) -> ();

    channel_path_create(transaction_id: accounting::id::TransactionId,node: &accounting::channel_path::ChannelPathNode) -> accounting::id::ChannelPathId;
}}

facade! { tx: channel {

    channel_upsert_by_name(name: &str,description: Option<&str>,account_id: Option<accounting::id::AccountId>,lang: &str) -> accounting::id::ChannelId;

    channel_update(id: accounting::id::ChannelId,description: Option<&str>,account_id: Option<accounting::id::AccountId>) -> ();

    channel_force_delete_by_id(channel_id: accounting::id::ChannelId) -> ();

    channel_in_use(channel_id: accounting::id::ChannelId) -> bool;

    channel_list() -> Vec<accounting::channel::Channel>;

    channel_resolve_by_name(name: &str) -> Option<accounting::channel::Channel>;

    channel_get_by_name(name: &str) -> Option<accounting::channel::Channel>;

    channel_get(id: accounting::id::ChannelId) -> Option<accounting::channel::Channel>;

    channel_create(channel: &accounting::channel::Channel) -> accounting::id::ChannelId;
}}

facade! { tx: commodity {

    commodity_upsert_by_symbol(symbol: &str,name: &str,precision: u8,lang: &str) -> accounting::id::CommodityId;

    commodity_create(commodity: &accounting::commodity::Commodity) -> accounting::id::CommodityId;

    commodity_list() -> Vec<accounting::commodity::Commodity>;

    commodity_get_by_symbol(symbol: &str) -> Option<accounting::commodity::Commodity>;
}}

facade! { tx: account {

    account_display_name(id: accounting::id::AccountId,lang: &str) -> Option<String>;

    account_update_parent(id: accounting::id::AccountId,new_parent_id: accounting::id::AccountId) -> ();

    account_is_descendant_of(account_id: accounting::id::AccountId,ancestor_id: accounting::id::AccountId) -> bool;

    account_rebuild_ancestors() -> ();

    account_list_owners() -> Vec<(accounting::id::AccountId,accounting::id::MemberId)>;

    account_update_by_path(path: &str,closed_at: Option<chrono::NaiveDate>,billing_day: Option<u8>,repayment_day: Option<u8>) -> ();

    account_get_or_create_by_path(path: &str,lang: &str) -> accounting::id::AccountId;

    account_find_root_id(account_id: accounting::id::AccountId) -> accounting::id::AccountId;
    /// 批量解析每个输入账户的根账户在指定语言的系统显示名（单条 SQL）。
    account_root_names_by_ids(account_ids: &[accounting::id::AccountId],lang: &str) -> Vec<(accounting::id::AccountId,String)>;

    account_find_root_name(account_id: accounting::id::AccountId,lang: &str) -> String;

    account_set_owners(account_id: accounting::id::AccountId,member_ids: &[accounting::id::MemberId]) -> ();

    account_get_owners(account_id: accounting::id::AccountId) -> Vec<accounting::id::MemberId>;

    account_delete(id: accounting::id::AccountId) -> ();

    account_reopen(id: accounting::id::AccountId) -> ();

    account_close(id: accounting::id::AccountId) -> ();


    account_list_children(parent_id: accounting::id::AccountId) -> Vec<accounting::account::Account>;

    account_list() -> Vec<accounting::account::Account>;

    account_get_by_parent_and_name(parent_id: Option<accounting::id::AccountId>,name: &str) -> Option<accounting::account::Account>;

    account_get_by_name(name: &str) -> Option<accounting::account::Account>;

    account_get(id: accounting::id::AccountId) -> Option<accounting::account::Account>;

    account_create_with_name(account: &accounting::account::Account,name: &str,lang: &str) -> accounting::id::AccountId;

    account_create_with_closure(account: &accounting::account::Account) -> accounting::id::AccountId;

    account_create(account: &accounting::account::Account) -> accounting::id::AccountId;
}}

facade! { db: posting {
    /// 不限下界的分录合计（一次性预算实际值口径，上界为 end_date 当天结束）
    posting_sum_before(account_ids: &[accounting::id::AccountId],end_date: chrono::NaiveDate,exclude_tag_ids: &[accounting::id::TagId],commodity_id: accounting::id::CommodityId) -> Vec<(accounting::id::AccountId,rust_decimal::Decimal)>;

    posting_sum_by_period(account_ids: &[accounting::id::AccountId],start_date: chrono::NaiveDate,end_date: chrono::NaiveDate,exclude_tag_ids: &[accounting::id::TagId],commodity_id: accounting::id::CommodityId) -> Vec<(accounting::id::AccountId,rust_decimal::Decimal)>;

    posting_daily_summary(start: chrono::NaiveDate,end: chrono::NaiveDate) -> Vec<crate::repo::posting::DailySummaryRow>;

    posting_summary(start: Option<chrono::NaiveDate>,end: Option<chrono::NaiveDate>) -> crate::repo::posting::PostingSummary;

    posting_sum_with_ancestors(ancestor_id: accounting::id::AccountId) -> Vec<(accounting::id::CommodityId,rust_decimal::Decimal)>;

    posting_delete_by_transaction(transaction_id: accounting::id::TransactionId) -> ();

    posting_sum_by_account(account_id: accounting::id::AccountId) -> Vec<(accounting::id::CommodityId,rust_decimal::Decimal)>;

    posting_has_postings(account_id: accounting::id::AccountId) -> bool;

    posting_list_by_account(account_id: accounting::id::AccountId) -> Vec<accounting::posting::Posting>;

    posting_list_by_transaction(transaction_id: accounting::id::TransactionId) -> Vec<accounting::posting::Posting>;

    posting_get(id: accounting::id::PostingId) -> Option<accounting::posting::Posting>;

    posting_insert(posting: &accounting::posting::Posting) -> accounting::id::PostingId;
}}

facade! { db: transaction {

    transaction_update(tx: &accounting::transaction::Transaction,tag_ids: &[accounting::id::TagId]) -> ();

    transaction_delete(id: accounting::id::TransactionId) -> ();

    transaction_count(filter: &accounting::transaction_filter::TransactionFilter) -> usize;

    transaction_list(filter: &accounting::transaction_filter::TransactionFilter,limit: usize,offset: usize) -> Vec<accounting::transaction::Transaction>;

    transaction_get(id: accounting::id::TransactionId) -> Option<accounting::transaction::Transaction>;

    transaction_insert(tx: &accounting::transaction::Transaction,tag_ids: &[accounting::id::TagId]) -> accounting::id::TransactionId;
}}

facade! { db: attachment {

    attachment_delete(id: accounting::id::AttachmentId) -> ();

    attachment_list_by_transaction(transaction_id: accounting::id::TransactionId) -> Vec<accounting::attachment::Attachment>;

    attachment_create(attachment: &accounting::attachment::Attachment) -> accounting::id::AttachmentId;
}}

facade! { db: tag {

    tag_names_by_transactions(transaction_ids: &[accounting::id::TransactionId],lang: &str) -> std::collections::HashMap<accounting::id::TransactionId,Vec<String>>;

    tag_upsert_by_name(name: &str,description: Option<&str>,lang: &str) -> accounting::id::TagId;

    tag_delete(name: &str) -> ();

    tag_update(id: accounting::id::TagId,name: &str,description: Option<&str>,lang: &str) -> ();

    tag_get_by_id(id: accounting::id::TagId) -> Option<accounting::tag::Tag>;

    tag_create(tag: &accounting::tag::Tag) -> accounting::id::TagId;

    tag_list() -> Vec<accounting::tag::Tag>;

    tag_get_by_name(name: &str) -> Option<accounting::tag::Tag>;
}}

facade! { tx: posting {

    posting_sum_with_ancestors(ancestor_id: accounting::id::AccountId) -> Vec<(accounting::id::CommodityId,rust_decimal::Decimal)>;

    posting_delete_by_transaction(transaction_id: accounting::id::TransactionId) -> ();

    posting_sum_by_account(account_id: accounting::id::AccountId) -> Vec<(accounting::id::CommodityId,rust_decimal::Decimal)>;

    posting_has_postings(account_id: accounting::id::AccountId) -> bool;

    posting_list_by_account(account_id: accounting::id::AccountId) -> Vec<accounting::posting::Posting>;

    posting_list_by_transaction(transaction_id: accounting::id::TransactionId) -> Vec<accounting::posting::Posting>;

    posting_get(id: accounting::id::PostingId) -> Option<accounting::posting::Posting>;

    posting_insert(posting: &accounting::posting::Posting) -> accounting::id::PostingId;
}}

facade! { tx: transaction {

    transaction_update(tx: &accounting::transaction::Transaction,tag_ids: &[accounting::id::TagId]) -> ();

    transaction_delete(id: accounting::id::TransactionId) -> ();

    transaction_count(filter: &accounting::transaction_filter::TransactionFilter) -> usize;

    transaction_list(filter: &accounting::transaction_filter::TransactionFilter,limit: usize,offset: usize) -> Vec<accounting::transaction::Transaction>;

    transaction_get(id: accounting::id::TransactionId) -> Option<accounting::transaction::Transaction>;

    transaction_insert(tx: &accounting::transaction::Transaction,tag_ids: &[accounting::id::TagId]) -> accounting::id::TransactionId;
}}

facade! { tx: attachment {

    attachment_delete(id: accounting::id::AttachmentId) -> ();

    attachment_list_by_transaction(transaction_id: accounting::id::TransactionId) -> Vec<accounting::attachment::Attachment>;

    attachment_create(attachment: &accounting::attachment::Attachment) -> accounting::id::AttachmentId;
}}

facade! { tx: tag {

    tag_upsert_by_name(name: &str,description: Option<&str>,lang: &str) -> accounting::id::TagId;

    tag_delete(name: &str) -> ();

    tag_create(tag: &accounting::tag::Tag) -> accounting::id::TagId;

    tag_list() -> Vec<accounting::tag::Tag>;

    tag_get_by_name(name: &str) -> Option<accounting::tag::Tag>;
}}

facade! { db: posting {

    posting_daily_delta_by_account(commodity_id: accounting::id::CommodityId) -> Vec<(accounting::id::AccountId,chrono::NaiveDate,rust_decimal::Decimal,)>;

    posting_sum_all_assets() -> Vec<(accounting::id::AccountId,accounting::id::CommodityId,rust_decimal::Decimal,)>;

    posting_sum_by_channel(filter: &accounting::transaction_filter::TransactionFilter,lang: &str) -> Vec<(accounting::id::ChannelId,accounting::id::CommodityId,String,rust_decimal::Decimal,)>;

    posting_sum_by_member(filter: &accounting::transaction_filter::TransactionFilter,lang: &str) -> Vec<(accounting::id::MemberId,accounting::id::CommodityId,String,rust_decimal::Decimal,)>;

    posting_sum_by_tag(filter: &accounting::transaction_filter::TransactionFilter,lang: &str) -> Vec<(accounting::id::TagId,accounting::id::CommodityId,String,rust_decimal::Decimal,)>;
}}

facade! { db: tag {
    /// 批量取交易-标签 ID 关联，用于系统标签（如 pending）按 ID 判定。
    tag_ids_by_transactions(transaction_ids: &[accounting::id::TransactionId]) -> std::collections::HashMap<accounting::id::TransactionId,Vec<accounting::id::TagId>>;
}}

facade! { tx: posting {

    posting_sum_by_channel(filter: &accounting::transaction_filter::TransactionFilter,lang: &str) -> Vec<(accounting::id::ChannelId,accounting::id::CommodityId,String,rust_decimal::Decimal,)>;

    posting_sum_by_member(filter: &accounting::transaction_filter::TransactionFilter,lang: &str) -> Vec<(accounting::id::MemberId,accounting::id::CommodityId,String,rust_decimal::Decimal,)>;

    posting_sum_by_tag(filter: &accounting::transaction_filter::TransactionFilter,lang: &str) -> Vec<(accounting::id::TagId,accounting::id::CommodityId,String,rust_decimal::Decimal,)>;
}}

facade! { db: account_mapping {

    account_mapping_list_all() -> Vec<accounting::account_mapping::AccountMapping> as mapping_list_all;

    account_mapping_count_by_account(account_id: accounting::id::AccountId) -> i64 as mapping_count_by_account;

    account_mapping_delete(member_id: accounting::id::MemberId,channel_id: accounting::id::ChannelId,category: &str) -> bool as mapping_delete;

    account_mapping_list(member_id: accounting::id::MemberId,channel_id: accounting::id::ChannelId) -> Vec<accounting::account_mapping::AccountMapping> as mapping_list;

    account_mapping_find(member_id: accounting::id::MemberId,channel_id: accounting::id::ChannelId,category: &str) -> Option<accounting::account_mapping::AccountMapping> as mapping_find;

    account_mapping_upsert(mapping: &accounting::account_mapping::AccountMapping) -> () as mapping_upsert;
}}

facade! { db: saving_plan {

    saving_plan_list_all_with_accounts() -> Vec<(accounting::saving_plan::SavingPlan,Vec<accounting::saving_plan::SavingPlanAccount>,)>;

    saving_plan_get_accounts(plan_id: accounting::id::SavingPlanId) -> Vec<accounting::saving_plan::SavingPlanAccount>;

    saving_plan_delete(id: accounting::id::SavingPlanId) -> ();

    #[allow(clippy::too_many_arguments)]
    saving_plan_upsert_by_name(name: &str,period: Option<accounting::finance_period::FinancePeriod>,deadline: Option<chrono::NaiveDate>,commodity_id: accounting::id::CommodityId,target_amount: rust_decimal::Decimal,account_ids: &[accounting::id::AccountId],lang: &str) -> accounting::id::SavingPlanId;

    #[allow(clippy::too_many_arguments)]
    saving_plan_update(plan_id: accounting::id::SavingPlanId,name: &str,period: Option<accounting::finance_period::FinancePeriod>,deadline: Option<chrono::NaiveDate>,commodity_id: accounting::id::CommodityId,target_amount: rust_decimal::Decimal,account_ids: &[accounting::id::AccountId],lang: &str) -> ();

    saving_plan_get_by_name(name: &str) -> Option<accounting::saving_plan::SavingPlan>;

    saving_plan_list() -> Vec<accounting::saving_plan::SavingPlan>;

    saving_plan_get(id: accounting::id::SavingPlanId) -> Option<accounting::saving_plan::SavingPlan>;

    #[allow(clippy::too_many_arguments)]
    saving_plan_create(name: &str,period: Option<accounting::finance_period::FinancePeriod>,deadline: Option<chrono::NaiveDate>,commodity_id: accounting::id::CommodityId,target_amount: rust_decimal::Decimal,account_ids: &[accounting::id::AccountId],lang: &str) -> accounting::id::SavingPlanId;
}}

facade! { db: budget {

    budget_get_by_name(name: &str) -> Option<accounting::budget::Budget>;

    budget_upsert_by_name(name: &str,period: Option<accounting::finance_period::FinancePeriod>,deadline: Option<chrono::NaiveDate>,commodity_id: accounting::id::CommodityId,limits: &[(accounting::id::AccountId, rust_decimal::Decimal)],lang: &str) -> accounting::id::BudgetId;

    budget_list_all_with_limits() -> Vec<(accounting::budget::Budget,Vec<accounting::budget::BudgetLimit>,)>;

    budget_get_limits(budget_id: accounting::id::BudgetId) -> Vec<accounting::budget::BudgetLimit>;

    budget_delete(id: accounting::id::BudgetId) -> ();

    #[allow(clippy::too_many_arguments)]
    budget_update(budget_id: accounting::id::BudgetId,name: &str,period: Option<accounting::finance_period::FinancePeriod>,deadline: Option<chrono::NaiveDate>,commodity_id: accounting::id::CommodityId,limits: &[(accounting::id::AccountId, rust_decimal::Decimal)],lang: &str) -> ();

    budget_list() -> Vec<accounting::budget::Budget>;

    budget_get(id: accounting::id::BudgetId) -> Option<accounting::budget::Budget>;

    budget_create(name: &str,period: Option<accounting::finance_period::FinancePeriod>,deadline: Option<chrono::NaiveDate>,commodity_id: accounting::id::CommodityId,limits: &[(accounting::id::AccountId, rust_decimal::Decimal)],lang: &str) -> accounting::id::BudgetId;
}}

facade! { db: repo_root {

    set_setting(key: &str,value: &str) -> ();

    get_setting(key: &str) -> Option<String>;
}}

facade! { tx: budget {

    budget_list_all_with_limits() -> Vec<(accounting::budget::Budget,Vec<accounting::budget::BudgetLimit>,)>;

    budget_upsert_by_name(name: &str,period: Option<accounting::finance_period::FinancePeriod>,deadline: Option<chrono::NaiveDate>,commodity_id: accounting::id::CommodityId,limits: &[(accounting::id::AccountId, rust_decimal::Decimal)],lang: &str) -> accounting::id::BudgetId;
}}

facade! { tx: account_mapping {

    account_mapping_list_all() -> Vec<accounting::account_mapping::AccountMapping> as mapping_list_all;

    account_mapping_upsert(mapping: &accounting::account_mapping::AccountMapping) -> () as mapping_upsert;
}}

facade! { tx: repo_root {

    set_setting(key: &str,value: &str) -> ();

    get_setting(key: &str) -> Option<String>;
}}

facade! { db: posting {
    sum_by_account_with_descendants(account_ids: &[accounting::id::AccountId],start_date: chrono::NaiveDate,end_date: chrono::NaiveDate,exclude_tag_ids: &[accounting::id::TagId],commodity_id: accounting::id::CommodityId) -> Vec<(accounting::id::AccountId,rust_decimal::Decimal)>;
}}

//! 交易 API handler

use crate::dto::{ChannelPathNodeDto, CreateTransactionRequest, PostingDto, TransactionDto};
use crate::handlers::{Lang, member::AppState};
use accounting::channel_path::ChannelPathNode;
use accounting::datetime_utils;
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, MemberId, PostingId, TagId, TransactionId};
use accounting::posting::Posting;
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use accounting_service::transaction_service::TransactionService;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, put},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_i18n::t;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

/// 交易列表查询参数
///
/// serde_urlencoded 0.7 无法对 struct 字段反序列化重复键（`?account=1&account=2`
/// 会被当作重复字段报错），因此 handler 以 `Vec<(String, String)>` 取出全部键值对，
/// 再通过 [`TxQuery::from_pairs`] 手动构建，天然兼容单值与多值。
pub struct TxQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub account: Vec<i64>,
    pub member: Vec<i64>,
    pub tag: Vec<String>,
    pub channel: Vec<i64>,
    pub keyword: Option<String>,
    pub reimbursable: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl TxQuery {
    fn from_pairs(pairs: Vec<(String, String)>, lang: &str) -> Result<Self, String> {
        let mut q = Self {
            from: None,
            to: None,
            account: Vec::new(),
            member: Vec::new(),
            tag: Vec::new(),
            channel: Vec::new(),
            keyword: None,
            reimbursable: None,
            limit: None,
            offset: None,
        };
        for (key, value) in pairs {
            match key.as_str() {
                "from" => q.from = Some(value),
                "to" => q.to = Some(value),
                "account" => q.account.push(parse_id(&key, &value, lang)?),
                "member" => q.member.push(parse_id(&key, &value, lang)?),
                "channel" => q.channel.push(parse_id(&key, &value, lang)?),
                "tag" => q.tag.push(value),
                "keyword" => q.keyword = Some(value),
                "reimbursable" => {
                    q.reimbursable = Some(value.parse::<bool>().map_err(|e| {
                        t!(
                            "tx_err_invalid_reimbursable",
                            locale = lang,
                            value = value,
                            error = e
                        )
                        .to_string()
                    })?)
                }
                "limit" => q.limit = Some(parse_id(&key, &value, lang)?),
                "offset" => q.offset = Some(parse_id(&key, &value, lang)?),
                _ => {}
            }
        }
        Ok(q)
    }
}

fn parse_id(key: &str, value: &str, lang: &str) -> Result<i64, String> {
    value.parse::<i64>().map_err(|e| {
        t!(
            "tx_err_invalid_query_value",
            locale = lang,
            key = key,
            value = value,
            error = e
        )
        .to_string()
    })
}

/// 解析日期时间字符串
fn parse_date_time(s: &str, lang: &str) -> Result<chrono::NaiveDateTime, AccountingError> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(datetime_utils::start_of_day)
        .map_err(|_| {
            AccountingError::InvalidDate(
                t!("invalid_date_format", locale = lang, value = s).to_string(),
            )
        })
}

/// 交易展示所需的批量解析结果
struct DisplayContext {
    account_paths: HashMap<i64, String>,
    account_types: HashMap<i64, String>,
    commodities: HashMap<i64, String>,
    members: HashMap<i64, String>,
    channel_names: HashMap<ChannelId, String>,
}

/// 批量解析账户路径/类型、币种符号、成员名、渠道名（禁止逐条 N+1）
async fn load_display_context(
    db: &accounting_sql::SqliteDatabase,
    lang: &str,
) -> Result<DisplayContext, String> {
    let accounts: HashMap<AccountId, accounting::account::Account> = db
        .account_list()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| (a.id, a))
        .collect();
    let account_ids: Vec<AccountId> = accounts.keys().copied().collect();
    let account_names = db
        .account_display_names(&account_ids, lang)
        .await
        .map_err(|e| e.to_string())?;
    let account_paths: HashMap<i64, String> = accounts
        .values()
        .map(|a| (a.id.0, a.display_path(&accounts, &account_names)))
        .collect();
    let account_types = build_account_type_map(&accounts, &account_names);

    let commodity_list = db.commodity_list().await.map_err(|e| e.to_string())?;
    let commodities: HashMap<i64, String> = commodity_list
        .into_iter()
        .map(|c| (c.id.0, c.symbol))
        .collect();

    let member_list = db.member_list().await.map_err(|e| e.to_string())?;
    let member_ids: Vec<MemberId> = member_list.iter().map(|m| m.id).collect();
    let member_names = db
        .member_display_names(&member_ids, lang)
        .await
        .map_err(|e| e.to_string())?;
    let members: HashMap<i64, String> = member_names
        .into_iter()
        .map(|(id, name)| (id.0, name))
        .collect();

    let channel_list = db.channel_list().await.map_err(|e| e.to_string())?;
    let channel_ids: Vec<ChannelId> = channel_list.iter().map(|c| c.id).collect();
    let channel_names = db
        .channel_display_names(&channel_ids, lang)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DisplayContext {
        account_paths,
        account_types,
        commodities,
        members,
        channel_names,
    })
}

/// 列出交易（含筛选）
async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Query(pairs): Query<Vec<(String, String)>>,
) -> Result<Json<Vec<TransactionDto>>, String> {
    let query = TxQuery::from_pairs(pairs, &lang)?;
    let db = state.db();
    let mut filter = TransactionFilter::default();

    if let Some(from) = query.from {
        let date = NaiveDate::parse_from_str(&from, "%Y-%m-%d").map_err(|e| {
            t!(
                "tx_err_invalid_from_date",
                locale = lang.as_str(),
                error = e
            )
            .to_string()
        })?;
        filter.start_date = Some(date);
    }

    if let Some(to) = query.to {
        let date = NaiveDate::parse_from_str(&to, "%Y-%m-%d").map_err(|e| {
            t!("tx_err_invalid_to_date", locale = lang.as_str(), error = e).to_string()
        })?;
        filter.end_date = Some(date);
    }

    filter.account_ids = query.account.into_iter().map(AccountId).collect();

    filter.member_ids = query.member.into_iter().map(MemberId).collect();

    filter.channel_ids = query.channel.into_iter().map(ChannelId).collect();

    for tag_name in &query.tag {
        let tag = db
            .tag_get_by_name(tag_name)
            .await
            .map_err(|e| e.to_string())?;
        if let Some(tag) = tag {
            filter.tag_ids.push(tag.id);
        } else {
            return Err(t!(
                "tx_err_tag_not_found",
                locale = lang.as_str(),
                name = tag_name
            )
            .to_string());
        }
    }

    if let Some(keyword) = query.keyword {
        filter.keyword = Some(keyword);
    }

    if let Some(reimbursable) = query.reimbursable {
        filter.has_reimbursable = Some(reimbursable);
    }

    let service = TransactionService::new(db.clone());
    let transactions = service
        .list(filter, query.limit, query.offset)
        .await
        .map_err(|e| e.to_string())?;

    let ctx = load_display_context(db, &lang).await?;
    let tx_ids: Vec<TransactionId> = transactions.iter().map(|(tx, _, _)| tx.id).collect();
    let tag_map = db
        .tag_names_by_transactions(&tx_ids, &lang)
        .await
        .map_err(|e| e.to_string())?;
    let tag_ids_map = db
        .tag_ids_by_transactions(&tx_ids)
        .await
        .map_err(|e| e.to_string())?;
    let pending_id = pending_tag_id(db).await?;

    let dtos: Vec<TransactionDto> = transactions
        .into_iter()
        .map(|(tx, postings, channel_paths)| TransactionDto {
            id: tx.id.0,
            date_time: tx.date_time.to_string(),
            description: tx.description,
            kind: match tx.kind {
                accounting::transaction::TransactionKind::Refund => "refund".to_string(),
                accounting::transaction::TransactionKind::Reimbursement => {
                    "reimbursement".to_string()
                }
                _ => "normal".to_string(),
            },
            member_id: tx.member_id.0,
            member_name: ctx
                .members
                .get(&tx.member_id.0)
                .cloned()
                .unwrap_or_default(),
            tags: tag_map.get(&tx.id).cloned().unwrap_or_default(),
            pending: tx_has_pending(tag_ids_map.get(&tx.id), pending_id),
            channel_paths: channel_paths
                .into_iter()
                .map(|n| ChannelPathNodeDto {
                    position: n.position,
                    channel_id: n.channel_id.0,
                    channel_name: ctx
                        .channel_names
                        .get(&n.channel_id)
                        .cloned()
                        .unwrap_or_else(|| n.channel_id.0.to_string()),
                    status: n.status.as_str().to_string(),
                })
                .collect(),
            postings: postings
                .into_iter()
                .map(|p| posting_to_dto(p, &ctx))
                .collect(),
        })
        .collect();

    Ok(Json(dtos))
}

/// 分录转 DTO
fn posting_to_dto(p: Posting, ctx: &DisplayContext) -> PostingDto {
    PostingDto {
        id: p.id.0,
        transaction_id: p.transaction_id.0,
        account_id: p.account_id.0,
        account: ctx
            .account_paths
            .get(&p.account_id.0)
            .cloned()
            .unwrap_or_default(),
        account_type: ctx
            .account_types
            .get(&p.account_id.0)
            .cloned()
            .unwrap_or_default(),
        commodity: ctx
            .commodities
            .get(&p.commodity_id.0)
            .cloned()
            .unwrap_or_default(),
        amount: p.amount.to_string(),
        is_reimbursable: p.is_reimbursable,
        linked_posting_id: p.linked_posting_id.map(|id| id.0),
        reversal_total: p.reversal_total.to_string(),
    }
}

/// 解析系统待处理（pending）标签 ID；系统标签不存在时返回 None（此时各交易恒非 pending）。
async fn pending_tag_id(db: &accounting_sql::SqliteDatabase) -> Result<Option<TagId>, String> {
    Ok(db
        .tag_get_by_name("pending")
        .await
        .map_err(|e| e.to_string())?
        .map(|t| t.id))
}

/// 交易是否已附加系统待处理标签（按标签 ID 判定，规避显示名随界面语言漂移）。
fn tx_has_pending(tag_ids: Option<&Vec<TagId>>, pending_id: Option<TagId>) -> bool {
    match (tag_ids, pending_id) {
        (Some(ids), Some(pid)) => ids.contains(&pid),
        _ => false,
    }
}

/// 创建交易
async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<i64>, String> {
    let db = state.db();

    let date_time = parse_date_time(&req.date_time, &lang).map_err(|e| e.to_string())?;
    let member_id = MemberId(req.member_id);

    let postings = build_postings(db, &lang, TransactionId(0), req.postings).await?;
    let tag_ids = resolve_tag_ids(db, req.tags, &lang).await?;

    let tx_kind = match req.kind.as_str() {
        "refund" => accounting::transaction::TransactionKind::Refund,
        "reimbursement" => accounting::transaction::TransactionKind::Reimbursement,
        _ => accounting::transaction::TransactionKind::Normal,
    };

    let transaction = Transaction {
        id: TransactionId(0),
        date_time,
        description: req.description,
        kind: tx_kind,
        member_id,
    };

    let channel_path_nodes = build_channel_path_nodes(req.channel_paths, &lang)?;

    let service = TransactionService::new(db.clone());
    let id = service
        .submit(transaction, postings, tag_ids, channel_path_nodes)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(id.0))
}

/// 按请求构建分录列表（账户名命中任意语言的名字）
async fn build_postings(
    db: &accounting_sql::SqliteDatabase,
    lang: &str,
    transaction_id: TransactionId,
    requests: Vec<crate::dto::PostingRequest>,
) -> Result<Vec<Posting>, String> {
    let mut postings = Vec::new();
    for posting_req in requests {
        let account = db
            .account_get_by_name(&posting_req.account)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                t!(
                    "tx_err_account_not_found",
                    locale = lang,
                    name = &posting_req.account
                )
                .to_string()
            })?;

        let commodity = db
            .commodity_get_by_symbol(&posting_req.commodity)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                t!(
                    "tx_err_commodity_not_found",
                    locale = lang,
                    name = &posting_req.commodity
                )
                .to_string()
            })?;

        let amount = Decimal::from_str(&posting_req.amount)
            .map_err(|e| t!("tx_err_invalid_amount", locale = lang, error = e).to_string())?;

        postings.push(Posting {
            id: PostingId(0),
            transaction_id,
            account_id: account.id,
            commodity_id: commodity.id,
            amount,
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: posting_req.is_reimbursable,
            linked_posting_id: posting_req.linked_posting_id.map(PostingId),
            reversal_total: Decimal::ZERO,
        });
    }
    Ok(postings)
}

/// 解析标签名列表为标签 ID，不存在时按请求语言创建
async fn resolve_tag_ids(
    db: &accounting_sql::SqliteDatabase,
    tag_names: Vec<String>,
    lang: &str,
) -> Result<Vec<accounting::id::TagId>, String> {
    let mut tag_ids = Vec::new();
    for tag_name in tag_names {
        let tag = db
            .tag_get_by_name(&tag_name)
            .await
            .map_err(|e| e.to_string())?;
        let tag_id = match tag {
            Some(t) => t.id,
            None => db
                .tag_upsert_by_name(&tag_name, None, lang)
                .await
                .map_err(|e| e.to_string())?,
        };
        tag_ids.push(tag_id);
    }
    Ok(tag_ids)
}

/// 按请求构建渠道链路节点列表
fn build_channel_path_nodes(
    nodes: Vec<crate::dto::ChannelPathNodeRequest>,
    lang: &str,
) -> Result<Vec<ChannelPathNode>, String> {
    nodes
        .into_iter()
        .map(|n| {
            let status = n
                .status
                .parse()
                .map_err(|e| t!("tx_err_invalid_status", locale = lang, error = e).to_string())?;
            Ok::<_, String>(ChannelPathNode {
                position: n.position,
                channel_id: ChannelId(n.channel_id),
                status,
            })
        })
        .collect()
}

/// 获取单笔交易（含分录和链路）
async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
) -> Result<Json<TransactionDto>, String> {
    let db = state.db();

    let service = TransactionService::new(db.clone());
    let (tx, postings, channel_paths) = service
        .get(TransactionId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| t!("tx_err_not_found", locale = lang.as_str()).to_string())?;

    let ctx = load_display_context(db, &lang).await?;
    let tag_map = db
        .tag_names_by_transactions(&[tx.id], &lang)
        .await
        .map_err(|e| e.to_string())?;
    let tag_ids_map = db
        .tag_ids_by_transactions(&[tx.id])
        .await
        .map_err(|e| e.to_string())?;
    let pending_id = pending_tag_id(db).await?;

    let posting_dtos: Vec<PostingDto> = postings
        .into_iter()
        .map(|p| posting_to_dto(p, &ctx))
        .collect();

    Ok(Json(TransactionDto {
        id: tx.id.0,
        date_time: tx.date_time.to_string(),
        description: tx.description,
        kind: match tx.kind {
            accounting::transaction::TransactionKind::Refund => "refund".to_string(),
            accounting::transaction::TransactionKind::Reimbursement => "reimbursement".to_string(),
            _ => "normal".to_string(),
        },
        member_id: tx.member_id.0,
        member_name: ctx
            .members
            .get(&tx.member_id.0)
            .cloned()
            .unwrap_or_default(),
        tags: tag_map.get(&tx.id).cloned().unwrap_or_default(),
        pending: tx_has_pending(tag_ids_map.get(&tx.id), pending_id),
        channel_paths: channel_paths
            .into_iter()
            .map(|n| ChannelPathNodeDto {
                position: n.position,
                channel_id: n.channel_id.0,
                channel_name: ctx
                    .channel_names
                    .get(&n.channel_id)
                    .cloned()
                    .unwrap_or_else(|| n.channel_id.0.to_string()),
                status: n.status.as_str().to_string(),
            })
            .collect(),
        postings: posting_dtos,
    }))
}

/// 获取单笔分录
async fn get_posting(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
) -> Result<Json<PostingDto>, String> {
    let db = state.db();
    let posting = db
        .posting_get(PostingId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| t!("tx_err_posting_not_found", locale = lang.as_str()).to_string())?;

    let ctx = load_display_context(db, &lang).await?;

    Ok(Json(posting_to_dto(posting, &ctx)))
}

/// 更新交易
async fn update_transaction(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<String, String> {
    let db = state.db();
    let date_time = parse_date_time(&req.date_time, &lang).map_err(|e| e.to_string())?;
    let member_id = MemberId(req.member_id);

    let postings = build_postings(db, &lang, TransactionId(id), req.postings).await?;
    let tag_ids = resolve_tag_ids(db, req.tags, &lang).await?;

    let tx_kind = match req.kind.as_str() {
        "refund" => accounting::transaction::TransactionKind::Refund,
        "reimbursement" => accounting::transaction::TransactionKind::Reimbursement,
        _ => accounting::transaction::TransactionKind::Normal,
    };

    let transaction = Transaction {
        id: TransactionId(id),
        date_time,
        description: req.description,
        kind: tx_kind,
        member_id,
    };

    let channel_path_nodes = build_channel_path_nodes(req.channel_paths, &lang)?;

    let service = TransactionService::new(db.clone());
    service
        .update(transaction, postings, tag_ids, channel_path_nodes)
        .await
        .map_err(|e| e.to_string())?;
    Ok("updated".to_string())
}

/// 删除交易
async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();
    let service = TransactionService::new(db.clone());
    service
        .delete(TransactionId(id))
        .await
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 对账标记
async fn reconcile_channel_path(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<crate::dto::ReconcileRequest>,
) -> Result<String, String> {
    let db = state.db();
    let service = TransactionService::new(db.clone());
    let status = if req.unset {
        accounting::channel_path::ChannelPathStatus::Default
    } else {
        accounting::channel_path::ChannelPathStatus::Verified
    };
    service
        .update_status(accounting::id::ChannelPathId(id), status)
        .await
        .map_err(|e| e.to_string())?;
    Ok("updated".to_string())
}

/// 构建 账户 ID → 账户类型 映射（沿父链找到根账户，根显示名双语均可被 from_str 接受）
fn build_account_type_map(
    accounts: &HashMap<AccountId, accounting::account::Account>,
    display_names: &HashMap<AccountId, String>,
) -> HashMap<i64, String> {
    accounts
        .keys()
        .map(|id| {
            let mut current = *id;
            loop {
                match accounts.get(&current) {
                    Some(acc) => {
                        if acc.parent_id.is_none() {
                            let name = display_names.get(&acc.id).cloned().unwrap_or_default();
                            let type_str =
                                match accounting::account_type::AccountType::from_str(&name) {
                                    Ok(accounting::account_type::AccountType::Asset) => {
                                        "asset".to_string()
                                    }
                                    Ok(accounting::account_type::AccountType::Equity) => {
                                        "equity".to_string()
                                    }
                                    Ok(accounting::account_type::AccountType::Income) => {
                                        "income".to_string()
                                    }
                                    Ok(accounting::account_type::AccountType::Expense) => {
                                        "expense".to_string()
                                    }
                                    Err(_) => String::new(),
                                };
                            break (id.0, type_str);
                        }
                        match acc.parent_id {
                            Some(parent) => current = parent,
                            None => break (id.0, String::new()),
                        }
                    }
                    None => break (id.0, String::new()),
                }
            }
        })
        .collect()
}

/// 交易路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/transactions",
            get(list_transactions).post(create_transaction),
        )
        .route(
            "/api/transactions/{id}",
            get(get_transaction)
                .put(update_transaction)
                .delete(delete_transaction),
        )
        .route("/api/postings/{id}", get(get_posting))
        .route(
            "/api/channel-paths/{id}/reconcile",
            put(reconcile_channel_path),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::CommodityId;
    use accounting::transaction::TransactionKind;
    use accounting_sql::SqliteDatabase;
    use chrono::NaiveDateTime;
    use rust_decimal::prelude::FromStr;

    async fn setup() -> Arc<AppState> {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        Arc::new(AppState { db })
    }

    async fn account_id(state: &Arc<AppState>, name: &str) -> i64 {
        state
            .db()
            .account_get_by_name(name)
            .await
            .unwrap()
            .unwrap()
            .id
            .0
    }

    /// 创建一笔 35.00 的普通支出（Fees → Cash），可附加指定标签。
    async fn create_expense(state: &Arc<AppState>, tag_ids: Vec<TagId>) -> i64 {
        let service = TransactionService::new(state.db().clone());
        let member_id = state
            .db()
            .member_get_or_create_by_name("测试用户", "zh-CN")
            .await
            .unwrap();
        let fees = AccountId(account_id(state, "Expenses:Fees").await);
        let cash = AccountId(account_id(state, "Assets:Cash").await);
        let cny = CommodityId(
            state
                .db()
                .commodity_list()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.symbol == "CNY")
                .expect("seed 应包含 CNY 商品")
                .id
                .0,
        );
        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDateTime::parse_from_str("2026-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
            description: "午餐".to_string(),
            kind: TransactionKind::Normal,
            member_id,
        };
        let postings = vec![
            Posting {
                id: PostingId(0),
                transaction_id: TransactionId(0),
                account_id: fees,
                commodity_id: cny,
                amount: Decimal::from_str("35.00").unwrap(),
                cost: None,
                cost_commodity_id: None,
                is_reimbursable: false,
                linked_posting_id: None,
                reversal_total: Decimal::ZERO,
            },
            Posting {
                id: PostingId(0),
                transaction_id: TransactionId(0),
                account_id: cash,
                commodity_id: cny,
                amount: Decimal::from_str("-35.00").unwrap(),
                cost: None,
                cost_commodity_id: None,
                is_reimbursable: false,
                linked_posting_id: None,
                reversal_total: Decimal::ZERO,
            },
        ];
        service
            .submit(tx, postings, tag_ids, vec![])
            .await
            .unwrap()
            .0
    }

    #[tokio::test]
    async fn list_transactions_marks_system_pending_tag() {
        let state = setup().await;
        let pending_tag_id = state
            .db()
            .tag_get_by_name("pending")
            .await
            .unwrap()
            .unwrap()
            .id;

        let normal_id = create_expense(&state, vec![]).await;
        let pending_id = create_expense(&state, vec![pending_tag_id]).await;

        let Json(dtos) = list_transactions(
            State(state.clone()),
            Lang("en".to_string()),
            Query(Vec::<(String, String)>::new()),
        )
        .await
        .unwrap();

        let normal = dtos.iter().find(|d| d.id == normal_id).unwrap();
        let pending = dtos.iter().find(|d| d.id == pending_id).unwrap();
        assert!(!normal.pending, "无 pending 标签的交易应为 pending=false");
        assert!(
            pending.pending,
            "带系统 pending 标签的交易应为 pending=true"
        );
        assert_eq!(normal.tags.len(), 0);
        assert!(pending.tags.contains(&"pending".to_string()));
    }

    #[tokio::test]
    async fn get_transaction_marks_system_pending_tag() {
        let state = setup().await;
        let pending_tag_id = state
            .db()
            .tag_get_by_name("pending")
            .await
            .unwrap()
            .unwrap()
            .id;

        let normal_id = create_expense(&state, vec![]).await;
        let pending_id = create_expense(&state, vec![pending_tag_id]).await;

        let Json(normal) = get_transaction(
            State(state.clone()),
            Lang("en".to_string()),
            Path(normal_id),
        )
        .await
        .unwrap();
        let Json(pending) = get_transaction(
            State(state.clone()),
            Lang("en".to_string()),
            Path(pending_id),
        )
        .await
        .unwrap();

        assert!(!normal.pending);
        assert!(pending.pending);
    }
}

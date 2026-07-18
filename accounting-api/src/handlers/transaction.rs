//! 交易 API handler

use crate::dto::{ChannelPathNodeDto, CreateTransactionRequest, PostingDto, TransactionDto};
use crate::handlers::{Lang, member::AppState};
use accounting::channel_path::ChannelPathNode;
use accounting::datetime_utils;
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, MemberId, PostingId, TransactionId};
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
#[derive(serde::Deserialize)]
pub struct TxQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    #[serde(default, deserialize_with = "deserialize_vec_from_single_or_list")]
    pub account: Vec<i64>,
    #[serde(default, deserialize_with = "deserialize_vec_from_single_or_list")]
    pub member: Vec<i64>,
    #[serde(default)]
    pub tag: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec_from_single_or_list")]
    pub channel: Vec<i64>,
    pub keyword: Option<String>,
    pub reimbursable: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 支持单值或多值反序列化（`?account=1&account=2` 或 `?account=1`）
fn deserialize_vec_from_single_or_list<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum SingleOrList<T> {
        List(Vec<T>),
        Single(T),
    }
    match <SingleOrList<T> as serde::Deserialize>::deserialize(deserializer)? {
        SingleOrList::List(v) => Ok(v),
        SingleOrList::Single(v) => Ok(vec![v]),
    }
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
    Query(query): Query<TxQuery>,
) -> Result<Json<Vec<TransactionDto>>, String> {
    let db = state.db();
    let mut filter = TransactionFilter::default();

    if let Some(from) = query.from {
        let date = NaiveDate::parse_from_str(&from, "%Y-%m-%d")
            .map_err(|e| format!("Invalid from date: {}", e))?;
        filter.start_date = Some(date);
    }

    if let Some(to) = query.to {
        let date = NaiveDate::parse_from_str(&to, "%Y-%m-%d")
            .map_err(|e| format!("Invalid to date: {}", e))?;
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
            return Err(format!("Tag not found: {}", tag_name));
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

/// 创建交易
async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<i64>, String> {
    let db = state.db();

    let date_time = parse_date_time(&req.date_time, &lang).map_err(|e| e.to_string())?;
    let member_id = MemberId(req.member_id);

    let postings = build_postings(db, TransactionId(0), req.postings).await?;
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

    let channel_path_nodes = build_channel_path_nodes(req.channel_paths)?;

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
    transaction_id: TransactionId,
    requests: Vec<crate::dto::PostingRequest>,
) -> Result<Vec<Posting>, String> {
    let mut postings = Vec::new();
    for posting_req in requests {
        let account = db
            .account_get_by_name(&posting_req.account)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Account not found: {}", posting_req.account))?;

        let commodity = db
            .commodity_get_by_symbol(&posting_req.commodity)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Commodity not found: {}", posting_req.commodity))?;

        let amount =
            Decimal::from_str(&posting_req.amount).map_err(|e| format!("Invalid amount: {}", e))?;

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
) -> Result<Vec<ChannelPathNode>, String> {
    nodes
        .into_iter()
        .map(|n| {
            let status = n
                .status
                .parse()
                .map_err(|e| format!("Invalid status: {}", e))?;
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
        .ok_or("Transaction not found")?;

    let ctx = load_display_context(db, &lang).await?;
    let tag_map = db
        .tag_names_by_transactions(&[tx.id], &lang)
        .await
        .map_err(|e| e.to_string())?;

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
        .ok_or("Posting not found")?;

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

    let postings = build_postings(db, TransactionId(id), req.postings).await?;
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

    let channel_path_nodes = build_channel_path_nodes(req.channel_paths)?;

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

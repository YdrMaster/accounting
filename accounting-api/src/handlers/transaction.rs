//! 交易 API handler

use crate::dto::{ChannelPathNodeDto, CreateTransactionRequest, PostingDto, TransactionDto};
use crate::handlers::member::AppState;
use accounting::channel_path::ChannelPathNode;
use accounting::datetime_utils;
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, MemberId, PostingId, TagId, TransactionId};
use accounting::posting::Posting;
use accounting::tag::Tag;
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
fn parse_date_time(s: &str) -> Result<chrono::NaiveDateTime, AccountingError> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(datetime_utils::start_of_day)
        .map_err(|_| AccountingError::InvalidDate(t!("invalid_date_format", value = s).to_string()))
}

/// 列出交易（含筛选）
async fn list_transactions(
    State(state): State<Arc<AppState>>,
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

    let (account_paths, commodities, members, account_types, tag_map) = {
        let accounts: std::collections::HashMap<
            accounting::id::AccountId,
            accounting::account::Account,
        > = db
            .account_list()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|a| (a.id, a))
            .collect();
        let account_paths: std::collections::HashMap<i64, String> = accounts
            .values()
            .map(|a| (a.id.0, a.display_path(&accounts)))
            .collect();
        let commodities: std::collections::HashMap<i64, String> = db
            .commodity_list()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|c| (c.id.0, c.symbol))
            .collect();
        let members: std::collections::HashMap<i64, String> = db
            .member_list()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|m| (m.id.0, m.name))
            .collect();
        let account_types = build_account_type_map(&accounts);
        let tx_ids: Vec<accounting::id::TransactionId> =
            transactions.iter().map(|(tx, _, _)| tx.id).collect();
        let tag_map = db
            .tag_names_by_transactions(&tx_ids)
            .await
            .map_err(|e| e.to_string())?;
        (account_paths, commodities, members, account_types, tag_map)
    };

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
            member_name: members.get(&tx.member_id.0).cloned().unwrap_or_default(),
            tags: tag_map.get(&tx.id).cloned().unwrap_or_default(),
            channel_paths: channel_paths
                .into_iter()
                .map(|n| ChannelPathNodeDto {
                    position: n.position,
                    channel_id: n.channel_id.0,
                    reconciled: n.reconciled,
                })
                .collect(),
            postings: postings
                .into_iter()
                .map(|p| PostingDto {
                    id: p.id.0,
                    transaction_id: p.transaction_id.0,
                    account: account_paths
                        .get(&p.account_id.0)
                        .cloned()
                        .unwrap_or_default(),
                    account_type: account_types
                        .get(&p.account_id.0)
                        .cloned()
                        .unwrap_or_default(),
                    commodity: commodities
                        .get(&p.commodity_id.0)
                        .cloned()
                        .unwrap_or_default(),
                    amount: p.amount.to_string(),
                    is_reimbursable: p.is_reimbursable,
                    linked_posting_id: p.linked_posting_id.map(|id| id.0),
                    reversal_total: p.reversal_total.to_string(),
                })
                .collect(),
        })
        .collect();

    Ok(Json(dtos))
}

/// 创建交易
async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<i64>, String> {
    let db = state.db();

    let date_time = parse_date_time(&req.date_time).map_err(|e| e.to_string())?;
    let member_id = MemberId(req.member_id);

    let mut postings = Vec::new();
    for posting_req in req.postings {
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
            transaction_id: TransactionId(0),
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

    let mut tag_ids = Vec::new();
    for tag_name in req.tags {
        let tag = db
            .tag_get_by_name(&tag_name)
            .await
            .map_err(|e| e.to_string())?;
        let tag_id = match tag {
            Some(t) => t.id,
            None => {
                let new_tag = Tag {
                    id: TagId(0),
                    name: tag_name.clone(),
                    description: None,
                    is_system: false,
                };
                db.tag_create(&new_tag).await.map_err(|e| e.to_string())?
            }
        };
        tag_ids.push(tag_id);
    }

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

    let channel_path_nodes: Vec<ChannelPathNode> = req
        .channel_paths
        .into_iter()
        .map(|n| ChannelPathNode {
            position: n.position,
            channel_id: ChannelId(n.channel_id),
            reconciled: false,
        })
        .collect();

    let service = TransactionService::new(db.clone());
    let id = service
        .submit(transaction, postings, tag_ids, channel_path_nodes)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(id.0))
}

/// 获取单笔交易（含分录和链路）
async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<TransactionDto>, String> {
    let db = state.db();

    let service = TransactionService::new(db.clone());
    let (tx, postings, channel_paths) = service
        .get(TransactionId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Transaction not found")?;

    // 批量查询账户和商品名称
    let accounts: std::collections::HashMap<
        accounting::id::AccountId,
        accounting::account::Account,
    > = db
        .account_list()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| (a.id, a))
        .collect();
    let account_paths: std::collections::HashMap<i64, String> = accounts
        .values()
        .map(|a| (a.id.0, a.display_path(&accounts)))
        .collect();
    let account_types = build_account_type_map(&accounts);

    let commodities: std::collections::HashMap<i64, String> = db
        .commodity_list()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| (c.id.0, c.symbol))
        .collect();

    let members: std::collections::HashMap<i64, String> = db
        .member_list()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|m| (m.id.0, m.name))
        .collect();

    let tag_map = db
        .tag_names_by_transactions(&[tx.id])
        .await
        .map_err(|e| e.to_string())?;

    let posting_dtos: Vec<PostingDto> = postings
        .into_iter()
        .map(|p| PostingDto {
            id: p.id.0,
            transaction_id: p.transaction_id.0,
            account: account_paths
                .get(&p.account_id.0)
                .cloned()
                .unwrap_or_default(),
            account_type: account_types
                .get(&p.account_id.0)
                .cloned()
                .unwrap_or_default(),
            commodity: commodities
                .get(&p.commodity_id.0)
                .cloned()
                .unwrap_or_default(),
            amount: p.amount.to_string(),
            is_reimbursable: p.is_reimbursable,
            linked_posting_id: p.linked_posting_id.map(|id| id.0),
            reversal_total: p.reversal_total.to_string(),
        })
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
        member_name: members.get(&tx.member_id.0).cloned().unwrap_or_default(),
        tags: tag_map.get(&tx.id).cloned().unwrap_or_default(),
        channel_paths: channel_paths
            .into_iter()
            .map(|n| ChannelPathNodeDto {
                position: n.position,
                channel_id: n.channel_id.0,
                reconciled: n.reconciled,
            })
            .collect(),
        postings: posting_dtos,
    }))
}

/// 获取单笔分录
async fn get_posting(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<PostingDto>, String> {
    let db = state.db();
    let posting = db
        .posting_get(PostingId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Posting not found")?;

    let accounts: std::collections::HashMap<
        accounting::id::AccountId,
        accounting::account::Account,
    > = db
        .account_list()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| (a.id, a))
        .collect();
    let account_paths: std::collections::HashMap<i64, String> = accounts
        .values()
        .map(|a| (a.id.0, a.display_path(&accounts)))
        .collect();
    let account_types = build_account_type_map(&accounts);
    let commodities: std::collections::HashMap<i64, String> = db
        .commodity_list()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| (c.id.0, c.symbol))
        .collect();

    Ok(Json(PostingDto {
        id: posting.id.0,
        transaction_id: posting.transaction_id.0,
        account: account_paths
            .get(&posting.account_id.0)
            .cloned()
            .unwrap_or_default(),
        account_type: account_types
            .get(&posting.account_id.0)
            .cloned()
            .unwrap_or_default(),
        commodity: commodities
            .get(&posting.commodity_id.0)
            .cloned()
            .unwrap_or_default(),
        amount: posting.amount.to_string(),
        is_reimbursable: posting.is_reimbursable,
        linked_posting_id: posting.linked_posting_id.map(|id| id.0),
        reversal_total: posting.reversal_total.to_string(),
    }))
}

/// 更新交易
async fn update_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<String, String> {
    let db = state.db();
    let date_time = parse_date_time(&req.date_time).map_err(|e| e.to_string())?;
    let member_id = MemberId(req.member_id);

    let (postings, tag_ids) = {
        let mut postings = Vec::new();
        for posting_req in req.postings {
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

            let amount = Decimal::from_str(&posting_req.amount)
                .map_err(|e| format!("Invalid amount: {}", e))?;

            postings.push(Posting {
                id: PostingId(0),
                transaction_id: TransactionId(id),
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

        let mut tag_ids = Vec::new();
        for tag_name in req.tags {
            let tag = db
                .tag_get_by_name(&tag_name)
                .await
                .map_err(|e| e.to_string())?;
            let tag_id = match tag {
                Some(t) => t.id,
                None => {
                    let new_tag = Tag {
                        id: TagId(0),
                        name: tag_name.clone(),
                        description: None,
                        is_system: false,
                    };
                    db.tag_create(&new_tag).await.map_err(|e| e.to_string())?
                }
            };
            tag_ids.push(tag_id);
        }

        (postings, tag_ids)
    };

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

    let channel_path_nodes: Vec<ChannelPathNode> = req
        .channel_paths
        .into_iter()
        .map(|n| ChannelPathNode {
            position: n.position,
            channel_id: ChannelId(n.channel_id),
            reconciled: false,
        })
        .collect();

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
    service
        .update_reconciled(accounting::id::ChannelPathId(id), req.reconciled)
        .await
        .map_err(|e| e.to_string())?;
    Ok("updated".to_string())
}

fn build_account_type_map(
    accounts: &std::collections::HashMap<accounting::id::AccountId, accounting::account::Account>,
) -> std::collections::HashMap<i64, String> {
    use std::str::FromStr;
    accounts
        .keys()
        .map(|id| {
            let mut current = *id;
            loop {
                match accounts.get(&current) {
                    Some(acc) => {
                        if acc.parent_id.is_none() {
                            let type_str =
                                match accounting::account_type::AccountType::from_str(&acc.name) {
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

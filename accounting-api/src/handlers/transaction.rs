//! 交易 API handler

use crate::dto::{CreateTransactionRequest, PostingDto, TransactionDto};
use crate::handlers::member::AppState;
use accounting::datetime_utils;
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, MemberId, PostingId, TagId, TransactionId};
use accounting::posting::Posting;
use accounting::tag::Tag;
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use accounting_service::transaction_service::TransactionService;
use accounting_sql::database::Database;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
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
    #[serde(deserialize_with = "deserialize_vec_from_single_or_list")]
    pub account: Vec<i64>,
    #[serde(deserialize_with = "deserialize_vec_from_single_or_list")]
    pub member: Vec<i64>,
    #[serde(default)]
    pub tag: Vec<String>,
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
    let db = state.db().map_err(|e| e.to_string())?;
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

    for tag_name in &query.tag {
        let tag = db
            .tag_repo()
            .get_by_name(&db.connection(), tag_name)
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

    let (account_paths, commodities) = {
        let conn = db.connection();
        let accounts: std::collections::HashMap<
            accounting::id::AccountId,
            accounting::account::Account,
        > = db
            .account_repo()
            .list(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|a| (a.id, a))
            .collect();
        let account_paths: std::collections::HashMap<i64, String> = accounts
            .values()
            .map(|a| (a.id.0, a.display_path(&accounts)))
            .collect();
        let commodities: std::collections::HashMap<i64, String> = db
            .commodity_repo()
            .list(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|c| (c.id.0, c.symbol))
            .collect();
        (account_paths, commodities)
    };

    let service = TransactionService::new(db);
    let transactions = service
        .list(filter, query.limit, query.offset)
        .await
        .map_err(|e| e.to_string())?;

    let dtos: Vec<TransactionDto> = transactions
        .into_iter()
        .map(|(tx, postings)| TransactionDto {
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
            member_id: tx.member_id.map(|id| id.0),
            channel_id: tx.channel_id.map(|id| id.0),
            postings: postings
                .into_iter()
                .map(|p| PostingDto {
                    id: p.id.0,
                    transaction_id: p.transaction_id.0,
                    account: account_paths
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
    let db = state.db().map_err(|e| e.to_string())?;

    let date_time = parse_date_time(&req.date_time).map_err(|e| e.to_string())?;
    let member_id = req.member_id.map(MemberId);
    let tx_description = req.description;

    let mut postings = Vec::new();
    for posting_req in req.postings {
        let account = db
            .account_repo()
            .get_by_name(&db.connection(), &posting_req.account)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Account not found: {}", posting_req.account))?;

        let commodity = db
            .commodity_repo()
            .get_by_symbol(&db.connection(), &posting_req.commodity)
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
            description: None,
            is_reimbursable: posting_req.is_reimbursable,
            linked_posting_id: posting_req.linked_posting_id.map(PostingId),
            reversal_total: Decimal::ZERO,
        });
    }

    let mut tag_ids = Vec::new();
    for tag_name in req.tags {
        let tag = db
            .tag_repo()
            .get_by_name(&db.connection(), &tag_name)
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
                db.tag_repo()
                    .create(&db.connection(), &new_tag)
                    .map_err(|e| e.to_string())?
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
        description: tx_description,
        kind: tx_kind,
        member_id,
        channel_id: req.channel_id.map(ChannelId),
    };

    let service = TransactionService::new(db);
    let id = service
        .submit(transaction, postings, tag_ids)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(id.0))
}

/// 获取单笔交易（含分录）
async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<TransactionDto>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();

    let tx = db
        .transaction_repo()
        .get(&conn, TransactionId(id))
        .map_err(|e| e.to_string())?
        .ok_or("Transaction not found")?;

    let postings = db
        .posting_repo()
        .list_by_transaction(&conn, TransactionId(id))
        .map_err(|e| e.to_string())?;

    // 批量查询账户和商品名称
    let accounts: std::collections::HashMap<
        accounting::id::AccountId,
        accounting::account::Account,
    > = db
        .account_repo()
        .list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| (a.id, a))
        .collect();
    let account_paths: std::collections::HashMap<i64, String> = accounts
        .values()
        .map(|a| (a.id.0, a.display_path(&accounts)))
        .collect();

    let commodities: std::collections::HashMap<i64, String> = db
        .commodity_repo()
        .list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| (c.id.0, c.symbol))
        .collect();

    let posting_dtos: Vec<PostingDto> = postings
        .into_iter()
        .map(|p| PostingDto {
            id: p.id.0,
            transaction_id: p.transaction_id.0,
            account: account_paths
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
        member_id: tx.member_id.map(|id| id.0),
        channel_id: tx.channel_id.map(|id| id.0),
        postings: posting_dtos,
    }))
}

/// 获取单笔分录
async fn get_posting(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<PostingDto>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    let posting = db
        .posting_repo()
        .get(&conn, PostingId(id))
        .map_err(|e| e.to_string())?
        .ok_or("Posting not found")?;

    let accounts: std::collections::HashMap<
        accounting::id::AccountId,
        accounting::account::Account,
    > = db
        .account_repo()
        .list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| (a.id, a))
        .collect();
    let account_paths: std::collections::HashMap<i64, String> = accounts
        .values()
        .map(|a| (a.id.0, a.display_path(&accounts)))
        .collect();
    let commodities: std::collections::HashMap<i64, String> = db
        .commodity_repo()
        .list(&conn)
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
    let db = state.db().map_err(|e| e.to_string())?;
    let date_time = parse_date_time(&req.date_time).map_err(|e| e.to_string())?;
    let member_id = req.member_id.map(MemberId);

    let (postings, tag_ids) = {
        let conn = db.connection();

        let mut postings = Vec::new();
        for posting_req in req.postings {
            let account = db
                .account_repo()
                .get_by_name(&conn, &posting_req.account)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Account not found: {}", posting_req.account))?;

            let commodity = db
                .commodity_repo()
                .get_by_symbol(&conn, &posting_req.commodity)
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
                description: None,
                is_reimbursable: posting_req.is_reimbursable,
                linked_posting_id: posting_req.linked_posting_id.map(PostingId),
                reversal_total: Decimal::ZERO,
            });
        }

        let mut tag_ids = Vec::new();
        for tag_name in req.tags {
            let tag = db
                .tag_repo()
                .get_by_name(&conn, &tag_name)
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
                    db.tag_repo()
                        .create(&conn, &new_tag)
                        .map_err(|e| e.to_string())?
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
        channel_id: req.channel_id.map(ChannelId),
    };

    let service = TransactionService::new(db);
    service
        .update(transaction, postings, tag_ids)
        .await
        .map_err(|e| e.to_string())?;
    Ok("updated".to_string())
}

/// 删除交易
async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let service = TransactionService::new(db);
    service
        .delete(TransactionId(id))
        .await
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
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
}

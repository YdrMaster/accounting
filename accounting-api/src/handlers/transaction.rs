//! 交易 API handler

use crate::dto::{CreateTransactionRequest, TransactionDto};
use crate::handlers::member::AppState;
use accounting::error::AccountingError;
use accounting::id::{AccountId, MemberId, PostingId, TagId, TransactionId};
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
use std::str::FromStr;
use std::sync::Arc;

/// 交易列表查询参数
#[derive(serde::Deserialize)]
pub struct TxQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub account: Option<i64>,
    pub member: Option<i64>,
    pub tag: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 解析日期时间字符串
fn parse_date_time(s: &str) -> Result<chrono::NaiveDateTime, AccountingError> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        .map_err(|_| {
            AccountingError::InvalidDate(format!(
                "时间格式应为 YYYY-MM-DD 或 YYYY-MM-DD HH:MM:SS: {}",
                s
            ))
        })
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

    if let Some(account) = query.account {
        filter.account_id = Some(AccountId(account));
    }

    if let Some(member) = query.member {
        filter.member_id = Some(MemberId(member));
    }

    if let Some(tag_name) = query.tag {
        let tag = db
            .tag_repo()
            .get_by_name(&db.connection(), &tag_name)
            .map_err(|e| e.to_string())?;
        if let Some(tag) = tag {
            filter.tag_id = Some(tag.id);
        }
    }

    if let Some(keyword) = query.keyword {
        filter.keyword = Some(keyword);
    }

    let service = TransactionService::new(db);
    let transactions = service
        .list(filter, query.limit, query.offset)
        .await
        .map_err(|e| e.to_string())?;

    let dtos: Vec<TransactionDto> = transactions
        .into_iter()
        .map(|(tx, _)| TransactionDto {
            id: tx.id.0,
            date_time: tx.date_time.to_string(),
            description: tx.description,
            member_id: tx.member_id.map(|id| id.0),
            is_template: tx.is_template,
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
            member_id,
            channel_id: None,
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

    let transaction = Transaction {
        id: TransactionId(0),
        date_time,
        description: tx_description,
        member_id,
        is_template: false,
    };

    let service = TransactionService::new(db);
    let id = service
        .submit(transaction, postings, tag_ids)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(id.0))
}

/// 获取单笔交易
async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<TransactionDto>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let service = TransactionService::new(db);
    let result = service
        .get(TransactionId(id))
        .await
        .map_err(|e| e.to_string())?;

    match result {
        Some((tx, _)) => Ok(Json(TransactionDto {
            id: tx.id.0,
            date_time: tx.date_time.to_string(),
            description: tx.description,
            member_id: tx.member_id.map(|id| id.0),
            is_template: tx.is_template,
        })),
        None => Err("Transaction not found".to_string()),
    }
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
            "/api/transactions/:id",
            get(get_transaction).delete(delete_transaction),
        )
}

//! 账户映射 API handler

use crate::dto::{MappingDto, SetMappingRequest};
use crate::handlers::{Lang, member::AppState};
use accounting::id::{AccountId, ChannelId, MemberId};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use rust_i18n::t;
use std::sync::Arc;

/// 映射查询参数
#[derive(serde::Deserialize)]
pub struct MappingQuery {
    pub member_id: i64,
    pub channel_id: i64,
}

/// 删除映射查询参数
#[derive(serde::Deserialize)]
pub struct DeleteMappingQuery {
    pub member_id: i64,
    pub channel_id: i64,
    pub category: String,
}

/// 列出某个 (成员, 渠道) 的所有映射
async fn list_mappings(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MappingQuery>,
) -> Result<Json<Vec<MappingDto>>, String> {
    let db = state.db();
    let service = accounting_service::mapping_service::MappingService::new(db.clone());
    let mappings = service
        .list(MemberId(query.member_id), ChannelId(query.channel_id))
        .await
        .map_err(|e| e.to_string())?;
    let dtos = mappings
        .into_iter()
        .map(|m| MappingDto {
            member_id: m.member_id.0,
            channel_id: m.channel_id.0,
            category: m.category,
            account_id: m.account_id.0,
        })
        .collect();
    Ok(Json(dtos))
}

/// 设置映射（upsert 语义，重复设置覆盖）
async fn set_mapping(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Json(req): Json<SetMappingRequest>,
) -> Result<String, String> {
    let db = state.db();
    // Web 场景直接给账户 ID：先校验账户存在，再直写映射，不走路径解析
    db.account_get(AccountId(req.account_id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| t!("account_not_found", locale = lang.as_str()).to_string())?;

    let mapping = accounting::account_mapping::AccountMapping {
        member_id: MemberId(req.member_id),
        channel_id: ChannelId(req.channel_id),
        category: req.category,
        account_id: AccountId(req.account_id),
    };
    db.account_mapping_upsert(&mapping)
        .await
        .map_err(|e| e.to_string())?;
    Ok("ok".to_string())
}

/// 删除单条映射（不存在返回错误）
async fn delete_mapping(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DeleteMappingQuery>,
) -> Result<String, String> {
    let db = state.db();
    let service = accounting_service::mapping_service::MappingService::new(db.clone());
    service
        .delete(
            MemberId(query.member_id),
            ChannelId(query.channel_id),
            &query.category,
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 账户映射路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route(
        "/api/mappings",
        get(list_mappings).put(set_mapping).delete(delete_mapping),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::id::{ChannelId, MemberId};
    use accounting_sql::SqliteDatabase;

    async fn setup() -> (Arc<AppState>, MemberId, ChannelId) {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let member_id = db
            .member_get_or_create_by_name("Test User", "en")
            .await
            .unwrap();
        let channel_id = db
            .channel_upsert_by_name("TestPay", None, None, "en")
            .await
            .unwrap();
        (Arc::new(AppState { db }), member_id, channel_id)
    }

    async fn expenses_id(state: &Arc<AppState>) -> i64 {
        state
            .db()
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .unwrap()
            .id
            .0
    }

    #[tokio::test]
    async fn list_mappings_returns_seed_entries() {
        let (state, member_id, channel_id) = setup().await;
        let account_id = expenses_id(&state).await;

        set_mapping(
            State(state.clone()),
            Lang("en".to_string()),
            Json(SetMappingRequest {
                member_id: member_id.0,
                channel_id: channel_id.0,
                category: "Expenses:餐饮美食".to_string(),
                account_id,
            }),
        )
        .await
        .unwrap();

        let Json(list) = list_mappings(
            State(state.clone()),
            Query(MappingQuery {
                member_id: member_id.0,
                channel_id: channel_id.0,
            }),
        )
        .await
        .unwrap();

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].member_id, member_id.0);
        assert_eq!(list[0].channel_id, channel_id.0);
        assert_eq!(list[0].category, "Expenses:餐饮美食");
        assert_eq!(list[0].account_id, account_id);
    }

    #[tokio::test]
    async fn list_mappings_empty() {
        let (state, member_id, channel_id) = setup().await;

        let Json(list) = list_mappings(
            State(state.clone()),
            Query(MappingQuery {
                member_id: member_id.0,
                channel_id: channel_id.0,
            }),
        )
        .await
        .unwrap();

        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn set_mapping_rejects_missing_account() {
        let (state, member_id, channel_id) = setup().await;

        let result = set_mapping(
            State(state.clone()),
            Lang("en".to_string()),
            Json(SetMappingRequest {
                member_id: member_id.0,
                channel_id: channel_id.0,
                category: "Expenses:餐饮美食".to_string(),
                account_id: 99999,
            }),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn set_mapping_overwrites_existing_key() {
        let (state, member_id, channel_id) = setup().await;
        let expenses = expenses_id(&state).await;
        let fees = state
            .db()
            .account_get_by_name("Expenses:Fees")
            .await
            .unwrap()
            .unwrap()
            .id
            .0;

        for account_id in [expenses, fees] {
            set_mapping(
                State(state.clone()),
                Lang("en".to_string()),
                Json(SetMappingRequest {
                    member_id: member_id.0,
                    channel_id: channel_id.0,
                    category: "Expenses:餐饮美食".to_string(),
                    account_id,
                }),
            )
            .await
            .unwrap();
        }

        let Json(list) = list_mappings(
            State(state.clone()),
            Query(MappingQuery {
                member_id: member_id.0,
                channel_id: channel_id.0,
            }),
        )
        .await
        .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].account_id, fees);
    }

    #[tokio::test]
    async fn delete_mapping_removes_entry() {
        let (state, member_id, channel_id) = setup().await;
        let account_id = expenses_id(&state).await;

        set_mapping(
            State(state.clone()),
            Lang("en".to_string()),
            Json(SetMappingRequest {
                member_id: member_id.0,
                channel_id: channel_id.0,
                category: "Expenses:餐饮美食".to_string(),
                account_id,
            }),
        )
        .await
        .unwrap();

        delete_mapping(
            State(state.clone()),
            Query(DeleteMappingQuery {
                member_id: member_id.0,
                channel_id: channel_id.0,
                category: "Expenses:餐饮美食".to_string(),
            }),
        )
        .await
        .unwrap();

        let Json(list) = list_mappings(
            State(state.clone()),
            Query(MappingQuery {
                member_id: member_id.0,
                channel_id: channel_id.0,
            }),
        )
        .await
        .unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn delete_mapping_nonexistent_returns_err() {
        let (state, member_id, channel_id) = setup().await;

        let result = delete_mapping(
            State(state.clone()),
            Query(DeleteMappingQuery {
                member_id: member_id.0,
                channel_id: channel_id.0,
                category: "Expenses:不存在".to_string(),
            }),
        )
        .await;
        assert!(result.is_err());
    }
}

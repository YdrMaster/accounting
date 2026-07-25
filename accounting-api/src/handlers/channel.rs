//! 渠道 API handler

use crate::dto::{ChannelDto, CreateChannelRequest, UpdateChannelRequest};
use crate::handlers::{Lang, member::AppState};
use accounting::id::{AccountId, ChannelId};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, put},
};
use rust_i18n::t;
use std::sync::Arc;

/// 列出所有渠道
async fn list_channels(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
) -> Result<Json<Vec<ChannelDto>>, String> {
    let db = state.db();
    let channels = db.channel_list().await.map_err(|e| e.to_string())?;
    let ids: Vec<ChannelId> = channels.iter().map(|c| c.id).collect();
    let names = db
        .channel_display_names(&ids, &lang)
        .await
        .map_err(|e| e.to_string())?;

    // 适配器名字列表（转为 owned String，避免非 Send 的 dyn BillAdapter 跨 await 持有）
    let adapter_names: Vec<String> = accounting_service::import::builtin_adapters()
        .iter()
        .flat_map(|a| a.names().iter().map(|s| s.to_string()))
        .collect();
    let mut dtos = Vec::with_capacity(channels.len());
    for c in channels {
        // 渠道的任一语言名字能匹配某个内置适配器，即视为关联了导入适配器
        let all_names = db
            .channel_names_by_id(c.id)
            .await
            .map_err(|e| e.to_string())?;
        let has_import_adapter = all_names
            .iter()
            .any(|n| adapter_names.iter().any(|an| an.eq_ignore_ascii_case(n)));
        dtos.push(ChannelDto {
            id: c.id.0,
            name: names.get(&c.id).cloned().unwrap_or_default(),
            description: c.description,
            account_id: c.account_id.map(|id| id.0),
            is_system: c.is_system,
            has_import_adapter,
        });
    }

    Ok(Json(dtos))
}

/// 创建渠道
async fn create_channel(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Json(req): Json<CreateChannelRequest>,
) -> Result<Json<i64>, String> {
    let db = state.db();
    // 重名拒绝（NOCASE 全语言匹配）：upsert 语义会把创建变成对既有渠道（含内置渠道）的静默更新
    if db
        .channel_resolve_by_name(&req.name)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Err(t!("channel_name_exists", locale = lang.as_str()).to_string());
    }
    let id = db
        .channel_upsert_by_name(
            &req.name,
            req.description.as_deref(),
            req.account_id.map(AccountId),
            &lang,
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(id.0))
}

/// 更新渠道（改名 = 改写请求语言下的显示名；description、account_id 直接更新）
async fn update_channel(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Json(req): Json<UpdateChannelRequest>,
) -> Result<String, String> {
    let db = state.db();
    let existing = db
        .channel_get(ChannelId(id))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| t!("channel_not_found", locale = lang.as_str()).to_string())?;

    // name 三态：None=不修改；Some(Some(v))=改名；Some(None)=清空（名字不可为空，拒绝）
    if let Some(name) = &req.name {
        match name {
            Some(v) => {
                db.channel_rename(ChannelId(id), v, &lang)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            None => {
                return Err(t!("channel_name_required", locale = lang.as_str()).to_string());
            }
        }
    }

    let new_description = match req.description {
        Some(ref v) => v.as_deref(),
        None => existing.description.as_deref(),
    };
    let new_account_id = match req.account_id {
        Some(v) => v.map(AccountId),
        None => existing.account_id,
    };

    db.channel_update(ChannelId(id), new_description, new_account_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok("updated".to_string())
}

/// 删除渠道
async fn delete_channel(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db();
    // 检查渠道是否被 channel_paths 引用
    let count = db
        .channel_count_transactions_by_id(ChannelId(id))
        .await
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Err(t!("channel_in_use", locale = lang.as_str()).to_string());
    }
    db.channel_force_delete_by_id(ChannelId(id))
        .await
        .map_err(|e| e.to_string())?;
    Ok("deleted".to_string())
}

/// 渠道路由
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/channels", get(list_channels).post(create_channel))
        .route(
            "/api/channels/{id}",
            put(update_channel).delete(delete_channel),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    async fn setup() -> Arc<AppState> {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        Arc::new(AppState { db })
    }

    #[tokio::test]
    async fn list_channels_flags_builtin_channel_with_adapter() {
        let state = setup().await;
        let Json(dtos) = list_channels(State(state), Lang("zh-CN".to_string()))
            .await
            .unwrap();
        let alipay = dtos.iter().find(|c| c.name == "支付宝").unwrap();
        assert!(alipay.has_import_adapter);
    }

    #[tokio::test]
    async fn list_channels_user_channel_has_no_adapter() {
        let state = setup().await;
        state
            .db()
            .channel_upsert_by_name("云闪付", None, None, "zh-CN")
            .await
            .unwrap();
        let Json(dtos) = list_channels(State(state), Lang("zh-CN".to_string()))
            .await
            .unwrap();
        let unionpay = dtos.iter().find(|c| c.name == "云闪付").unwrap();
        assert!(!unionpay.has_import_adapter);
    }

    #[tokio::test]
    async fn create_channel_rejects_existing_name_exact() {
        let state = setup().await;
        let result = create_channel(
            State(state.clone()),
            Lang("zh-CN".to_string()),
            Json(CreateChannelRequest {
                name: "支付宝".to_string(),
                description: Some("试图劫持".to_string()),
                account_id: None,
            }),
        )
        .await;
        assert!(result.is_err());
        // 内置渠道不应被篡改
        let builtin = state
            .db()
            .channel_get(ChannelId(1))
            .await
            .unwrap()
            .unwrap();
        assert!(builtin.description.is_none());
    }

    #[tokio::test]
    async fn create_channel_rejects_existing_name_case_variant() {
        let state = setup().await;
        let result = create_channel(
            State(state),
            Lang("en".to_string()),
            Json(CreateChannelRequest {
                name: "ALIPAY".to_string(),
                description: None,
                account_id: None,
            }),
        )
        .await;
        assert!(result.is_err());
    }
}

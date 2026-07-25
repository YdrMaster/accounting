//! 渠道 API handler

use crate::dto::{
    ChannelDto, CreateChannelRequest, ImportResultDto, ImportRowErrorDto, UpdateChannelRequest,
};
use crate::handlers::{Lang, member::AppState};
use accounting::id::{AccountId, ChannelId, MemberId};
use accounting_service::import::AdaptError;
use accounting_service::import_service::ImportService;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
};
use rust_i18n::t;
use serde::Deserialize;
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

/// 导入账单请求 query 参数
#[derive(Deserialize)]
pub struct ImportBillQuery {
    /// 导入交易归属的成员 ID（前端当前选中成员，与交易表单一致）
    pub member_id: i64,
}

/// 导入账单文件
///
/// body 为账单文件原始字节。按渠道 id 反查全语言名字匹配内置适配器，
/// 以命中名作为 source 调用 ImportService；导入成员由 query 参数显式指定
/// （与交易创建、映射等 API 的惯例一致，Web 端无服务端"当前用户"概念）。
async fn import_bill(
    State(state): State<Arc<AppState>>,
    Lang(lang): Lang,
    Path(id): Path<i64>,
    Query(query): Query<ImportBillQuery>,
    body: Bytes,
) -> Result<Json<ImportResultDto>, (StatusCode, String)> {
    let db = state.db();
    let internal = |e: String| (StatusCode::INTERNAL_SERVER_ERROR, e);

    // 渠道必须存在
    let channel = db
        .channel_get(ChannelId(id))
        .await
        .map_err(|e| internal(e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                t!("channel_not_found", locale = lang.as_str()).to_string(),
            )
        })?;

    // 反查全语言名字，匹配内置适配器（与 list_channels 同一判定逻辑，对改名免疫）
    let names = db
        .channel_names_by_id(channel.id)
        .await
        .map_err(|e| internal(e.to_string()))?;
    // 适配器名字列表转为 owned String，避免非 Send 的 dyn BillAdapter 跨 await 持有
    let adapter_names: Vec<String> = accounting_service::import::builtin_adapters()
        .iter()
        .flat_map(|a| a.names().iter().map(|s| s.to_string()))
        .collect();
    let source = names
        .iter()
        .find(|n| adapter_names.iter().any(|an| an.eq_ignore_ascii_case(n)))
        .cloned()
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                t!("channel_no_import_adapter", locale = lang.as_str()).to_string(),
            )
        })?;

    // 导入成员由调用方显式指定（Web 端当前选中成员），必须存在
    let member_id = query.member_id;
    db.member_get(MemberId(member_id))
        .await
        .map_err(|e| internal(e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                t!("member_not_found", locale = lang.as_str()).to_string(),
            )
        })?;

    let result = ImportService::new(db.clone())
        .import(&body, &source, MemberId(member_id))
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                t!(
                    "import_failed",
                    locale = lang.as_str(),
                    error = e.to_string()
                )
                .to_string(),
            )
        })?;

    // 适配器对非账单文件是宽容的（逐行跳过，不报错）：imported=0 且 skipped=0
    // 说明文件中没有任何可识别的数据行，按解析失败处理，避免"导入 0 条"的假成功
    if result.imported == 0 && result.skipped == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            t!("import_no_entries", locale = lang.as_str()).to_string(),
        ));
    }

    let errors = result
        .errors
        .iter()
        .map(|e| match e {
            AdaptError::Row { row, detail } => ImportRowErrorDto {
                row: *row,
                detail: detail.to_string(),
            },
            AdaptError::Encoding { .. } => ImportRowErrorDto {
                row: 0,
                detail: e.to_string(),
            },
        })
        .collect();

    Ok(Json(ImportResultDto {
        imported: result.imported,
        skipped: result.skipped,
        pending_tag_name: result.pending_tag_name,
        errors,
    }))
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
        // 账单导入：裸字节 body，放宽默认 2MB 上限以容纳多年账单
        .route(
            "/api/channels/{id}/import",
            post(import_bill).layer(DefaultBodyLimit::max(32 * 1024 * 1024)),
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
        let builtin = state.db().channel_get(ChannelId(1)).await.unwrap().unwrap();
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

    const ALIPAY_CSV: &[u8] = concat!(
        "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
        "2024-01-15 12:30:00,餐饮美食,美团外卖,mei***@tuan.com,美团外卖-午餐,支出,35.00,蚂蚁宝藏信用卡,交易成功,2024011522001470000001\t,MO20240101\t,,\n",
        "2024-01-16 09:00:00,交通出行,滴滴出行,chu***@didichuxing.com,快车费,支出,28.50,蚂蚁宝藏信用卡,交易成功,2024011622001470000002\t,MO20240102\t,,\n",
    )
    .as_bytes();

    async fn setup_with_member() -> Arc<AppState> {
        let state = setup().await;
        state
            .db()
            .member_get_or_create_by_name("测试用户", "zh-CN")
            .await
            .unwrap();
        state
    }

    async fn alipay_channel_id(state: &Arc<AppState>) -> i64 {
        state
            .db()
            .channel_get_by_name("alipay")
            .await
            .unwrap()
            .unwrap()
            .id
            .0
    }

    /// "测试用户" 的成员 id（setup_with_member 已创建）
    async fn test_member_id(state: &Arc<AppState>) -> i64 {
        state
            .db()
            .member_get_or_create_by_name("测试用户", "zh-CN")
            .await
            .unwrap()
            .0
    }

    #[tokio::test]
    async fn import_bill_success() {
        let state = setup_with_member().await;
        let channel_id = alipay_channel_id(&state).await;
        let member_id = test_member_id(&state).await;

        let Json(result) = import_bill(
            State(state),
            Lang("zh-CN".to_string()),
            Path(channel_id),
            Query(ImportBillQuery { member_id }),
            Bytes::from_static(ALIPAY_CSV),
        )
        .await
        .unwrap();

        assert_eq!(result.imported, 2);
        assert_eq!(result.skipped, 0);
        assert!(result.errors.is_empty());
        assert_eq!(result.pending_tag_name.as_deref(), Some("pending"));
    }

    #[tokio::test]
    async fn import_bill_no_adapter_returns_400() {
        let state = setup_with_member().await;
        let member_id = test_member_id(&state).await;
        let channel_id = state
            .db()
            .channel_upsert_by_name("云闪付", None, None, "zh-CN")
            .await
            .unwrap()
            .0;

        let err = import_bill(
            State(state),
            Lang("zh-CN".to_string()),
            Path(channel_id),
            Query(ImportBillQuery { member_id }),
            Bytes::from_static(ALIPAY_CSV),
        )
        .await
        .unwrap_err();

        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn import_bill_channel_not_found() {
        let state = setup_with_member().await;
        let member_id = test_member_id(&state).await;

        let err = import_bill(
            State(state),
            Lang("zh-CN".to_string()),
            Path(9999),
            Query(ImportBillQuery { member_id }),
            Bytes::from_static(ALIPAY_CSV),
        )
        .await
        .unwrap_err();

        assert_eq!(err.0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn import_bill_member_not_found_returns_400() {
        let state = setup_with_member().await;
        let channel_id = alipay_channel_id(&state).await;

        let err = import_bill(
            State(state),
            Lang("zh-CN".to_string()),
            Path(channel_id),
            Query(ImportBillQuery { member_id: 9999 }),
            Bytes::from_static(ALIPAY_CSV),
        )
        .await
        .unwrap_err();

        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn import_bill_works_after_display_name_renamed() {
        let state = setup_with_member().await;
        let channel_id = alipay_channel_id(&state).await;
        let member_id = test_member_id(&state).await;
        // 用户把中文显示名改为非适配器名
        state
            .db()
            .channel_rename(ChannelId(channel_id), "我的支付宝", "zh-CN")
            .await
            .unwrap();

        let Json(result) = import_bill(
            State(state),
            Lang("zh-CN".to_string()),
            Path(channel_id),
            Query(ImportBillQuery { member_id }),
            Bytes::from_static(ALIPAY_CSV),
        )
        .await
        .unwrap();

        assert_eq!(result.imported, 2);
    }

    #[tokio::test]
    async fn import_bill_uses_explicit_member() {
        let state = setup_with_member().await;
        let member_b = state
            .db()
            .member_get_or_create_by_name("成员B", "zh-CN")
            .await
            .unwrap();
        let channel_id = alipay_channel_id(&state).await;

        let _ = import_bill(
            State(state.clone()),
            Lang("zh-CN".to_string()),
            Path(channel_id),
            Query(ImportBillQuery {
                member_id: member_b.0,
            }),
            Bytes::from_static(ALIPAY_CSV),
        )
        .await
        .unwrap();

        let txs = state
            .db()
            .transaction_list(
                &accounting::transaction_filter::TransactionFilter::default(),
                100,
                0,
            )
            .await
            .unwrap();
        assert_eq!(txs.len(), 2);
        assert!(txs.iter().all(|t| t.member_id == member_b));
    }

    #[tokio::test]
    async fn import_bill_parse_failure_returns_400() {
        let state = setup_with_member().await;
        let channel_id = alipay_channel_id(&state).await;
        let member_id = test_member_id(&state).await;

        let err = import_bill(
            State(state),
            Lang("zh-CN".to_string()),
            Path(channel_id),
            Query(ImportBillQuery { member_id }),
            Bytes::from_static(b"this is not a bill file"),
        )
        .await
        .unwrap_err();

        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }
}

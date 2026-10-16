//! budget / saving_plan handler 共享的请求解析与错误响应辅助

use rust_i18n::t;
use serde::Serialize;

/// API 错误响应体
#[derive(Serialize)]
pub(crate) struct ApiError {
    pub(crate) error: String,
}

/// 状态类端点共用的查询参数（`?date=YYYY-MM-DD`，缺省当天）
#[derive(serde::Deserialize)]
pub(crate) struct StatusDateQuery {
    pub date: Option<String>,
}

/// 解析状态查询的 date 参数（缺省当天；格式无效返回本地化错误信息）
pub(crate) fn parse_status_date(
    date: Option<&str>,
    lang: &str,
) -> Result<chrono::NaiveDate, String> {
    match date {
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|e| t!("err_invalid_date", locale = lang, error = e).to_string()),
        None => Ok(chrono::Local::now().date_naive()),
    }
}

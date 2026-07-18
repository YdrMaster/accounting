//! API handlers 模块入口

pub mod account;
pub mod budget;
pub mod channel;
pub mod commodity;
pub mod me;
pub mod member;
pub mod report;
pub mod tag;
pub mod transaction;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::convert::Infallible;

/// 请求语言：取自 query 参数 `?lang=`（如 `?lang=zh-CN`），缺省 `en`。
///
/// 所有实体显示名解析与 rust_i18n 文案均以每次请求的该参数为准，
/// 不依赖进程级 `rust_i18n::set_locale`。
pub struct Lang(pub String);

impl<S> FromRequestParts<S> for Lang
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let lang = parts
            .uri
            .query()
            .and_then(|q| q.split('&').find_map(|pair| pair.strip_prefix("lang=")))
            .filter(|s| !s.is_empty())
            .unwrap_or("en")
            .to_string();
        Ok(Lang(lang))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn extract(uri: &str) -> String {
        let req = axum::http::Request::builder().uri(uri).body(()).unwrap();
        let (mut parts, _) = req.into_parts();
        Lang::from_request_parts(&mut parts, &()).await.unwrap().0
    }

    #[tokio::test]
    async fn lang_defaults_to_en() {
        assert_eq!(extract("/api/members").await, "en");
        assert_eq!(extract("/api/members?limit=10").await, "en");
        assert_eq!(extract("/api/members?lang=").await, "en");
    }

    #[tokio::test]
    async fn lang_from_query() {
        assert_eq!(extract("/api/members?lang=zh-CN").await, "zh-CN");
        assert_eq!(
            extract("/api/members?limit=10&lang=zh-CN&offset=0").await,
            "zh-CN"
        );
    }
}

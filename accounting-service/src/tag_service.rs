use accounting::error::AccountingError;
use accounting::id::TagId;
use accounting::tag::Tag;
use accounting_sql::SqliteDatabase;
use rust_i18n::t;

/// 内置标签描述的本地化文案：按系统英文名映射到翻译键，未知名返回 `None`。
///
/// 数据库 `tags.description` 只存英文原文，系统标签的展示文案按请求语言从此处取译。
pub fn system_tag_description(en_name: &str, lang: &str) -> Option<String> {
    let desc = match en_name {
        "repayment" => t!("system_tag_desc_repayment", locale = lang),
        "pending" => t!("system_tag_desc_pending", locale = lang),
        "exclude-from-income-statement" => {
            t!(
                "system_tag_desc_exclude_from_income_statement",
                locale = lang
            )
        }
        "exclude-from-budget" => t!("system_tag_desc_exclude_from_budget", locale = lang),
        _ => return None,
    };
    Some(desc.to_string())
}

/// 标签服务
pub struct TagService {
    db: SqliteDatabase,
}

impl TagService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 列出所有标签
    pub async fn list(&self) -> Result<Vec<Tag>, AccountingError> {
        self.db
            .tag_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 添加标签
    ///
    /// 名字按 `lang` 语言写入名字表；同名标签已存在时返回既有标签 ID。
    pub async fn add(
        &self,
        name: String,
        description: Option<String>,
        lang: &str,
    ) -> Result<TagId, AccountingError> {
        self.db
            .tag_upsert_by_name(&name, description.as_deref(), lang)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }

    /// 删除标签
    pub async fn delete(&self, name: &str) -> Result<(), AccountingError> {
        self.db
            .tag_delete(name)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    #[test]
    fn test_system_tag_description_bilingual() {
        // 已知系统标签按语言返回对应文案
        assert_eq!(
            system_tag_description("repayment", "en"),
            Some("Installment or credit card repayment marker".to_string())
        );
        assert_eq!(
            system_tag_description("repayment", "zh-CN"),
            Some("分期或信用卡还款标记".to_string())
        );
        assert_eq!(
            system_tag_description("exclude-from-income-statement", "zh-CN"),
            Some("不计入收支统计".to_string())
        );
        assert_eq!(
            system_tag_description("exclude-from-budget", "zh-CN"),
            Some("不计入预算统计".to_string())
        );
        assert_eq!(
            system_tag_description("pending", "zh-CN"),
            Some("导入的交易待确认".to_string())
        );
        // 未知标签名返回 None
        assert_eq!(system_tag_description("travel", "zh-CN"), None);
    }

    #[tokio::test]
    async fn test_tag_lifecycle() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        let service = TagService::new(db);

        let id = service
            .add("travel".to_string(), Some("旅行".to_string()), "en")
            .await
            .unwrap();
        assert!(id.0 > 0);

        let list = service.list().await.unwrap();
        assert!(list.iter().any(|t| t.id == id));
        assert!(
            service
                .db
                .tag_get_by_name("travel")
                .await
                .unwrap()
                .is_some()
        );

        service.delete("travel").await.unwrap();
        let list = service.list().await.unwrap();
        assert!(!list.iter().any(|t| t.id == id));
        assert!(
            service
                .db
                .tag_get_by_name("travel")
                .await
                .unwrap()
                .is_none()
        );
    }
}

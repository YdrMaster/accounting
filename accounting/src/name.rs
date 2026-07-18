//! 名字按语言管理：语言标签规范
//!
//! 名字数据存于各实体的名字表（见 accounting-sql 的 names 模块），
//! 本模块只提供语言标签常量与归一化规则。

/// 语言标签常量
pub mod lang {
    /// 英文（默认回退语言）
    pub const EN: &str = "en";
    /// 中文（`zh-*` 归一为此标签）
    pub const ZH_CN: &str = "zh-CN";
    /// 未确定语言（如导入账单自动创建的实体名）
    pub const UND: &str = "und";

    /// 将语言标签归一化：
    /// - `zh-*` → `zh-CN`
    /// - 其余原样返回
    pub fn normalize(lang: &str) -> &str {
        if lang.starts_with("zh") { ZH_CN } else { lang }
    }
}

#[cfg(test)]
mod tests {
    use super::lang;

    #[test]
    fn test_normalize_zh_variants() {
        assert_eq!(lang::normalize("zh-CN"), "zh-CN");
        assert_eq!(lang::normalize("zh"), "zh-CN");
        assert_eq!(lang::normalize("zh-TW"), "zh-CN");
    }

    #[test]
    fn test_normalize_other_langs_unchanged() {
        assert_eq!(lang::normalize("en"), "en");
        assert_eq!(lang::normalize("und"), "und");
        assert_eq!(lang::normalize("ja"), "ja");
    }
}

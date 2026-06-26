use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 配置导出/导入文件顶层结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigFile {
    pub version: String,
    pub settings: Option<Settings>,
    pub commodities: Vec<YamlCommodity>,
    pub members: Vec<YamlMember>,
    pub channels: Vec<YamlChannel>,
    pub tags: Vec<YamlTag>,
    pub accounts: Vec<YamlAccount>,
    pub account_owners: Vec<YamlAccountOwner>,
    pub account_mappings: Vec<YamlAccountMapping>,
    pub budgets: Vec<YamlBudget>,
}

impl ConfigFile {
    pub fn current_version() -> &'static str {
        "1.0"
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            version: Self::current_version().to_string(),
            settings: None,
            commodities: Vec::new(),
            members: Vec::new(),
            channels: Vec::new(),
            tags: Vec::new(),
            accounts: Vec::new(),
            account_owners: Vec::new(),
            account_mappings: Vec::new(),
            budgets: Vec::new(),
        }
    }
}

/// 应用设置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub language: String,
}

/// 币种 / 商品
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YamlCommodity {
    pub symbol: String,
    pub name: String,
    pub precision: u8,
}

/// 成员
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YamlMember {
    pub name: String,
}

/// 支付渠道
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YamlChannel {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
}

/// 标签
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YamlTag {
    pub name: String,
    pub description: Option<String>,
}

/// 账户
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YamlAccount {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_day: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repayment_day: Option<u8>,
}

/// 账户持有人关系
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YamlAccountOwner {
    pub account: String,
    pub members: Vec<String>,
}

/// 导入映射规则
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YamlAccountMapping {
    pub member: String,
    pub channel: String,
    /// 分类字符串 -> 目标账户路径
    pub mappings: BTreeMap<String, String>,
}

/// 预算
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YamlBudget {
    pub name: String,
    pub period: String,
    pub commodity: String,
    /// 账户路径 -> 限额字符串
    pub limits: BTreeMap<String, String>,
}

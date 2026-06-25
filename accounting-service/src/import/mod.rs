use accounting::id::{ChannelId, CommodityId, MemberId};
use accounting::transaction::TransactionKind;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use std::error::Error;
use std::fmt;

/// 账单适配器 trait — 定义统一的账单解析接口
pub trait BillAdapter {
    /// 适配器名称列表（如 &["alipay", "支付宝"]），任一匹配即可
    fn names(&self) -> &[&str];

    /// 解析原始文件字节，返回 BillEntry 迭代器
    fn parse<'a>(
        &'a self,
        data: &[u8],
        ctx: &ImportContext,
    ) -> Result<Box<dyn Iterator<Item = Result<BillEntry, AdaptError>> + 'a>, AdaptError>;
}

/// 导入上下文 — 为适配器提供运行时补充信息
#[derive(Debug, Clone)]
pub struct ImportContext {
    pub member_id: MemberId,
    pub channel_id: ChannelId,
    pub commodity_id: CommodityId,
    /// 导入根账户名称（如 "Import" 或 "导入"）
    pub import_root: String,
}

/// 适配器输出的标准账目条目
#[derive(Debug, Clone)]
pub struct BillEntry {
    pub date_time: NaiveDateTime,
    pub description: String,
    pub kind: TransactionKind,
    pub postings: Vec<BillPosting>,
    pub tags: Vec<String>,
    /// 源文件行号（用于错误报告）
    pub row: Option<usize>,
}

/// 账目条目中的单个分录
#[derive(Debug, Clone)]
pub struct BillPosting {
    /// 账户路径（如 "Import:支付宝:餐饮美食"）
    pub account_path: String,
    /// 商品符号（如 "CNY"）
    pub commodity_symbol: String,
    /// 金额（支出为负，收入为正）
    pub amount: Decimal,
    /// 是否可报销
    pub is_reimbursable: bool,
}

/// 适配器解析错误
#[derive(Debug, Clone)]
pub enum AdaptError {
    /// 单行解析错误
    RowError { row: usize, message: String },
    /// 文件格式错误
    FormatError(String),
}

impl fmt::Display for AdaptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdaptError::RowError { row, message } => {
                write!(f, "第 {row} 行：{message}")
            }
            AdaptError::FormatError(msg) => write!(f, "格式错误：{msg}"),
        }
    }
}

impl Error for AdaptError {}

/// 返回所有内置适配器列表
pub fn builtin_adapters() -> Vec<Box<dyn BillAdapter>> {
    vec![Box::new(alipay::AlipayAdapter)]
}

/// 根据名称查找适配器
pub fn find_adapter<'a>(
    name: &str,
    adapters: &'a [Box<dyn BillAdapter>],
) -> Option<&'a dyn BillAdapter> {
    adapters
        .iter()
        .find(|a| a.names().iter().any(|n| n.eq_ignore_ascii_case(name)))
        .map(|a| a.as_ref())
}

pub mod alipay;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_adapter_by_name() {
        let adapters = builtin_adapters();
        let found = find_adapter("alipay", &adapters);
        assert!(found.is_some());
        assert_eq!(found.unwrap().names(), &["alipay", "支付宝"]);

        let found_alias = find_adapter("支付宝", &adapters);
        assert!(found_alias.is_some());
        assert_eq!(found_alias.unwrap().names(), &["alipay", "支付宝"]);

        let missing = find_adapter("unknown", &adapters);
        assert!(missing.is_none());
    }
}

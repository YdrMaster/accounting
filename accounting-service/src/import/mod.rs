use accounting::id::{ChannelId, CommodityId, MemberId};
use accounting::posting_role::PostingRole;
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
    /// 渠道名称（如 "支付宝"）
    pub channel_name: String,
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
    /// 角色（收支侧 or 资产侧）
    pub role: PostingRole,
    /// 分类名称（如 "餐饮美食"、"蚂蚁宝藏信用卡"）
    pub category: String,
    /// 商品符号（如 "CNY"）
    pub commodity_symbol: String,
    /// 金额（收支侧：支出为正、收入为负；资产侧：与收支侧相反）
    pub amount: Decimal,
    /// 是否可报销
    pub is_reimbursable: bool,
}

/// 适配器解析错误
#[derive(Debug, Clone)]
pub enum AdaptError {
    /// 文件编码错误
    Encoding { source: String },
    /// 单行解析错误
    Row { row: usize, detail: RowErrorDetail },
}

/// 单行解析错误的具体原因
#[derive(Debug, Clone)]
pub enum RowErrorDetail {
    /// 缺少指定列
    MissingColumn { index: usize, name: String },
    /// 金额解析失败
    AmountParse { value: String, source: String },
    /// 日期解析失败
    DateParse { value: String },
    /// 交易已关闭
    ClosedTransaction,
    /// 上层 service 包装的其他错误（message 已由源 crate 本地化）
    Other { message: String },
}

impl fmt::Display for RowErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RowErrorDetail::MissingColumn { index, name } => {
                write!(f, "missing column at index {index} ({name})")
            }
            RowErrorDetail::AmountParse { value, source } => {
                write!(f, "amount parse failed for '{value}': {source}")
            }
            RowErrorDetail::DateParse { value } => {
                write!(f, "date parse failed for '{value}'")
            }
            RowErrorDetail::ClosedTransaction => write!(f, "transaction closed"),
            RowErrorDetail::Other { message } => write!(f, "{message}"),
        }
    }
}

impl fmt::Display for AdaptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdaptError::Encoding { source } => write!(f, "encoding error: {source}"),
            AdaptError::Row { row, detail } => write!(f, "row {row}: {detail}"),
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

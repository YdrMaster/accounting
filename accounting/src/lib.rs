//! 核心库：记账数据模型与算法

/// 账户模型
pub mod account;
/// 账户类型与分期方式
pub mod account_type;
/// 金额转换工具
pub mod amount;
/// 附件模型
pub mod attachment;
/// 余额计算
pub mod balance;
/// 支付渠道模型
pub mod channel;
/// 闭包表计算
pub mod closure;
/// 商品/货币模型
pub mod commodity;
/// 错误类型
pub mod error;
/// ID 类型定义
pub mod id;
/// 分期工具
pub mod installment;
/// 成员模型
pub mod member;
/// 分录模型
pub mod posting;
pub use transaction::TransactionKind;
/// 日期时间工具函数
pub mod datetime_utils;
/// 标签模型
pub mod tag;
/// 交易模型
pub mod transaction;
/// 交易查询条件
pub mod transaction_filter;
/// 交易验证规则
pub mod validation;

rust_i18n::i18n!("locales", fallback = "en");

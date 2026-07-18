//! 核心库：记账数据模型与算法

/// 账户模型
pub mod account;
/// 账户映射模型
pub mod account_mapping;
/// 账户类型与分期方式
pub mod account_type;
/// 金额转换工具
pub mod amount;
/// 附件模型
pub mod attachment;
/// 余额计算
pub mod balance;
/// 预算模型
pub mod budget;
/// 支付渠道模型
pub mod channel;
/// 交易链路模型
pub mod channel_path;
/// 闭包表计算
pub mod closure;
/// 商品/货币模型
pub mod commodity;
/// 错误类型
pub mod error;
/// 财务周期
pub mod finance_period;
/// ID 类型定义
pub mod id;
/// 分期工具
pub mod installment;
/// 成员模型
pub mod member;
/// 名字按语言管理
pub mod name;
/// 分录模型
pub mod posting;
/// 分录角色（收支侧/资产侧）
pub mod posting_role;
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

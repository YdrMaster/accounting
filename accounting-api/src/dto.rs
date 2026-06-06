//! 请求/响应 DTO

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// 通用错误响应。
#[derive(Serialize)]
pub struct ErrorResponse {
    /// 错误信息。
    pub error: String,
}

/// 成员响应。
#[derive(Serialize)]
pub struct MemberDto {
    /// 成员 ID。
    pub id: i64,
    /// 成员名称。
    pub name: String,
}

/// 账户响应。
#[derive(Serialize)]
pub struct AccountDto {
    /// 账户 ID。
    pub id: i64,
    /// 完整名称。
    pub full_name: String,
    /// 账户类型。
    pub account_type: String,
    /// 父账户 ID。
    pub parent_id: Option<i64>,
    /// 关闭日期（ISO 8601 格式）。
    pub closed_at: Option<String>,
    /// 是否为系统账户。
    pub is_system: bool,
    /// 账单日。
    pub billing_day: Option<u8>,
    /// 还款日。
    pub repayment_day: Option<u8>,
}

/// 创建账户请求。
#[derive(Deserialize)]
pub struct CreateAccountRequest {
    /// 完整名称。
    pub full_name: String,
    /// 账单日。
    pub billing_day: Option<u8>,
    /// 还款日。
    pub repayment_day: Option<u8>,
}

/// 交易响应。
#[derive(Serialize)]
pub struct TransactionDto {
    /// 交易 ID。
    pub id: i64,
    /// 交易日期时间（ISO 8601 格式）。
    pub date_time: String,
    /// 交易描述。
    pub description: String,
    /// 成员 ID。
    pub member_id: Option<i64>,
    /// 是否为模板。
    pub is_template: bool,
}

/// 创建交易请求。
#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    /// 交易日期时间（ISO 8601 格式）。
    pub date_time: String,
    /// 交易描述。
    pub description: String,
    /// 成员 ID。
    pub member_id: Option<i64>,
    /// 分录列表。
    pub postings: Vec<PostingRequest>,
    /// 标签列表。
    pub tags: Vec<String>,
}

/// 分录请求。
#[derive(Deserialize)]
pub struct PostingRequest {
    /// 账户名称。
    pub account: String,
    /// 货币符号。
    pub commodity: String,
    /// 金额字符串。
    pub amount: String,
}

/// 标签响应。
#[derive(Serialize)]
pub struct TagDto {
    /// 标签 ID。
    pub id: i64,
    /// 标签名称。
    pub name: String,
    /// 标签描述。
    pub description: Option<String>,
    /// 是否为系统标签。
    pub is_system: bool,
}

/// 货币响应。
#[derive(Serialize)]
pub struct CommodityDto {
    /// 货币 ID。
    pub id: i64,
    /// 货币符号。
    pub symbol: String,
    /// 货币名称。
    pub name: String,
    /// 精度（小数位数）。
    pub precision: u8,
}

/// 当前用户响应。
#[derive(Serialize)]
pub struct MeDto {
    /// 当前成员 ID。
    pub member_id: i64,
    /// 当前成员名称。
    pub member_name: String,
}

/// 切换当前用户请求。
#[derive(Deserialize)]
pub struct SetMeRequest {
    /// 目标成员 ID。
    pub member_id: i64,
}

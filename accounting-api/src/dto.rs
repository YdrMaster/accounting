//! 请求/响应 DTO

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
    /// 排序位置。
    pub position: i64,
    /// 所有者成员 ID 列表。
    pub owner_ids: Vec<i64>,
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
    /// 所有者成员 ID 列表。
    pub owner_ids: Vec<i64>,
}

/// 设置账户所有者请求。
#[derive(Deserialize)]
pub struct SetAccountOwnersRequest {
    /// 所有者成员 ID 列表。
    pub owner_ids: Vec<i64>,
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
    /// 交易类型。
    pub kind: String,
    /// 成员 ID。
    pub member_id: Option<i64>,
    /// 渠道 ID。
    pub channel_id: Option<i64>,
    /// 是否为模板。
    pub is_template: bool,
    /// 分录列表。
    pub postings: Vec<PostingDto>,
}

/// 分录响应。
#[derive(Serialize)]
pub struct PostingDto {
    /// 分录 ID。
    pub id: i64,
    /// 所属交易 ID。
    pub transaction_id: i64,
    /// 账户名称。
    pub account: String,
    /// 货币符号。
    pub commodity: String,
    /// 金额字符串。
    pub amount: String,
    /// 可报销标记。
    pub is_reimbursable: bool,
    /// 关联分录 ID。
    pub linked_posting_id: Option<i64>,
    /// 已冲正总额。
    pub reversal_total: String,
}

/// 创建交易请求。
#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    /// 交易日期时间（ISO 8601 格式）。
    pub date_time: String,
    /// 交易描述。
    pub description: String,
    /// 交易类型。
    #[serde(default = "default_kind")]
    pub kind: String,
    /// 成员 ID。
    pub member_id: Option<i64>,
    /// 渠道 ID。
    pub channel_id: Option<i64>,
    /// 分录列表。
    pub postings: Vec<PostingRequest>,
    /// 标签列表。
    pub tags: Vec<String>,
}

fn default_kind() -> String {
    "normal".to_string()
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
    /// 可报销标记。
    #[serde(default)]
    pub is_reimbursable: bool,
    /// 关联分录 ID。
    pub linked_posting_id: Option<i64>,
}

/// 渠道响应。
#[derive(Serialize)]
pub struct ChannelDto {
    /// 渠道 ID。
    pub id: i64,
    /// 渠道名称。
    pub name: String,
    /// 渠道描述。
    pub description: Option<String>,
}

/// 创建渠道请求。
#[derive(Deserialize)]
pub struct CreateChannelRequest {
    /// 渠道名称。
    pub name: String,
    /// 渠道描述。
    pub description: Option<String>,
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

/// 重命名账户请求。
#[derive(Deserialize)]
pub struct RenameAccountRequest {
    /// 新完整名称。
    pub full_name: String,
}

/// 账户排序请求。
#[derive(Deserialize)]
pub struct ReorderRequest {
    /// 账户 ID 列表（按新顺序排列）。
    pub ids: Vec<i64>,
}

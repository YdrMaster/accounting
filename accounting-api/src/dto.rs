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
    /// 账户名（本级名称）。
    pub name: String,
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
    /// 所有者成员 ID 列表。
    pub owner_ids: Vec<i64>,
}

/// 创建账户请求。
#[derive(Deserialize)]
pub struct CreateAccountRequest {
    /// 账户名（本级名称）。
    pub name: String,
    /// 父账户 ID（根账户不传）。
    pub parent_id: Option<i64>,
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

/// 链路节点响应。
#[derive(Serialize)]
pub struct ChannelPathNodeDto {
    /// 在链路中的位置（从 0 开始）。
    pub position: i32,
    /// 渠道 ID。
    pub channel_id: i64,
    /// 渠道名称。
    pub channel_name: String,
    /// 链路节点状态。
    pub status: String,
}

/// 链路节点请求。
#[derive(Deserialize, Clone)]
pub struct ChannelPathNodeRequest {
    /// 在链路中的位置（从 0 开始）。
    pub position: i32,
    /// 渠道 ID。
    pub channel_id: i64,
    /// 链路节点状态（可选，默认 default）。
    #[serde(default = "default_channel_path_status")]
    pub status: String,
}

fn default_channel_path_status() -> String {
    "default".to_string()
}

/// 对账标记请求。
#[derive(Deserialize)]
pub struct ReconcileRequest {
    /// 是否取消已校验标记。
    #[serde(default)]
    pub unset: bool,
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
    pub member_id: i64,
    /// 成员名称。
    pub member_name: String,
    /// 标签名称列表。
    pub tags: Vec<String>,
    /// 渠道链路节点列表。
    pub channel_paths: Vec<ChannelPathNodeDto>,
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
    /// 账户类型。
    pub account_type: String,
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
    pub member_id: i64,
    /// 渠道链路节点列表。
    #[serde(default)]
    pub channel_paths: Vec<ChannelPathNodeRequest>,
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
    /// 关联资产账户 ID。
    pub account_id: Option<i64>,
    /// 是否为系统内置渠道。
    pub is_system: bool,
}

/// 创建渠道请求。
#[derive(Deserialize)]
pub struct CreateChannelRequest {
    /// 渠道名称。
    pub name: String,
    /// 渠道描述。
    pub description: Option<String>,
    /// 关联资产账户 ID。
    pub account_id: Option<i64>,
}

/// 更新渠道请求。
#[derive(Deserialize)]
pub struct UpdateChannelRequest {
    /// 关联资产账户 ID。
    pub account_id: Option<i64>,
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
    /// 新名称（本级名称）。
    pub name: String,
}

/// 更新账户字段请求。
#[derive(Deserialize)]
pub struct UpdateAccountRequest {
    /// 账单日。
    pub billing_day: Option<u8>,
    /// 还款日。
    pub repayment_day: Option<u8>,
}

// ─── 预算 DTO ───

/// 预算表响应。
#[derive(Serialize)]
pub struct BudgetDto {
    /// 预算表 ID。
    pub id: i64,
    /// 预算表名称。
    pub name: String,
    /// 周期类型。
    pub period: String,
    /// 币种 ID。
    pub commodity_id: i64,
}

/// 预算限额响应。
#[derive(Serialize)]
pub struct BudgetLimitDto {
    /// 账户 ID。
    pub account_id: i64,
    /// 限额金额。
    pub amount: String,
}

/// 预算表详情响应（含限额列表）。
#[derive(Serialize)]
pub struct BudgetDetailDto {
    /// 预算表信息。
    pub budget: BudgetDto,
    /// 限额列表。
    pub limits: Vec<BudgetLimitDto>,
}

/// 预算执行情况响应。
#[derive(Serialize)]
pub struct BudgetStatusDto {
    /// 预算表信息。
    pub budget: BudgetDto,
    /// 周期起始日期。
    pub period_start: String,
    /// 周期结束日期。
    pub period_end: String,
    /// 各账户执行情况。
    pub items: Vec<BudgetItemStatusDto>,
}

/// 单个账户的预算执行情况。
#[derive(Serialize)]
pub struct BudgetItemStatusDto {
    /// 账户 ID。
    pub account_id: i64,
    /// 限额金额。
    pub limit_amount: String,
    /// 实际金额。
    pub actual_amount: String,
    /// 剩余金额（正=剩余，负=超支）。
    pub remaining: String,
    /// 执行百分比。
    pub percentage: String,
}

/// 创建/更新预算限额请求。
#[derive(Deserialize)]
pub struct BudgetLimitRequest {
    /// 账户 ID。
    pub account_id: i64,
    /// 限额金额。
    pub amount: String,
}

/// 创建预算表请求。
#[derive(Deserialize)]
pub struct CreateBudgetRequest {
    /// 预算表名称。
    pub name: String,
    /// 周期类型。
    pub period: String,
    /// 币种 ID。
    pub commodity_id: i64,
    /// 限额列表。
    pub limits: Vec<BudgetLimitRequest>,
}

/// 更新预算表请求。
#[derive(Deserialize)]
pub struct UpdateBudgetRequest {
    /// 预算表名称。
    pub name: String,
    /// 周期类型。
    pub period: String,
    /// 币种 ID。
    pub commodity_id: i64,
    /// 限额列表。
    pub limits: Vec<BudgetLimitRequest>,
}

/// 解析周期字符串为 FinancePeriod。
pub fn parse_period(s: &str) -> Result<accounting::finance_period::FinancePeriod, String> {
    use accounting::finance_period::FinancePeriod;
    match s.to_lowercase().as_str() {
        "daily" => Ok(FinancePeriod::Daily),
        "weekly-sun" => Ok(FinancePeriod::WeeklyFromSunday),
        "weekly-mon" => Ok(FinancePeriod::WeeklyFromMonday),
        "monthly" => Ok(FinancePeriod::Monthly),
        "yearly" => Ok(FinancePeriod::Yearly),
        _ => Err(format!("无效周期类型: {}", s)),
    }
}

/// 将 FinancePeriod 转为 API 字符串。
pub fn to_period_string(period: accounting::finance_period::FinancePeriod) -> &'static str {
    use accounting::finance_period::FinancePeriod;
    match period {
        FinancePeriod::Daily => "daily",
        FinancePeriod::WeeklyFromSunday => "weekly-sun",
        FinancePeriod::WeeklyFromMonday => "weekly-mon",
        FinancePeriod::Monthly => "monthly",
        FinancePeriod::Yearly => "yearly",
    }
}

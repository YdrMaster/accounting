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

/// 变更账户父节点请求。
#[derive(Deserialize)]
pub struct SetAccountParentRequest {
    /// 新父账户 ID（必填，不允许移动为根账户）。
    pub parent_id: i64,
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
    /// 账户 ID。
    pub account_id: i64,
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
    /// 是否关联了内置账单导入适配器。
    pub has_import_adapter: bool,
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
    /// 渠道名称（None=不修改，Some(None)=清空，Some(Some(v))=设为v）。
    pub name: Option<Option<String>>,
    /// 渠道描述（None=不修改，Some(None)=清空，Some(Some(v))=设为v）。
    pub description: Option<Option<String>>,
    /// 关联资产账户 ID（None=不修改，Some(None)=取消关联，Some(Some(v))=关联v）。
    pub account_id: Option<Option<i64>>,
}

/// 账单导入结果。
#[derive(Debug, Serialize)]
pub struct ImportResultDto {
    /// 成功导入条数。
    pub imported: usize,
    /// 跳过条数。
    pub skipped: usize,
    /// 导入交易挂载的待处理标签系统名（无系统标签时为 None）。
    pub pending_tag_name: Option<String>,
    /// 逐行错误明细。
    pub errors: Vec<ImportRowErrorDto>,
}

/// 账单导入的逐行错误。
#[derive(Debug, Serialize)]
pub struct ImportRowErrorDto {
    /// 源文件行号。
    pub row: usize,
    /// 人类可读的错误描述。
    pub detail: String,
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
    /// 周期类型（一次性预算为 null）。
    pub period: Option<String>,
    /// 截止日期（"YYYY-MM-DD"，无截止为 null）。
    pub deadline: Option<String>,
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
    /// 是否已失效（查询日晚于 deadline）。
    pub expired: bool,
    /// 周期起始日期（一次性预算为 null）。
    pub period_start: Option<String>,
    /// 周期结束日期（一次性预算为 null）。
    pub period_end: Option<String>,
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
    /// 周期类型（缺省/null 表示一次性预算）。
    pub period: Option<String>,
    /// 截止日期（"YYYY-MM-DD"，可为 null）。
    pub deadline: Option<String>,
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
    /// 周期类型（可为 null）。
    pub period: Option<String>,
    /// 截止日期（"YYYY-MM-DD"，可为 null）。
    pub deadline: Option<String>,
    /// 币种 ID。
    pub commodity_id: i64,
    /// 限额列表。
    pub limits: Vec<BudgetLimitRequest>,
}

/// 账户映射响应。
#[derive(Serialize)]
pub struct MappingDto {
    /// 成员 ID。
    pub member_id: i64,
    /// 渠道 ID。
    pub channel_id: i64,
    /// 映射 key（分类字符串）。
    pub category: String,
    /// 目标账户 ID。
    pub account_id: i64,
}

/// 设置账户映射请求。
#[derive(Deserialize)]
pub struct SetMappingRequest {
    /// 成员 ID。
    pub member_id: i64,
    /// 渠道 ID。
    pub channel_id: i64,
    /// 映射 key（分类字符串）。
    pub category: String,
    /// 目标账户 ID。
    pub account_id: i64,
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

/// 解析可选周期字符串（None 或空字符串 → None，表示一次性）。
pub fn parse_period_opt(
    s: Option<&str>,
) -> Result<Option<accounting::finance_period::FinancePeriod>, String> {
    match s {
        Some(s) if !s.is_empty() => parse_period(s).map(Some),
        _ => Ok(None),
    }
}

/// 将可选 FinancePeriod 转为 API 字符串（None → None，序列化为 JSON null）。
pub fn period_to_string(
    period: Option<accounting::finance_period::FinancePeriod>,
) -> Option<String> {
    period.map(|p| to_period_string(p).to_string())
}

/// 解析可选截止日期字符串（None 或空字符串 → None）。
pub fn parse_deadline(s: Option<&str>) -> Result<Option<chrono::NaiveDate>, String> {
    match s {
        Some(s) if !s.is_empty() => chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(Some)
            .map_err(|e| format!("无效日期: {}", e)),
        _ => Ok(None),
    }
}

// ─── 攒钱计划 DTO ───

/// 攒钱计划响应。
#[derive(Serialize)]
pub struct SavingPlanDto {
    /// 攒钱计划 ID。
    pub id: i64,
    /// 攒钱计划名称。
    pub name: String,
    /// 周期类型（一次性计划为 null）。
    pub period: Option<String>,
    /// 截止日期（"YYYY-MM-DD"，无截止为 null）。
    pub deadline: Option<String>,
    /// 币种 ID。
    pub commodity_id: i64,
    /// 目标金额（Decimal 序列化为字符串）。
    pub target_amount: String,
    /// 关联账户 ID 列表。
    pub account_ids: Vec<i64>,
}

/// 攒钱计划详情响应（含账户 ID 列表）。
#[derive(Serialize)]
pub struct SavingPlanDetailDto {
    /// 攒钱计划信息。
    pub plan: SavingPlanDto,
    /// 关联账户 ID 列表。
    pub account_ids: Vec<i64>,
}

/// 攒钱计划状态响应。
#[derive(Serialize)]
pub struct SavingPlanStatusDto {
    /// 攒钱计划信息。
    pub plan: SavingPlanDto,
    /// 是否已失效（查询日晚于 deadline）。
    pub expired: bool,
    /// 周期起始日期（一次性计划为 null）。
    pub period_start: Option<String>,
    /// 周期结束日期（一次性计划为 null）。
    pub period_end: Option<String>,
    /// 目标金额。
    pub target_amount: String,
    /// 账户集合（含后代）截至查询日的余额合计。
    pub current_balance: String,
    /// 缺口（target_amount - current_balance，负值表示已超额）。
    pub gap: String,
    /// 是否达标（current_balance >= target_amount）。
    pub met: bool,
    /// 全局分配口径下本计划分配到的金额（Decimal 序列化为字符串）。
    pub allocated: String,
    /// 满足率（allocated / target_amount * 100，Decimal 序列化为字符串）。
    pub satisfaction: String,
    /// 各账户分配明细。
    pub accounts: Vec<AccountAllocationDto>,
}

/// 攒钱计划单账户分配明细。
#[derive(Serialize)]
pub struct AccountAllocationDto {
    /// 账户 ID。
    pub account_id: i64,
    /// 该账户（含后代）截至查询日的余额。
    pub balance: String,
    /// 被更早检查点的计划占用的金额。
    pub occupied_by_earlier: String,
    /// 本计划分配到的金额。
    pub allocated: String,
}

/// 创建攒钱计划请求。
#[derive(Deserialize)]
pub struct CreateSavingPlanRequest {
    /// 攒钱计划名称。
    pub name: String,
    /// 周期类型（缺省/null 表示一次性计划）。
    pub period: Option<String>,
    /// 截止日期（"YYYY-MM-DD"，可为 null）。
    pub deadline: Option<String>,
    /// 币种 ID。
    pub commodity_id: i64,
    /// 目标金额（Decimal 字符串）。
    pub target_amount: String,
    /// 关联账户 ID 列表。
    pub account_ids: Vec<i64>,
}

/// 更新攒钱计划请求。
#[derive(Deserialize)]
pub struct UpdateSavingPlanRequest {
    /// 攒钱计划名称。
    pub name: String,
    /// 周期类型（可为 null）。
    pub period: Option<String>,
    /// 截止日期（"YYYY-MM-DD"，可为 null）。
    pub deadline: Option<String>,
    /// 币种 ID。
    pub commodity_id: i64,
    /// 目标金额（Decimal 字符串）。
    pub target_amount: String,
    /// 关联账户 ID 列表。
    pub account_ids: Vec<i64>,
}

# Design: saving-plan-allocation

## Context

saving-plan 已全链路落地（archive/2026-08-01-saving-plan）：`SavingPlanService::get_saving_plan_status(plan_id, date)` 单计划独立计算——`current_balance = account_balance_by_ids(账户集合, commodity, date)`（含后代、合计口径），`gap = target − balance`，`met = balance ≥ target`。无计划间协调。

本次要解决共享账户的重复计数问题。关键现状约束：

- `account_balance_by_ids` 返回集合**合计**，分配算法需要**按单个账户**的余额（含各自后代），SQL 层需新增按账户分组的余额查询（或调整现有函数返回分组结果）。
- 计划数量是个人量级（个位数到几十），全局遍历无性能压力，纯计算不落库。

## Goals / Non-Goals

**Goals:**

- 全局分配遍历：同币种有效计划按检查点升序，每个计划只计算一次，顺序占用，欠费占光可用。
- 分配偏好：尽量为「下一个有交集的计划」保留交集账户。
- 状态输出：allocated、satisfaction、每账户明细（余额/被占用/本计划分配）。
- CLI show 显示明细、list 显示满足率；API 向后兼容加字段。

**Non-Goals:**

- 全局最优分配（字典序最大流）——贪心规则已确认接受其边界（见 D3）。
- 预算侧的任何改动。
- 分配结果持久化/历史快照。
- 前端 UI。
- 永久计划与过期计划的显示逻辑变化（保持现状独立计算）。

## Decisions

### D1: 参与规则与检查点定义

参与全局分配的计划：未过期（`date ≤ deadline`，无 deadline 恒未过期）**且**有检查点：

- 一次性计划（period=None, deadline 有）：检查点 = deadline
- 周期计划（period 有）：检查点 = min(当前周期末, deadline)（deadline 可空则取周期末）
- 永久计划（period/deadline 皆空）：**不参与**，状态保持独立计算
- 过期计划：**不参与**，状态保持独立计算（显示 expired）

分配按 `commodity_id` 分组，组内互不影响。排序键 = (检查点, plan_id)（id 做确定性 tie-break）。

### D2: 占用语义——每计划一次、欠费占光

无论周期与否，每个计划只计算一次，占用额 `allocated = min(target, available)`。`available = Σ(账户余额 − occupied[账户])`。欠费（available < target）时占光全部可用，后续共享账户计划只能捡剩——先到先得，无比例分配。

### D3: 账户内分配偏好——保留下一交集计划的账户

计划 i 需要在其账户集合 Sᵢ 上落实 `allocated` 的扣减分布时：

1. 找排序在 i 之后、第一个满足 Sⱼ ∩ Sᵢ ≠ ∅ 的计划 j。
2. 优先从 Sᵢ \ Sⱼ（与 j 无关的账户）扣减，按账户 id 升序逐个取 `min(余额−已占用, 剩余需扣)`。
3. 不够时再从 Sᵢ ∩ Sⱼ 扣减（同序）。
4. 无后续交集计划时，直接按账户 id 升序扣减（分布只影响展示，不影响任何满足率）。

**已知边界（已确认接受）**：只看**下一个**交集计划的贪心不是全局最优（反例：计划1{A,B}、计划2{A,B,C}、计划3{A}，计划1 面对全交集拿不到偏好信息）。选它的理由：可解释（"先保最近的"）、实现简单、个人场景计划少。字典序最大流为过度设计。

### D4: 输出结构

```rust
pub struct SavingPlanAccountAllocation {
    pub account_id: AccountId,
    pub balance: Decimal,               // 该账户（含后代）余额
    pub occupied_by_earlier: Decimal,   // 被更早检查点计划占用
    pub allocated: Decimal,             // 本计划分配到
}

// SavingPlanStatus 增加：
pub allocated: Decimal,        // Σ allocated
pub satisfaction: Decimal,     // allocated / target * 100
pub accounts: Vec<SavingPlanAccountAllocation>,
```

不参与分配的计划（过期/永久）：`allocated`/`satisfaction`/`accounts` 语义退化为现状口径（accounts 明细中 occupied_by_earlier=0、allocated 按独立计算填充或不返回分配语义——实现时以 spec 场景为准，保持简单：参与分配的字段对非参与计划按「无其他计划竞争」的退化结果返回）。

### D5: 计算入口——一次全局计算，多处消费

service 新增内部函数 `compute_allocations(date) -> Vec<SavingPlanStatus>`（按 commodity 分组遍历，直接产出含分配字段的状态）。`get_saving_plan_status(plan_id, date)` 内部调它取对应行；`list_saving_plan_statuses(date)`（供 CLI list 满足率列）复用同一计算。无缓存——计划量级下每次重算足够快。

### D6: SQL 层——按账户分组余额

现有 `account_balance_by_ids` 是合计口径。新增 `account_balances_by_ids(account_ids, commodity_id, as_of) -> Vec<(AccountId, Decimal)>`：每个指定账户各自（含后代）的余额，父子同选时**不去重**（分配以「选中的账户」为粒度，每个选中账户是独立资金池视角）；调用方保证同一账户不被两个计划在同一轮分配中重复当作不同池子（occupied 按 account_id 记账，天然处理）。

注意：若同一计划同时选中父子账户，两池余额重叠（子账户的钱被计两次）——校验层暂不禁绝，按现状语义文档化（预算的 per-account 限额同样不处理此重叠）。

## Risks / Trade-offs

- [贪心非全局最优] → D3 已述，语义可解释性优先，spec 中用场景锁定行为。
- [同一计划选父子账户导致余额重复计数] → D6 文档化；后续如需可在 validate 禁止同计划内选祖先-后代账户。
- [status 从 O(单计划) 变 O(全部计划)] → 计划个位数，无感；且 list 满足率列本来就需要全局计算。
- [satisfaction 对永久/过期计划的退化语义可能造成 API 消费者困惑] → D4 明确退化口径并在 spec 场景锁定。

## Migration Plan

无 schema 变更、无数据迁移：纯计算逻辑 + 响应新增字段（serde 向后兼容）。

## Open Questions

- 无。

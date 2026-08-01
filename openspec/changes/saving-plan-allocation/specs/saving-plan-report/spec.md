# saving-plan-report

## MODIFIED Requirements

### Requirement: 攒钱计划状态数据结构
系统 SHALL 定义以下数据结构：

```rust
pub struct SavingPlanDetail {
    pub plan: SavingPlan,
    pub account_ids: Vec<AccountId>,
}

pub struct SavingPlanAccountAllocation {
    pub account_id: AccountId,
    pub balance: Decimal,               // 该账户（含后代）截至查询日余额
    pub occupied_by_earlier: Decimal,   // 被更早检查点的计划占用的金额
    pub allocated: Decimal,             // 本计划分配到的金额
}

pub struct SavingPlanStatus {
    pub plan: SavingPlan,
    pub expired: bool,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub target_amount: Decimal,
    pub current_balance: Decimal,
    pub gap: Decimal,
    pub met: bool,
    pub allocated: Decimal,             // Σ 各账户 allocated
    pub satisfaction: Decimal,          // allocated / target_amount * 100
    pub accounts: Vec<SavingPlanAccountAllocation>,
}
```

其中 `gap = target_amount − current_balance`，`met = current_balance ≥ target_amount`，`satisfaction = allocated / target_amount * 100`。`met` 仍按独立余额口径判定（反映账面上钱够不够），`satisfaction` 按全局分配口径判定（反映按检查点顺序轮到本计划时钱够不够）。

#### Scenario: 未达标时 gap 为正
- **WHEN** 目标金额 5000，当前余额合计 3200
- **THEN** gap=1800，met=false

#### Scenario: 已达标时 met 为 true
- **WHEN** 目标金额 5000，当前余额合计 5300
- **THEN** gap=-300，met=true

#### Scenario: 满足率计算
- **WHEN** 目标金额 4000，全局分配到 3000
- **THEN** allocated=3000，satisfaction=75

### Requirement: 攒钱计划状态计算
SavingPlanService SHALL 提供 `get_saving_plan_status(plan_id, date)` 方法。`current_balance` SHALL 为账户集合（含所有后代子账户，利用闭包表 `account_ancestors` 展开）截至查询日（含当日）的余额合计，且仅统计 commodity_id 与计划币种匹配的分录。`accounts` 明细中每个账户的 `balance` 遵循同一口径（含后代、仅本币、截至查询日）。

#### Scenario: 多账户合并余额
- **WHEN** 攒钱计划关联余额宝（余额 3000）和微信零钱（余额 2000）
- **THEN** current_balance = 5000

#### Scenario: 包含后代子账户余额
- **WHEN** 攒钱计划关联 Assets:Bank（自身余额 1000），其子账户 Assets:Bank:Checking 余额 2500
- **THEN** current_balance = 3500

#### Scenario: 非本币分录不计入
- **WHEN** 计划币种为 CNY，账户有 CNY 余额 4000 和 USD 余额 100
- **THEN** current_balance = 4000 CNY（100 USD 不计入）

#### Scenario: 查询截至当日的余额
- **WHEN** 账户在查询日之后有一笔 500 入账，查询日之前余额 4500
- **THEN** current_balance = 4500（查询日后的交易不计入）

### Requirement: 过期计划的状态返回
查询日 > deadline 时，状态 SHALL 返回 `expired=true`，同时仍返回其余字段（period_start/period_end、target_amount、current_balance、gap、met）的正常计算值。过期计划 MUST NOT 参与全局资金分配（不占用资金，也不被其他计划视为竞争者）；其 `allocated`/`satisfaction`/`accounts` 按无竞争的退化口径计算（等同于全局只存在它一个计划，`occupied_by_earlier` 恒为 0）。

#### Scenario: 过期计划仍返回完整状态
- **WHEN** 查询 deadline=2026-09-30 的攒钱计划在 2026-10-15 的状态，当前余额 5200，目标 5000
- **THEN** expired=true，且 current_balance=5200、gap=-200、met=true 正常返回

#### Scenario: 未过期计划 expired 为 false
- **WHEN** 查询 deadline=2026-09-30 的攒钱计划在 2026-09-30 的状态
- **THEN** expired=false

#### Scenario: 过期计划不占用资金
- **WHEN** 计划 1 deadline=2026-08-31（已过期）关联账户 A，计划 2 deadline=2026-09-30 关联账户 A，A 余额 3000，两计划目标均为 3000
- **THEN** 计划 2 的 occupied_by_earlier 为 0，satisfaction 为 100

## ADDED Requirements

### Requirement: 全局资金分配遍历
状态计算 SHALL 基于一次全局分配遍历：收集所有参与计划（未过期且有检查点），按 `commodity_id` 分组（跨币种互不争钱），组内按（检查点, plan_id）升序排列，顺序为每个计划执行一次资金占用。每个计划只计算一次（无论周期与否），占用额 `allocated = min(target, available)`，`available = Σ(账户 balance − 已被占用)`。欠费（available < target）的计划 SHALL 占光其全部可用资金。检查点定义：一次性计划为 deadline；周期计划为 min(查询日所在周期末, deadline)。永久计划（period 与 deadline 皆空）MUST NOT 参与分配，其分配字段按无竞争退化口径计算。

#### Scenario: 按检查点顺序先到先得
- **WHEN** 计划 1（{A,B} 目标 3000）检查点早于计划 2（{A,C} 目标 2000），A 3000、B 1000、C 500
- **THEN** 计划 1 allocated=3000（satisfaction=100，按分配偏好为 A 2000 + B 1000），计划 2 可用 = A 剩 1000 + C 500 = 1500，satisfaction=75

#### Scenario: 欠费计划占光可用
- **WHEN** 计划 1（{C,D} 目标 4000），C 2000、D 1000，无更早计划占用
- **THEN** 计划 1 allocated=3000，satisfaction=75

#### Scenario: 跨币种不争钱
- **WHEN** 计划 1（CNY，{A} 目标 3000）检查点早于计划 2（USD，{A} 目标 100），A 有 CNY 3000 与 USD 50
- **THEN** 计划 2 的 USD 可用不受计划 1 占用影响，satisfaction=50

#### Scenario: 周期计划检查点为当前周期末
- **WHEN** 计划 1（一次性，deadline=2026-09-30，{A} 目标 1000）与计划 2（monthly，{A} 目标 500），查询日 2026-07-15
- **THEN** 计划 2 检查点（2026-07-31）早于计划 1（2026-09-30），计划 2 先占用

#### Scenario: 永久计划不参与分配
- **WHEN** 计划 1（period/deadline 皆空，{A} 目标 1000）与计划 2（deadline=2026-09-30，{A} 目标 2000），A 余额 2500
- **THEN** 计划 2 的 occupied_by_earlier 为 0（计划 1 未占用），计划 1 的分配字段按无竞争退化口径返回

### Requirement: 账户内分配偏好
计划 i 在其账户集合 Sᵢ 上落实 `allocated` 的扣减分布时，SHALL 找到排序在 i 之后第一个满足 Sⱼ ∩ Sᵢ ≠ ∅ 的计划 j，优先从 Sᵢ \ Sⱼ（与 j 无关的账户）扣减，不足时再从 Sᵢ ∩ Sⱼ 扣减；同类账户内按账户 id 升序逐个取 `min(余额−已占用, 剩余需扣)`。无后续交集计划时直接按账户 id 升序扣减。

#### Scenario: 为下一交集计划保留交集账户
- **WHEN** 计划 1（{A,B} 目标 3000）、计划 2（{C,D} 目标 4000）、计划 3（{A,E} 目标 2000）按检查点排列，A 3000、B 1000、C 2000、D 1000、E 500
- **THEN** 计划 1 的分配为 A 2000 + B 1000（尽量不动与计划 3 交集的 A），计划 1 satisfaction=100

#### Scenario: 交集账户被保留后的级联
- **WHEN** 同上例，计划 3 计算时 A 已被计划 1 占用 2000
- **THEN** 计划 3 可用 = A 剩 1000 + E 500 = 1500，allocated=1500，satisfaction=75

#### Scenario: 无后续交集计划时按账户顺序分配
- **WHEN** 计划 1（{C,D} 目标 4000）之后无任何交集计划，C 2000、D 1000
- **THEN** 计划 1 的分配为 C 2000 + D 1000（占光），satisfaction=75

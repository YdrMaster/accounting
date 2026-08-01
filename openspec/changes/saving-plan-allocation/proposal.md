# Proposal: saving-plan-allocation

## Why

saving-plan 变更（已归档 2026-08-01-saving-plan）中，每个攒钱计划的状态独立计算：余额合计 vs 目标。当多个计划共享同一资产账户时，同一笔钱被重复计数，满足率虚高——例如账户 A 有 3000 元，计划 1（A+B 目标 3000）和计划 3（A+E 目标 2000）各自独立看都「快达标了」，实际上钱不够分。需要按时间紧迫度（检查点顺序）全局分配资金归属，让满足率反映真实的先到先得。

## What Changes

- 攒钱计划状态计算从「单计划独立查询」改为「全局分配遍历」：同币种的有效计划按检查点升序，每个计划只计算一次，顺序占用资金，欠费计划占光其全部可用。
- 分配偏好：每个计划分配时，向前看**下一个账户有交集的计划**，尽量先占用与其无关的账户，为后续计划保留交集账户。
- 状态输出增加：`allocated`（实际分配到金额）、`satisfaction`（满足率 = allocated/target）、每账户明细（余额/被更早计划占用/本计划分配）。
- 参与规则：过期计划（查询日 > deadline）与永久计划（period/deadline 皆空）不参与全局分配，状态保持现有独立计算；分配按 commodity 分组，跨币种不争钱。
- CLI `saving-plan show` 显示满足率与账户分配明细；`saving-plan list` 增加满足率列。
- API status 响应增加 allocated/satisfaction/账户明细字段（向后兼容的新增字段）。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `saving-plan-report`: 核心变化——状态计算改为全局分配遍历；SavingPlanStatus 增加 allocated/satisfaction/账户明细；参与规则（过期/永久计划除外、币种分组、检查点定义）。
- `saving-plan-api`: status 响应结构增加 allocated、satisfaction、accounts 明细字段。
- `saving-plan-cli`: show 输出满足率与账户分配明细；list 增加满足率列。
- `saving-plan-tracking`: Service 层增加全局分配计算入口（单计划状态查询内部改为全局计算取一行）。

## Impact

- **service 层**: `accounting-service/src/report/saving_plan.rs`——新增全局分配算法（核心改动），`get_saving_plan_status` 改为基于分配结果。
- **API 层**: `accounting-api/src/dto.rs` + `handlers/saving_plan.rs`——status 响应新字段。
- **CLI 层**: `accounting-cli/src/cmd/saving_plan.rs`——show/list 输出。
- **SQL 层**: 预计无 schema 变更（分配为纯计算，不落库）；可能需要按账户集合批量取余额的查询调整（现有 `account_balance_by_ids` 是合计口径，分配需要按单个账户的余额）。
- **前端**: 不在本次范围（新增字段向后兼容）。

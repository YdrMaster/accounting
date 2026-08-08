# Design: fix-saving-plan-badge

## Context

攒钱计划卡片（`accounting-web/src/views/SavingPlanView.vue`）的状态徽标当前用 `status.met` 判定：

```
徽标:  v-else-if="status.met" → 「已达标」,否则 → 缺口徽标
环形:  Number(status.satisfaction) >= 100 ? 绿 : 黄
```

两个字段口径不同（见 `openspec/specs/saving-plan-report/spec.md`):

- `met = current_balance >= target_amount`——账面口径，余额直接取自计划账户集合，共享账户时各计划值相同；
- `satisfaction = allocated / target_amount * 100`——分配口径，`allocated` 来自按（检查点， plan_id）排序的全局资金占用，每计划独立。

于是共享账户的计划只要余额够第一个计划达标，全部显示「已达标」，与各自环形互相矛盾。

## Goals / Non-Goals

**Goals:**

- 卡片徽标与环形统一使用分配口径 `satisfaction` 判定，各计划徽标互相独立。
- 补共享账户场景的测试，防回归。

**Non-Goals:**

- 不改后端 `met` 语义与 API DTO（账面口径在详情区仍有价值，规格明确规定详情区同时展示两种口径）。
- 不改详情展开区的任何展示。
- 不改后端分配算法。

## Decisions

- **前端修口径，不动后端。** 卡片徽标判定从 `status.met` 改为 `Number(status.satisfaction) >= 100`，与 `ringClass`/`ringColor` 已有的判定保持一致。备选方案是把后端 `met` 改成分配口径——否决：会破坏 saving-plan-report 规格定义的账面语义，且详情区依赖该字段展示账面口径，改动面大、收益相同。
- **沿用现有 i18n 词条**(`savingPlan.metBadge` / `gapBadge`)，文案不变。
- **TDD**：先在 `SavingPlanView.spec.ts` 加"两计划 met 均为 true 但 satisfaction 不同"的失败用例，再改实现。

## Risks / Trade-offs

- [用户想从卡片直接看账面达标情况] → 详情展开区仍展示账面 `met`，信息不丢失，只是入口深一层。

# Proposal: fix-saving-plan-badge

## Why

攒钱计划卡片列表中，「已达标」徽标使用账面口径 `met`(`current_balance >= target_amount`）判定。当多个计划共享账户（或账户集合重叠）时，各计划的 `current_balance` 来自同一批账户余额，只要余额够一个计划达标，所有共享账户的计划都会显示「已达标」——用户看到的是"第一个已达标就每个都显示已达标"，与卡片环形（按分配口径 `satisfaction` 着色）互相矛盾：环形 25% 黄色却挂着「已达标」徽标。

## What Changes

- 攒钱计划卡片的「已达标」徽标改为按分配口径判定：`satisfaction >= 100` 才显示「已达标」，否则显示缺口徽标；与环形颜色规则（100 绿 / <100 黄 / 失效灰）一致。
- 展开的状态详情区保持现状：继续同时展示账面口径（余额/缺口/met）与分配口径（已分配/满足率），账面 `met` 字段及其后端语义不变。
- 后端 `met` 语义与 API 不变（`met` 仍按独立余额口径），本次仅修正前端卡片徽标使用的口径。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `saving-plan-view`: 「计划列表展示」需求中卡片状态徽标的判定口径从账面 `met` 改为分配口径 `satisfaction`，并新增共享账户场景的徽标场景。

## Impact

- 前端：`accounting-web/src/views/SavingPlanView.vue`（卡片徽标判定）,`accounting-web/src/views/__tests__/SavingPlanView.spec.ts`（新增/更新测试）。
- 规格：`openspec/specs/saving-plan-view/spec.md`。
- 后端、API DTO、i18n 词条（沿用现有 metBadge/gapBadge）均无变化。

# account-drag-reposition

## Why

当前调整账户层级结构几乎不可能：后端没有任何修改账户父节点的能力，建错的账户只能删除重建（还需先清空子账户和交易）。同时新建账户抽屉中的父账户下拉选择步骤繁琐。拖放重定位 + "先建为子账户、再拖放归位"的流程能让账户树的组织变得直接、低成本。

## What Changes

- 账户页支持拖放卡片重定位：将账户卡片拖到另一张卡片上，使其成为目标账户的子账户（移动即整棵子树迁移）。
- 支持同级内拖动排序，顺序仅保存在浏览器 localStorage，不落库、不跨设备同步。
- 允许跨账户类型拖放（后端不限制），跨类型时前端弹确认提示（移动会改变账户及子树的推导类型）。
- 支持拖动已关闭的账户。
- 新增后端 API `PUT /api/accounts/{id}/parent`：校验防成环、同级不重名等，事务内更新 `parent_id` 并重建 `account_ancestors` 闭包表。
- **BREAKING（前端交互）**：新建账户抽屉移除"父账户"选择字段；只能先选中一个账户再点新建，创建为其子账户；未选中时点击新建提示"请先选择一个账户"。账户类型字段随之移除（类型由父链推导）。

## Capabilities

### New Capabilities

- `account-move`: 后端变更账户父节点的服务能力——端点、校验规则（防成环、同级重名、根账户不可移动）、闭包表事务重建、移动语义（整棵子树路径变更、移动前后视为不同账户不做特殊处理）。

### Modified Capabilities

- `account-page`: 新建账户流程变更（移除父账户选择，依赖选中态）；新增拖放重定位交互（落卡为子、同级内存排序、跨类型确认、localStorage 持久顺序）。

## Impact

- **后端**：`accounting-sql`（repo 新增 parent 更新 + 闭包重建方法）、`accounting` 或 `accounting-service`（校验逻辑）、`accounting-api`（新端点 `PUT /api/accounts/{id}/parent`）。
- **前端**：`accounting-web` 的 `AccountsView.vue`、`AccountGrid.vue`、`AccountRowGroup.vue`、`AccountCard.vue`、`AccountCreateDrawer.vue`、`api/client.ts`；可能引入拖放实现（手写手势或轻量库）。
- **数据**：无 schema 变更（不新增排序字段）；`account_ancestors` 闭包表在移动时重建。
- **OpenSpec**：修改 `openspec/specs/account-page/spec.md` 中"新建账户"相关需求。

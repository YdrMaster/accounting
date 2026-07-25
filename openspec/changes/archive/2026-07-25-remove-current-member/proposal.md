# remove-current-member

## Why

系统里有两套互不相干的"当前用户"概念，均为死代码：服务端 `GET/PUT /api/me` + settings 表 `current_member_id`（2026-06-06 API 骨架期引入，前端从未调用，settings 键永远为空）；前端 member store 的 `currentMemberId`（内存态，恒等于成员列表第一项，唯一用途是交易表单的默认成员）。两套机制都没有真实的用户切换语义，反而制造了"系统知道当前用户"的假象。系统约定改为：所有需要成员的地方一律显式指定（CLI 导入的 `--member` 、交易表单的成员下拉本就是如此）。

## What Changes

- 删除后端 `GET/PUT /api/me` 端点及其 handler、`MeDto`/`SetMeRequest`、专属 i18n key；settings 表 `current_member_id` 键自然废弃（存量数据无害，不做迁移）
- 删除前端 member store 的 `currentMemberId`
- 交易表单成员字段改为默认未选中，用户手工选择（下拉、占位符、必填校验均已存在，仅去掉默认值）
- **BREAKING**：`GET/PUT /api/me` 端点移除（无已知调用方；前端从未使用）

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `transaction-form`: 成员选择默认值从"当前用户（GET /api/me）"改为"默认未选中，用户手工选择"

## Impact

- **accounting-api**：删除 `handlers/me.rs`、`router.rs` 的 me 路由、`dto.rs` 两个 DTO、locales 两个 key（`parse_member_id_failed`、`no_members`，已确认仅 me.rs 使用；`member_not_found` 因导入端点校验成员存在而保留）
- **accounting-web**：`stores/member.ts` 删 `currentMemberId`；`TransactionFormOverlay.vue` 删默认值（第 45 行）；相关测试更新
- **spec 层面**：`/api/me` 从未有独立 spec，删除为纯代码动作
- **关联变更**：`add-web-import`（未归档）已先行返工，导入端点改为显式 `member_id` 参数，不再依赖本变更删除的 `resolve_current_member_id`

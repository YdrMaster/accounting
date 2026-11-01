# Proposal: account-delete-cascade-feedback

## Why

账户删除目前有两个可用性缺陷：其一，所有删除失败（账户被引用、有子账户等）在后端被序列化为 HTTP 200 的错误文本，前端无从分辨，把失败当成功处理——不仅没有任何提示，还会把未删除的账户从列表中移除，形成刷新即复现的幽灵状态；其二，删除只允许叶子账户，即使整棵子树（各级所有后代）都没有任何引用，也必须逐个手动删除后代，操作繁琐。

## What Changes

- **BREAKING** 后端 `DELETE /api/accounts/{id}` 错误契约修复：业务校验失败 SHALL 返回 4xx/5xx 状态码与本地化错误文案，SHALL NOT 以 2xx 返回错误信息（对前端调用方是行为变更：失败从「200 + 文本」变为明确的错误状态码）。
- 删除引用检查从 2 类扩展到 5 类：分录（postings）、账户映射（account_mappings）之外，新增预算限额（budget_limits）、攒钱计划（saving_plan_accounts）、渠道（channels）引用；报错文案区分引用来源。
- 支持级联删除：目标账户及其整棵子树（经闭包表枚举）均未被任何引用占用时，一次请求删除整棵子树；子树内任一账户被引用时拒绝删除，错误文案指名挡路的后代账户及其引用来源。
- 前端删除确认弹窗无条件出现：有子账户时文案体现级联范围（「删除 X 及其 N 个子账户？」，N 由前端从账户树本地计算）；叶子账户维持现有确认文案。
- 前端删除失败改为弹窗提示（复用全局 `alertDialog`），失败时不再从列表移除账户。
- 补充删除链路的测试（后端 handler 级 + 前端 API 层）与新增 i18n 文案（后端 locales + 前端 locales）。

## Capabilities

### New Capabilities

- `account-delete`: 账户删除服务能力——`DELETE /api/accounts/{id}` 的引用检查（5 类引用面）、子树级联删除语义、指名道姓的本地化错误与「错误不伪装为成功」的 HTTP 契约。对齐既有 `account-move` 能力的组织方式。

### Modified Capabilities

- `account-page`: 「抽屉式编辑面板」需求中的「删除账户」场景——确认弹窗文案体现级联范围、删除失败以弹窗明确提示且不产生幽灵列表状态。

## Impact

- 后端：`accounting-api/src/handlers/account.rs`（delete_account 错误契约与级联编排）、`accounting-sql/src/repo/account.rs`（子树枚举/子树级引用检查/子树删除）、`accounting-api/locales/en.yaml` 与 `zh-CN.yaml`（新增错误文案 key）。
- 前端：`accounting-web/src/api/client.ts`（deleteAccount 错误透传）、`accounting-web/src/components/layout/AccountDrawer.vue`（确认与失败弹窗）、`accounting-web/src` locales（确认文案）。
- 测试：`accounting-api/src/handlers/account.rs` 测试模块（当前无任何 delete 测试，需补五类引用拦截、级联成功、后代挡路指名）、`accounting-web/src/api/__tests__/client.spec.ts`。
- 兼容性：单机 SQLite、自有前后端同仓发布，无外部 API 消费方；状态码语义变化仅影响本仓库前端（同步更新）。

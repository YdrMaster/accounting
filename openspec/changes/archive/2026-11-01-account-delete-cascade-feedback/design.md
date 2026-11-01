# Design: account-delete-cascade-feedback

## Context

`DELETE /api/accounts/{id}` 现状（`accounting-api/src/handlers/account.rs:225-270`）：

- handler 返回 `Result<String, String>`，axum 将 `String` 错误序列化为 **200 OK + 纯文本**，所有业务拒绝被前端当成功。
- 编排内联在 handler：查账户 → 查直接子账户（非空即拒）→ 查分录 → 查映射 → 单行删除，五步散落、无事务。同文件 close/reopen 则通过 `AccountService` 编排（`account.rs:197-222`），删除是唯一绕过 service 层的写操作。
- 引用检查仅覆盖 `postings` 与 `account_mappings` 且只查账户自身；`budget_limits`、`saving_plan_accounts`、`channels` 三类引用未检查，靠开启的外键（`accounting-sql/src/database.rs:55`）在删除时兜底报错——错误同样变成 200 被吞。
- 前端 `client.ts` 的 `deleteAccount` 仅在非 2xx 时抛错；`AccountDrawer.doDelete` 将 resolve 视为成功并 `emit('deleted')`，`AccountsView` 随即从 store 移除该账户，产生幽灵状态。项目已有全局 `alertDialog`/`confirmDialog`（`accounting-web/src/utils/dialog.ts`），交易删除失败即用 `alertDialog` 弹窗（`TransactionFormOverlay.vue:240-249`）。

动机与行为契约见 proposal.md 与 specs delta（`account-delete`、`account-page`）。

## Goals / Non-Goals

**Goals:**

- 删除失败的 HTTP 契约可判定：4xx/5xx + 本地化文案，前端弹窗呈现。
- 级联删除干净子树：一次请求删除目标及全部后代，检查与删除同事务。
- 引用检查覆盖全部 5 类引用面，且范围为整棵子树。
- 拒绝文案指名挡路账户及其引用来源。

**Non-Goals:**

- 不改其他账户端点（rename/set_parent/close/reopen）的错误契约，尽管它们部分也返回裸 `String` 错误——本次只修 delete 的契约缺陷。
- 不提供「强制删除并清理引用」之类的破坏性选项；被引用即拒绝，用户先清理引用。
- 不引入全局 toast 组件；沿用现有 `alertDialog`。
- 不改 close/reopen 的级联语义。

## Decisions

### D1: handler 错误类型改为 `(StatusCode, String)`，语义化状态码

`delete_account` 签名改为 `Result<String, (StatusCode, String)>`：账户不存在 → 404，系统账户 → 400，子树被引用 → 409 Conflict，其余内部错误 → 500。沿用同文件 `rename_account`（400）与 `set_parent`（409/400）的既有写法及 `account-move` 规格中「错误不伪装为成功」的先例。

被否决的备选：JSON `ApiError { error }` 包装（budget handler 的模式）——前端 `deleteAccount` 已按纯文本 `res.text()` 读取错误体，纯文本方案改动面最小；账户 handler 内无 JSON 错误先例，为单一端点引入新响应格式得不偿失。

### D2: 级联删除编排移入 `AccountService`，服务返回结构化错误

新增 `AccountService::delete`（或同义方法），在 service 层完成「子树枚举 → 引用检查 → 子树删除」的事务编排，与 close/reopen 的级联编排同层，符合「事务编排在 accounting-service」的项目分层。服务返回结构化错误枚举（`NotFound` / `SystemAccount` / `Blocked { account_id, source }` / `Db`），handler 负责把枚举映射为 `t!` 本地化文案与状态码——i18n 保留在 HTTP 层，service 层不感知语言。

被否决的备选：继续在 handler 内联编排（现状）——delete 是唯一绕过 service 的写操作，本次要加事务与子树逻辑，内联会进一步坐实分层破洞；且 handler 测试需要通过完整 router，编排下沉到 service 后单测更直接。

### D3: 子树枚举与删除用闭包表 + 单语句 DELETE

子树取 `SELECT account_id FROM account_ancestors WHERE ancestor_id = ?1`（闭包表含自身 depth=0 行，一次取出目标及全部后代，已有 `idx_account_ancestors_ancestor` 索引）。引用检查对子树 ID 集逐一探测 5 类引用（`postings`、`account_mappings`、`budget_limits`、`saving_plan_accounts`、`channels`），取第一个挡路账户即短路返回。子树干净则 `DELETE FROM accounts WHERE id IN (…)` 单语句删除，`account_ancestors`/`account_owners`/`account_names` 由 `ON DELETE CASCADE` 清理。

被否决的备选：

- 前端递归删叶子——N 次请求、非原子、半途失败留下半删树，且违背「级联语义在后端」的契约（specs 已定为单请求原子删除）。
- 递归 CTE 实时展开子树——闭包表本就是为 O(1) 后代查询建的（见 `accounting-sql/README.md`），CTE 是重复建设。

### D4: 检查与删除同事务

sqlx 事务内完成 D3 全部步骤，消除「检查通过后、删除前」并发写入分录/映射的竞态窗口。SQLite 单写者、个人记账规模，事务开销可忽略。被否决的备选：检查与删除分离、靠 FK 兜底——竞态下要么神秘 500、要么依赖不可读的外键文案，与本次「明确报错」的目标相悖。

### D5: 拒绝文案指名挡路账户与引用来源

`Blocked { account_id, source }` 中 `source` 为五类引用来源之一；handler 据此选择对应 i18n key（如 `subtree_blocked_by_posting` / `subtree_blocked_by_mapping` / `subtree_blocked_by_budget` / `subtree_blocked_by_saving_plan` / `subtree_blocked_by_channel`），并以请求语言查 `account_names` 取挡路账户显示名，经 `rust_i18n::t!` 带参插值（`main.rs:55` 已有带参先例）。`en.yaml`/`zh-CN.yaml` 同步补 key。被否决的备选：泛化文案「子树中存在被引用账户」——用户需自行翻树定位，已确认采用指名方案。

### D6: 前端确认与失败均为弹窗，失败不再移除列表项

- 确认：无条件弹确认（沿用现有 `confirmDialog`）；叶子账户沿用 `drawer.deleteConfirm`，有后代时改用新 key `drawer.deleteSubtreeConfirm`（文案如「删除〈name〉及其 N 个子账户？」），N = 后代总数，由前端从既有账户树本地计算，不加后端接口。
- 失败：`catch` → `alertDialog(t('drawer.deleteFailed', { message }))`，`message` 为后端返回的本地化文案（D1 修复后 `!res.ok → throw` 链路自然打通，`client.ts` 无需改动）；失败路径不 `emit('deleted')`，幽灵状态随之消失。
- 被否决的备选：抽屉内联错误横幅（现状 `error.value`）——用户明确要求弹窗；仅叶子确认——用户明确要求无条件确认。

### D7: 测试补在 handler/service 与前端 API 层

- 后端：`account.rs` 测试模块当前无任何 delete 测试，补齐——系统账户拒绝（4xx）、404、五类引用各一（含「后代挡路」，验证指名文案与 409）、干净多级子树级联成功（验证后代行与闭包行清除）。
- 前端：`client.spec.ts` 补 `deleteAccount` 非 2xx 抛错并携带响应文本。
- 被否决的备选：仅 SQL 层测试（现状仅 `account_mapping.rs` 有删除拦截测试）——本次缺陷根因在 handler 契约与编排，SQL 层测不到 200 伪装。

## Risks / Trade-offs

- [引用面盘点遗漏新增表] → 以 `schema.rs` 全量 `REFERENCES accounts` 为准（当前 5 类无级联引用即全部），测试按五类逐一覆盖；后续新增引用表时由 FK 兜底（D4 事务内表现为 500 而非静默成功）。
- [409 语义放宽] → 「子树被引用」用 409 Conflict 表达资源状态冲突，非严格 HTTP 语义下的并发修改冲突；与 `account-move` 的 409 先例一致，前端只判状态码类别，无实际影响。
- [大子树一次删除] → 单语句 `DELETE ... IN` 在个人记账数据量下无性能风险；事务期间独占写锁时间极短。
- [错误文案参数化依赖 `account_names` 显示名] → 挡路账户在请求语言下必有名称行（系统种子与建户流程保证），查不到时回退用 ID 兜底文案。

## Migration Plan

无数据迁移、无 schema 变更。前后端同仓同步发布，错误契约变化（200 → 4xx/5xx）的唯一消费方是本仓库前端，随本 change 一并更新；不存在过渡期双格式兼容问题。回滚直接 revert。

## Open Questions

无——引用面扩展、报错粒度、确认弹窗形态均已在探索阶段与用户确认。

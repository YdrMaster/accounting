# Tasks: account-delete-cascade-feedback

## 1. accounting-sql：子树枚举、引用探测与批量删除

- [x] 1.1 repo/account.rs 新增子树账户 ID 枚举查询（`account_ancestors WHERE ancestor_id = ?`，含自身 depth=0 行），附单元测试：多级子树返回完整 ID 集（含目标与各级后代），叶子账户仅返回自身。验证：`cargo test -p accounting-sql`
- [x] 1.2 新增子树引用探测查询：对子树 ID 集检查 `postings` / `account_mappings` / `budget_limits` / `saving_plan_accounts` / `channels` 五类引用，返回第一个挡路 `(account_id, 来源)` 或 `None`，附单元测试：五类引用各命中一例、深层后代挡路可被检出、全干净返回 `None`。验证：`cargo test -p accounting-sql`
- [x] 1.3 新增按 ID 集合批量删除账户的语句封装（`DELETE FROM accounts WHERE id IN (…)`，关联行靠 `ON DELETE CASCADE`），附单元测试：删除多级子树后 accounts 行、闭包行、`account_owners` 行全部清除。验证：`cargo test -p accounting-sql`

## 2. accounting-service：事务编排与结构化错误

- [x] 2.1 `AccountService` 新增删除方法：单事务内完成「子树枚举 → 五类引用检查 → 子树删除」，校验目标存在与非系统账户；返回结构化结果/错误枚举（`NotFound` / `SystemAccount` / `Blocked { account_id, source }` / 内部错误），附 service 测试：干净叶子删除成功、干净多级子树级联成功、自身被引用与后代被引用分别返回 `Blocked` 且指认正确账户与来源、系统账户与不存在账户返回对应错误。验证：`cargo test -p accounting-service`
- [x] 2.2 service 测试断言失败路径零副作用：`Blocked` / `SystemAccount` / `NotFound` 后目标与子树原样保留、闭包行不变。验证：`cargo test -p accounting-service`

## 3. accounting-api：错误契约、本地化与 handler 测试

- [x] 3.1 `accounting-api/locales/en.yaml` 与 `zh-CN.yaml` 新增文案 key：五类来源各一条（含 `{name}` 参数，如「子账户〈{name}〉有关联分录，不能删除」）与查不到显示名时的 ID 兜底条目，中英文同步。验证：`cargo test -p accounting-api` 中断言文案渲染
- [x] 3.2 `delete_account` handler 改为调用 service 删除方法，将结构化错误映射为 `(StatusCode, String)`：`NotFound` → 404、`SystemAccount` → 400、`Blocked` → 409（按来源选 key，按 `Lang` 查 `account_names` 取挡路账户显示名插入文案）、内部错误 → 500；移除 handler 内联的旧检查编排与 `delete_children_first` 拒绝分支。验证：`cargo build -p accounting-api` 通过
- [x] 3.3 清理孤儿代码：确认 `posting_has_postings` / `account_mapping_count_by_account` / `delete_children_first` key 是否还有其他调用方，无则删除。验证：`cargo test --workspace` 通过且无 unused 警告
- [x] 3.4 handler 测试模块补齐 delete 路由测试：系统账户（含根）拒绝且非 2xx、不存在 ID 返回 404、五类引用各一返回 409 且文案含挡路账户名与来源、干净多级子树删除成功且后代从列表消失、「错误不伪装为成功」断言（任一失败场景状态码非 2xx）。验证：`cargo test -p accounting-api`

## 4. accounting-web：确认与失败弹窗

- [x] 4.1 前端 locale 新增 `drawer.deleteSubtreeConfirm`（含 `{name}` 与 `{count}` 参数）与 `drawer.deleteFailed`（含 `{message}` 参数），中英文同步。验证：`npm run build`（i18n key 编译期校验如项目支持）
- [x] 4.2 `AccountDrawer.doDelete`：确认弹窗无条件出现——无后代沿用 `drawer.deleteConfirm`，有后代改用 `drawer.deleteSubtreeConfirm`（N 为本地账户树计算的后代总数）；失败 `catch` → `alertDialog(t('drawer.deleteFailed', { message: e.message }))`；仅成功路径 `emit('deleted')`。验证：组件逻辑走查 + `npm test`
- [x] 4.3 `client.spec.ts` 补 `deleteAccount` 非 2xx 响应抛错并携带响应文本、2xx 正常 resolve 的用例。验证：`npm test`
- [x] 4.4 浏览器端到端验证（dev server）：叶子账户删除（确认 → 列表移除、抽屉关闭）；多级干净子树删除（确认文案含正确 N → 整树消失）；有分录账户删除失败弹窗并指名账户；深层后代被引用时删除祖先失败弹窗指名后代；被预算/攒钱计划/渠道引用的账户删除失败弹窗说明来源；失败后刷新前后列表一致（无幽灵移除）。验证：逐场景在浏览器操作确认

## 5. 集成验证

- [x] 5.1 后端全量：`cargo test --workspace` 与 `cargo clippy --workspace` 通过。验证：命令零失败
- [x] 5.2 前端全量：`npm test` 与 `npm run build` 通过。验证：命令零失败

## Workflow follow-up

- `openspec validate account-delete-cascade-feedback` 通过后按项目流程 `openspec archive`（归档时同步活规格 `account-delete` 与 `account-page`）。

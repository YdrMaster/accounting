# Proposal: fix-amount-invalid-input

## Why

交易表单（新建/编辑）中，把某个分录金额框清空后再输入一个 `-`（同样可复现的还有 `.` 和 `-.`），会导致整个交易页面白屏，必须刷新才能恢复。根因是金额输入过滤器允许 `-`、`.`、`-.` 这类非法中间态字符串留在输入框里，而渲染期的配平计算直接用 `new Decimal(amount)` 解析该字符串，在 render 阶段抛出 `DecimalError`。由于 `-` 是 truthy，代码里的 `amount || 0` 兜底完全失效。这是一个可直接触发的页面崩溃缺陷，需要修复并让非法输入有明确的视觉与提交反馈。

## What Changes

- 为分录金额引入明确的合法性判定：仅 `-?` 前缀 + 至少一位数字 + 至多一个小数点的文本视为合法（如 `-5`、`10.`、`.5`）；空串视为「未填」，其余视为非法。
- 金额输入框内容非法（非空且不合法）时，输入框显示红色轮廓作为错误态。
- 配平计算与「添加分录」的自动配平改用容错解析：非法金额不参与求和、不抛异常，使页面在任何输入中间态下都不再崩溃。
- 收紧提交条件：只有当所有分录金额**非空且合法**、至少两笔分录、日期与成员已填、且所有分录金额之和为零时，确认按钮才可用。
- 非法金额不再被当作「已填写」：`-` 这类 truthy 非法串必须让确认按钮与「添加分录」按钮保持禁用（禁止提交给后端）。

## Capabilities

### New Capabilities
- 无

### Modified Capabilities
- `transaction-form`: 新增「金额输入合法性」要求（非法文本标红、不崩溃、不可提交），并修正「借贷平衡校验」「自动配平」使其在非法/未填金额下安全降级。

## Impact

- 前端：`accounting-web/src/components/layout/TransactionFormOverlay.vue`（金额输入过滤、配平计算、自动配平、`canSubmit`/`canAddPosting`、输入框样式）。
- 测试：`accounting-web/src/components/layout/__tests__/TransactionFormOverlay.spec.ts` 补充非法金额用例。
- 无后端、API、数据库或依赖变更；后端金额校验逻辑不变。

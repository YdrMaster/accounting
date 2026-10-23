# Tasks: fix-amount-invalid-input

## 1. 消除渲染期崩溃

- [x] 1.1 在 `accounting-web/src/components/layout/TransactionFormOverlay.vue` 引入 `parseAmount(text): Decimal | null`（空串与非法文本返回 `null`，其余返回 `Decimal`）与 `isAmountValid(text): boolean`，并让 `isBalanced`、`addPosting` 的求和改用 `parseAmount`（遇 `null` 视为不平，不入和）
- [x] 1.2 在 `accounting-web/src/components/layout/__tests__/TransactionFormOverlay.spec.ts` 增加用例：编辑态把某分录金额改为 `-` 后组件仍能正常渲染（不抛出异常），验证方式为 `cd accounting-web && npm test -- TransactionFormOverlay` 通过

## 2. 非法金额错误态

- [x] 2.1 金额输入框在「非空且非法」时绑定错误态 class（如 `.invalid`），并在 `TransactionFormOverlay.vue` 样式中新增红色轮廓（`border-color: var(--color-expense)`）；合法 `-5`/`5.`/`.5`/`0` 与空串不得标红
- [x] 2.2 补充测试断言：输入 `-` 时 `.amount-input` 具错误态 class 且输入值保留，输入 `-5` 或清空后不具该 class，验证方式为 `cd accounting-web && npm test -- TransactionFormOverlay` 通过

## 3. 收紧提交与添加分录条件

- [x] 3.1 `canSubmit` 与 `canAddPosting` 改为要求每条分录金额「非空且合法」（`isAmountValid`），保留原有日期/成员/至少两笔/配平为零条件
- [x] 3.2 补充测试：存在非法金额、空金额、不平、缺成员时确认按钮禁用，`-` 存在时「添加分录」按钮禁用；所有金额非空且合法、日期成员已填且配平为零时确认按钮可用，验证方式为 `cd accounting-web && npm test` 全部通过

## Workflow follow-up

- 运行 `openspec validate fix-amount-invalid-input --strict`，确认零告警后归档变更。

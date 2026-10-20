# Tasks: transaction-zero-amount

## 1. 抽取共享金额计算（D4）

- [x] 1.1 在 `accounting-web/src/utils/amount.ts` 新增共享纯函数（如 `sumAssetAmount(postings): Decimal`），返回资产类分录（`account_type === 'asset'`）金额之和、不含任何回退；在 `accounting-web/src/utils/__tests__/amount.spec.ts` 补用例（支出 -35、收入 +8000、零净额 -100/+100、无资产分录），`npm test -- amount` 通过。

## 2. 卡片金额去回退 + 低对比度 0.00（D1/D2）

- [x] 2.1 `TransactionCard.vue` 的 `computeAmount()` 改调共享函数、去掉零净额回退；在 `TransactionCard.spec.ts` 增补用例：非转账交易含 -100/+100 且另含非内置账户分录时，金额文本为 `0.00` 且不含 `+`；`npm test -- TransactionCard` 通过。
- [x] 2.2 为「非转账且金额为 0」输出 muted 样式类（如 `amount-zero`），`<style>` 用 `--text-muted`；在 `TransactionCard.spec.ts` 断言金额含 `amount-zero`、不含 `amount-positive` 与 `amount-transfer`。
- [x] 2.3 补边界用例：非转账零净额且无正值资产分录（如单条金额 `0` 的资产分录、或无资产分录）同样渲染 muted `0.00`；`npm test -- TransactionCard` 通过。

## 3. 按日分组汇总同口径（D3）

- [x] 3.1 `TransactionList.vue` 的 `computeAmount()` 改调共享函数、去掉回退；新增 `accounting-web/src/components/__tests__/TransactionList.spec.ts`，覆盖：某日仅含零净额非转账交易时分组头收/支均 `0.00`；某日仅含纯转账交易时该转账不计入收/支；`npm test` 通过。

## 4. 集成校验

- [x] 4.1 在 `accounting-web` 依次运行 `npm test`、`npm run build`、`npm run lint`，全部通过。
- [x] 4.2 在仓库根运行 `openspec validate transaction-zero-amount --strict` 通过。

## Workflow follow-up

- 项目评审通过后归档该变更（`openspec archive transaction-zero-amount`）；归档默认同步 `transaction-list-ui` 主规格。
- 校验归档结果：主规格「金额计算规则」含新增的零金额与按日汇总场景，且无 `## ADDED/MODIFIED/REMOVED` 增量头残留。

# Tasks: transaction-transfer-and-delete

## 1. 后端：转账判定与 DTO 契约

- [x] 1.1 在 `accounting-api` 组装 `TransactionDto` 时实时判定转账（≥2 条资产分录且 ≥2 个不同 `account_id`；资产分录同时存在 `amount<0` 与 `amount>=0`；非资产分录账户的 ID ∈ 由系统英文名（`is_system=1`/`lang='en'`）解析出的 {`Expenses:Fees`,`Expenses:InstallmentFees`,`Equity:Discounts`}），并新增只读字段 `is_transfer`；验证：`cargo test -p accounting-api` 用例覆盖纯转账、带手续费、带折扣、含零金额资产分录、期初余额、非内置支出、内置账户改名，断言各自 `is_transfer` 取值。
- [x] 1.2 前端 `accounting-web/src/types/api.ts` 的 `TransactionDto` 同步新增 `is_transfer: boolean`；验证：`npm run build`（含 vue-tsc 类型检查）通过。

## 2. 转账卡片渲染

- [x] 2.1 `TransactionCard.vue` 折叠态新增转账分支：`tx.is_transfer` 为真时，金额显示为蓝色（`--color-info`）、无正负号，取值 = 资产类分录中负金额之和的相反数；非转账维持现有红/绿逻辑；验证：组件测试断言纯转账（-1000/+1000）显示 `1000.00`、无 `+`、命中蓝色类，且普通支出/收入用例仍通过。
- [x] 2.2 转账时计算资产变动额 `S = 非资产类分录金额之和`：`S>0` 追加红色 `-S`，`S<0` 追加绿色 `+S`，`S=0` 不追加；验证：组件测试断言带手续费（Fees +10）显示红色 `-10.00`、带折扣（Discounts -10）显示绿色 `+10.00`、纯转账不出现第二个金额。
- [x] 2.3 清理已无引用的 i18n 键 `txCard.transfer`（中英文）；验证：`grep -rn "txCard.transfer" accounting-web/src` 无结果且 lint/测试通过。

## 3. 删除入口

- [x] 3.1 `TransactionFormOverlay.vue` 的 `form-footer` 新增「删除」按钮（仅编辑态显示），点击经 `confirmDialog` 确认后调用 `txStore.remove(id)` 并 `emit('close')`；验证：表单组件测试断言编辑态显示、新建态不显示、确认后删除并关闭、取消后表单保持。
- [x] 3.2 触摸端保留右滑删除并补充过程反馈（右滑位移 + 越阈高亮，阈值 60px），右滑越阈仍 `emit('delete')`；验证：组件测试模拟 `touchstart`/`touchmove`/`touchend` 断言越阈前无删除、越阈后触发 `delete` 且出现反馈类。

## 4. 集成验证

- [x] 4.1 前端整体验证：`npm run test`、`npm run build`、lint 全部通过；人工核对交易页与日历页的转账卡片（纯转账/手续费/折扣）与删除（触摸右滑、表单按钮）表现符合规格。
- [x] 4.2 后端整体验证：`cargo test -p accounting-api` 与 `cargo clippy` 通过；确认既有非转账用例无回归。

## Workflow follow-up

- 变更实现完成后，使用 `openspec validate transaction-transfer-and-delete` 校验，随后 `openspec archive` 同步活规格。

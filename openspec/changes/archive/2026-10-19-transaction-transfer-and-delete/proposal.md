# Proposal: transaction-transfer-and-delete

## Why

交易页面存在两个缺陷：（1）转账交易（资产账户间转移，可能伴随手续费/折扣）在折叠态卡片上与收入同形——绿色金额带 `+` 号，用户无法识别这是一笔转账；（2）删除交易在非触摸端完全没有入口，触摸端也只有一个无视觉反馈的右滑手势，导致该功能事实上不可发现、不可用。

## What Changes

- **转账判定由后端实时给出**：`TransactionDto` 新增只读布尔字段 `is_transfer`，由 API 组装 DTO 时按分录结构实时判定（不落盘），前端据此走转账渲染分支。
- **转账识别规则**：≥2 条资产分录且涉及 ≥2 个不同账户；资产分录中同时存在金额 `<0` 与 `≥0` 的记录（一出一进）；所有非资产分录必须属于 3 个内置账户 `Expenses:Fees`、`Expenses:InstallmentFees`、`Equity:Discounts`。内置账户身份由不可变的系统英文规范名（`is_system=1`、`lang='en'`）解析为账户 ID，判定只比对账户 ID，`SHALL NOT` 依赖本地化或用户自定义显示名。
- **转账卡片渲染**：折叠态金额改为蓝色（`--color-info`）、无符号，取值 = 资产分录中负金额之和的相反数（即「转出金额」）；若非资产分录金额之和记为 `S` 且 `S ≠ 0`，在其后再显示一个资产变动额——`S > 0`（资产减少）显示红色 `-S`，`S < 0`（资产增加）显示绿色 `+S`。转账不再使用「资产正值之和」，也不再使用收入绿色。
- **删除入口**：非触摸端在编辑表单底部提供「删除」按钮（仅编辑态显示）；触摸端在保留右滑删除的同时也提供该按钮，并为右滑补上过程反馈（越过阈值的高亮）。删除一律经确认对话框，硬删除、不提供撤销。
- **BREAKING**（契约扩展）：`TransactionDto` 新增字段，前端 `types/api.ts` 同步；旧客户端忽略该字段即可，属向后兼容，无需数据迁移。

## Capabilities

### New Capabilities

（无新增 capability。）

### Modified Capabilities

- `transaction-list-ui`：交易卡片展示与金额计算规则中的转账口径，由「资产分录正值之和 / 白色无符号」改为「蓝色转出金额 + 可选资产变动额」，并明确转账识别规则。
- `transaction-entry-display`：删除入口由「仅触摸端右滑」扩展为「触摸端右滑（带过程反馈）+ 编辑表单底部删除按钮（非触摸端唯一入口）」。
- `transaction-form`：编辑态表单底部新增「删除」操作。
- `transaction-api`：`TransactionDto` 新增只读字段 `is_transfer`。

## Impact

- **后端**：`accounting-api`（`TransactionDto` 契约与 DTO 组装，实时判定转账）。不改数据库 schema、不做数据迁移、不动报表聚合。
- **前端**：`accounting-web` 的 `TransactionCard.vue`（转账渲染、右滑反馈）、`TransactionFormOverlay.vue`（底部删除）、`types/api.ts`、i18n 文案与组件测试。
- **明确不在本轮范围**：收支统计口径（月收支汇总、日期分组头、`transaction-summary-api` 报表）与转账的关系，另行处理。

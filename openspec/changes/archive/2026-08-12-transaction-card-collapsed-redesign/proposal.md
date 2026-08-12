# Proposal: transaction-card-collapsed-redesign

## Why

交易卡片折叠态的展示存在两个问题：一是纯导入（待分类 pending）交易被整体隐藏金额、收支账户、资产账户，而这些交易恰恰最需要金额来辅助识别"这是哪一笔"；二是折叠态布局依赖某些区域有内容才不塌陷——描述为空时标题位 `flex:1` 吃满整行，留下大片空白。此外，类型判断分支（`isTransfer` / `isPureImport`）驱动的"转账""待处理"徽标体系增加了模板复杂度，却没有承载独立信息。

## What Changes

- 折叠态卡片重设计为**两行锚定结构**：第一行「主标题 + 金额」（右侧锚定），第二行「账户摘要 + 成员 + 标签 + 展开指示」。所有状态下空白都不会因为字段缺失而出现。
- **主标题回退链**：`描述 → 账户摘要`。描述为空时两行合并为一行，摘要直接进位为标题，消除 flex 空洞。
- **账户摘要统一为「按分录金额符号分侧对位」**：非正值（≤0）分录在左、正值（>0）分录在右，箭头由左指向右（资金流向），同侧多账户按序用 `、` 连接。不再分析账户类型。
- **移除 `isTransfer` / `isPureImport` 判断与「转账」「待处理」文字徽标**，所有交易以统一形式展示，不再有纯导入分支。对应行为从 `transaction-entry-display` 规格中删除。
- **折叠态金额对所有交易必现**（含纯导入/待分类交易），修复原始"额度没有显示"的缺陷。
- **pending 由琥珀色渐变标识**：待分类交易卡片背景左端琥珀色向右淡出（峰值约 20% 透明度），取代文字标签，文字保留原对比度。红/绿金额语义不受影响。
- **多币种金额符号策略**：交易仅含单一（主）币种时金额不带符号；含多币种时金额带主币种符号，次要币种在摘要行标出。
- 退款（kind=refund）、可报销（is_reimbursable）本期不做，保持现状不动。

## Capabilities

### New Capabilities
<!-- Capabilities being introduced. Use kebab-case for path segments you introduce
     (e.g., user-auth or identity/user-auth) that follow the project's existing
     spec organization. Each creates specs/<capability-path>/spec.md. -->

（无新增 capability——本次是既有展示行为的重构。）

### Modified Capabilities
<!-- Existing capabilities whose REQUIREMENTS are changing (not just implementation).
     Only list here if spec-level behavior changes (not just implementation details).
     Each needs a delta spec file. -->

- `transaction-list-ui`: 交易卡片展示需求改写为两行锚定结构、摘要符号分侧对位、金额必现、pending 渐变；金额计算规则场景同步更新。
- `transaction-entry-display`: 删除「纯导入账单检测 isPureImport」与「纯导入账单隐藏显示」需求；保留分录展开、独立状态、动画与手势需求。

## Impact

- **代码**：`accounting-web/src/components/TransactionCard.vue` 为主改动（模板结构、摘要计算、样式渐变）；`TransactionList.vue` 无需改动。
- **测试**：新增/更新组件测试覆盖——描述为空合并单行、pending 渐变标识、多币种符号、摘要对位；移除 isPureImport 相关断言。
- **规格**：两个既有 capability（`transaction-list-ui`、`transaction-entry-display`）的 delta spec。
- **后端 / API**：`TransactionDto` 增加 `pending: bool` 只读字段（服务端按系统标签判定，如系统 `pending` 标签已附加即 true），供前端做渐变标识；`tx.tags` 仍为本地化显示名。
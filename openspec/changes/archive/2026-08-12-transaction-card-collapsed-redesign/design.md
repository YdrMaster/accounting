# Design: transaction-card-collapsed-redesign

## Context

折叠态交易卡片（`accounting-web/src/components/TransactionCard.vue`）现为「顶/中/底」三行 flex 布局：顶部 `transfer/ie-accounts` 标签段、中部 `tx-name`（`flex:1`）+ 金额、底部 `asset-accounts` 右对齐。空穴来源于：`flex:1` 标题位在描述为空时撑满；`isPureImport()` 整体隐藏金额/账户两段（历史遗留——Import 根账户已移除多年，见 `71c6c1f6`，但隐藏分支未同步）。动机见 proposal.md——Why。

约束与现状：
- 前端 Vue 3 + TS，`TransactionDto` 由 `accounting-api` 组装；`tx.tags` 为**本地化显示名**（`tag_names_by_transactions(&ids, lang)`），不含系统标志。
- 卡片展开行、双击编辑、左右滑手势、展开动画为稳定行为，不参与本次改动。
- `accounting-web/src/stores/commodity.ts` 已有商品列表，可提供币种符号。

## Goals / Non-Goals

**Goals:**
- 折叠态任意字段缺失均不出现空白区域（两行锚定，无 `flex:1` 空洞）。
- 全类型统一渲染路径；待分类（pending）依赖稳定的判定信号并显式标识。
- 金额对每笔交易必现，符号策略随币种数量变化。

**Non-Goals:**
- 不改展开态、交互手势、懒加载/分组/月汇总。
- 不动退款（refund）、可报销（reimbursable）的现有展示。
- 不改变金额计算口径（asset 分录之和、为零取正值之和）。

## Decisions

### D1 折叠态改两行 CSS Grid 锚定
两行结构替代现有三行：
```
行1: [主标题 minmax(0,1fr)] [金额 auto] [ ▼ auto]
行2: [成员] [摘要] [标签...] ──────── [ ▼ auto]
描述为空 → 仅渲染行1（行1 的主标题位由摘要充任 + 金额居右）
```
`grid-template-columns: minmax(0, 1fr) auto`，主标题列 `min-width: 0` 保证长文本省略而非撑破。
- **备选**：保留 flex 三行并给 `.tx-name` 一个最小占位 —— 否决：空洞仍由字段缺失触发，只是换个表现；且三行结构在描述空时仍冗余。
- **备选**：主标题空时渲染占位文案 —— 否决：占位文案无信息，不如把真正相关的账户摘要前置。

### D2 摘要按分录金额符号分侧对位
以 `posting.amount` 符号分组：非正值（≤0）账户短名（leaf）在前、正值（>0）在后，中间 `" → "`；同侧多账户按分录顺序 `、` 连接。摘要计算完全不读 `account_type`。
- 移除 `isTransfer` / `isPureImport` / `getIncomeExpenseAccounts` / `getAssetAccounts` 四个函数；保留 `shortAccountName`、`formatAmount`。
- 金额计算 `computeAmount` 口径不变（转账仍以正值之和为折叠金额）。
- **备选**：按账户类型区分收支/资产对位 —— 否决：与用户确认改为资金流方向语义，且类型推导在 Import 路径下易误导。

### D3 pending 判定走 API 字段，不匹配标签名
`TransactionDto` 新增只读 `pending: bool`，由 `accounting-api` 在组装 DTO 时判定（该交易是否关联系统 `pending` 标签；系统标签按 `is_system` + 系统名判定，见 `resolve_pending_tag_id`）。前端仅在 `tx.pending` 为真时应用渐变。
- **备选**：前端在 `tx.tags` 中匹配 "pending" —— 否决：`tx.tags` 是本地化显示名，随界面语言变化，且用户自定义标签可能同名，判定不稳定。
- **备选**：前端匹配本地化词（zh "待处理"） —— 否决：硬编码语言词，随 i18n 演进漂移。

### D4 pending 琥珀渐变作为容器级标识
`.tx-card` 命中 `pending` 时叠加背景：
`background: linear-gradient(90deg, rgba(245, 158, 11, 0.20), rgba(245, 158, 11, 0) 65%);`
文字颜色不变（渐变仅是容器层）；与支出红/收入绿金额不发生色相冲突。不再渲染任何「待处理」文字徽标。
- **备选**：文字标签 / 左缘竖条 —— 否决：用户确认纯粹的渐变表达更符合待分类视觉提示（不产生徽标体系）。

### D5 多币种金额符号
交易 `postings` 的 `commodity` 去重数量 == 1 时，金额无符号；> 1 时：
- 主币种 = 出现次数最多者（平票取先出现；CNY 优先），金额前缀其符号；
- 币种符号取自 `CommodityDto.symbol`（如 `¥`/`$`）；取不到 symbol 时回退用币种代码。
- 次要币种代码在摘要行尾部 ` · <code>` 标出。
- **备选**：金额统一不显示符号、依赖上下文 —— 否决：多币种账目对新人可读性差。

## Risks / Trade-offs

- [渐变 20% 峰值在深色/浅色主题下的对比度漂移] → 用 `rgba` 固定 alpha 叠加，不做主题算术；冒烟验证文字可读性。
- [单行合并场景 member + tags + 摘要三要素叠加，标签 ≥3 时行内拥挤] → `text-overflow: ellipsis` 硬兜底防挤出；折叠 tag 截断阈值 N 接真实数据后再启用（Open Question，不阻塞本变更）。
- [`TransactionDto.pending` 是 API 契约小扩展] → 同步更新前端 `types/api.ts` 与客户端测试；后端复用已有系统标签查询，无数据迁移。
- [`pending` 判定依赖系统标签存在性] → 后端已具备稳定解析（`resolve_pending_tag_id`），对历史无标签数据恒 false，安全。
- [与并发变更 `user-facing-text-i18n` 重叠于标签实体名] → 本变更通过 `is_system` 系统标志判定，避开名字匹配，无冲突。

## Migration Plan

无数据迁移。上线顺序：后端随现有发布带上 `pending` 字段（向后兼容，旧前端忽略新字段）→ 前端模板替换一次性完成。回滚：还原 `TransactionCard.vue` 与 `types/api.ts` 即回到现状，无持续状态。

## Open Questions

- 多币种主币种规则细节（等频平票的次序）——纯展示细分，可在实现时定并以组件单测锁定，不影响规格。
- 折叠 tag 截断阈值 N 的取值——需真实数据分布（见 `user-facing-text-i18n` 前后的标签使用情况），延迟到应用上线后数据驱动。
- 未知 symbol 币种的前缀回退文案（用代码还是空）——实现时按 CommodityDto 现状定。
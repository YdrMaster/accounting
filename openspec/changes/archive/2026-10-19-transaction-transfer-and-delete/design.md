# Design: transaction-transfer-and-delete

## Context

动机见 proposal.md — Why。相关现状与约束：

- 折叠态卡片（`accounting-web/src/components/TransactionCard.vue`）金额由 `computeAmount()` 与 `displayAmount` 共同决定：`displayAmount.sign = amount.gt(0) ? '+' : ''`，模板用 `:class="{ 'amount-positive': amount.gt(0) }"` 上收入绿色。转账（资产分录之和为零、回退取正值之和）因此与收入同形。
- `TransactionList.vue` 另有一份复制的 `computeAmount`，仅用于日期分组头的收/支汇总，本轮不改。
- 前端 `PostingDto.account` 是**本地化完整路径**（`account_paths = display_path`），且 `accounting` 的 `Account` 模型不含名字（名字在按语言的 name 表），前端无法稳健识别 3 个内置账户。
- 后端 `load_display_context` 已持有全量账户映射，并具备按语言取规范名的能力（`account_display_names(ids, "en")`）；但「en 显示名」可被用户改掉，故识别改用不可变的系统英文名。
- 删除已全线接线（卡片 `emit('delete')` → `TransactionList` → 视图 `onDeleteTx` → `confirmDialog` → `txStore.remove` → `DELETE /api/transactions/:id`），仅入口是触摸端右滑且无过程反馈。

## Goals / Non-Goals

**Goals:**
- 交易卡片能识别转账并以中性方式呈现（蓝色转出额 + 可选资产变动额）。
- 交易删除在触摸端与非触摸端都有可发现的入口。

**Non-Goals:**
- 不改收支统计口径（月收支汇总、日期分组头、`transaction-summary-api` 报表）。
- 不改展开态、账户摘要对位、懒加载与按日分组。
- 不引入软删除或撤销。

## Decisions

### D1 转账判定由后端实时计算，不落盘
理由：判定是仅依赖分录（`account_id` + `account_type` + `amount`）与 3 个固定内置账户的纯函数；落盘需在 create/update/delete/import 及账户种子演进时同步维护，而 `update` 会整体替换 postings，最易漂移；本轮唯一用途是列表渲染，实时计算为 O(分录数)，成本可忽略。
实现落点：`accounting-api` 组装 `TransactionDto` 时计算；内置账户身份由系统英文规范名（`is_system = 1`、`lang = 'en'`）一次性解析为账户 ID（`account_ids_by_system_english_names`），判定分录时只比对 `account_id`，符合 `builtin-data-english-storage`。
- **备选**：新增 `transactions.is_transfer` 列——否决：需 schema 迁移并承担同步负担，其唯一价值（可索引的过滤/聚合）本轮不需要。

### D2 识别规则锁定为「≥2 资产账户 + 双符号 + 非资产白名单」
条件见 `transaction-list-ui`「转账交易识别」。要点：用「≥2 条资产分录且 ≥2 个不同 `account_id`」而非 `every(asset)` 或旧 `!some(income||expense)`，以排除期初余额、返现等 asset+equity 组合；用「同时存在 `amount<0` 与 `amount≥0`」确保一出一进（含「一负一零」，排除全非负/全负）。
- **备选**：沿用旧 `!some(income||expense)`——否决：会把 `Equity:OpeningBalances` 配对的期初余额误判为转账。
- **备选**：允许任意非资产分录——否决：会把普通「支出 + 手续费」等误判为转账。
- **备选**：按本地化/自定义显示名匹配 3 个内置账户——否决：显示名可被用户修改（系统名记录不可改），改名后识别会失效；改为按系统英文名解析出的账户 ID 判定。

### D5 内置账户身份用「系统英文名 → 账户 ID」而非硬编码 ID 或显示名
`accounts.id` 是 `AUTOINCREMENT`，没有编译期常量；内置账户的稳定身份是 `account_names` 中 `is_system = 1 且 lang = 'en'` 的系统名记录（改名只新增/切换显示名，系统名文本不可改）。因此新增 `account_ids_by_system_english_names`（`accounting-sql`）一次性解析 `Fees` / `InstallmentFees` / `Discounts` 三个账户 ID，后续判定只比对 `account_id`。这也与既有报表（如 `account_get_by_name("Equity:OpeningBalances")`）识别系统账户的口径一致。
- **备选**：本地上传硬编码账户 ID——否决：ID 非固定，随数据库创建顺序变化。
- **备选**：前端按 `PostingDto.account` 路径字符串匹配——否决：路径是本地化显示名，既不稳定也要在每处重复规则。

### D3 转账渲染：蓝色转出额 + 可选资产变动额
- 蓝色转出额 = 资产类分录中负金额之和的相反数（自转出账户流出的金额），无正负号，用 `--color-info`。
- 资产变动额：`S = 非资产类分录金额之和`；`S > 0`（资产减少）显示红色 `-S`，`S < 0`（资产增加）显示绿色 `+S`，`S = 0` 不显示。
- 理由：转出额是「这笔转账动了多少钱」的稳定锚点；`S` 表达手续费/折扣对净资产的净影响，与配平恒等式 `Σ资产 = -S` 一致。
- **备选**：蓝色取转入额（正资产分录之和）——否决：与用户期望的「转出金额」相反，且手续费场景下转入额会掩盖损耗。
- **备选**：蓝色取「所有负金额分录之和的相反数」——否决：折扣分录为负会被计入，优惠场景下会显示成转入额。
- 两个数值均由前端从 `postings` 现算；后端只提供 `is_transfer`，不重复计算金额。

### D4 删除入口：表单底部按钮为主，触摸端保留右滑并补反馈
- 编辑表单（`TransactionFormOverlay.vue` 的 `form-footer`）新增「删除」按钮，仅 `isEdit` 显示；点击 → `confirmDialog` → 删除 → `emit('close')`。
- 触摸端卡片右滑保留，新增过程反馈（右滑位移 + 越阈高亮），阈值仍 60px。
- 理由：既有参照（`BudgetView` / `SavingPlanView`）一律以「显式按钮 + 确认」实现删除；表单底部按钮在桌面可发现、与编辑同处，无需发明新手势。
- **备选**：卡片展开态内放删除按钮——否决：展开态是只读分录表，塞入操作行会破坏其语义。
- **备选**：右键菜单 / 长按菜单——否决：需新建菜单组件，且可发现性差。

## Risks / Trade-offs

- [右滑与移动端系统「右滑返回」冲突] → 保持 `touch-action: pan-y` 与 60px 阈值，并通过过程反馈让用户在越阈前即可放弃；表单底部按钮作为无手势备选入口。
- [前端展示与后端判定重复表达同一规则] → 规则口径只在规格「转账交易识别」单一陈述，前端仅消费 `is_transfer`；前端只做纯展示的金额计算。
- [`TransactionDto` 契约新增字段] → 向后兼容（旧客户端忽略新字段）；同步更新前端 `types/api.ts` 与相关测试。
- [日期分组头仍会把转账计入「收」] → 已知限制，本轮显式设为 Non-Goal，另行变更处理。

## Migration Plan

无数据迁移、无 schema 变更。上线顺序：后端先随发布提供 `is_transfer`（向后兼容，旧前端忽略）→ 前端切换转账渲染与删除入口。回滚：还原 DTO 组装与前端组件即可回到现状，无持续状态。

## Open Questions

- 蓝色转出额与资产变动额并排展示的分隔方式（间距/分隔符）——纯展示细节，实现时定并以组件测试锁定。
- 右滑过程中的反馈样式（底色/图标/位移幅度）——实现时定，不改变规格与任务拆分。

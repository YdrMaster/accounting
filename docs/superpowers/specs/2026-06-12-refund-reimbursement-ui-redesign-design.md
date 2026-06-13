# 退款/报销 UI 重设计文档

## 背景

现有退款/报销功能在 `accounting-web/src/views/TransactionForm.vue` 中要求用户直接填写 `linked_posting_id`（原分录 ID）来建立关联。该 ID 是数据库层概念，用户不应感知。因此需要重新设计 UI：

- 在 Dashboard 的日历区域增加独立的「退款模式」和「报销模式」。
- 用户通过点选分录（Posting）来选择要冲减的原始分录。
- 进入专用录入界面后，为每个选中的原分录自动生成冲减分录，用户只需填写到账资产和金额。

## 当前状态分析

### 后端现状

`postings` 表已包含退款/报销所需字段（`accounting-sql/src/schema.rs`）：

```sql
kind INTEGER NOT NULL DEFAULT 1 CHECK(kind BETWEEN 1 AND 3),
linked_posting_id INTEGER REFERENCES postings(id),
reversal_total INTEGER NOT NULL DEFAULT 0,
```

- `kind`: 1=Normal, 2=Refund, 3=Reimbursement。**计划重构**：将 `kind` 移到 `transactions` 表，作为交易级概念。
- `linked_posting_id`: 退款/报销分录指向被冲减的原分录。`linked_posting_id IS NOT NULL` 即标识该分录为冲减分录（替代 `kind` 的判断作用）。
- `reversal_total`: 由触发器自动维护，记录原分录累计被冲减金额。

`Posting` 领域模型（`accounting/src/posting.rs`）已包含 `linked_posting_id`、`reversal_total`；`Transaction`（`accounting/src/transaction.rs`）将新增 `kind` 字段。

`TransactionService`（`accounting-service/src/transaction_service.rs`）已实现对退款/报销分录的验证：

- `linked_posting_id` 必须非空。
- 被指向分录必须存在且 `kind == Normal`（重构后将改为被指向分录 `linked_posting_id IS NULL`）。
- 两个分录的 `account_id` 必须相同。
- 金额方向必须与原分录相反。

**注意**：当前存在两个缺失——(1) 冲减金额上限校验；(2) 未抽取到交易表，缺少交易级语义纯净性。本次一并补充/重构。

API DTO（`accounting-api/src/dto.rs`）已暴露 `kind`、`linked_posting_id`、`reversal_total`。

### 前端现状

1. **主题**：应用支持亮/暗色主题，通过 `accounting-web/src/stores/theme.ts` 管理，`App.vue` 会根据 `isDark` 切换 `html.dark` 类。

2. **Dashboard**：`accounting-web/src/views/Dashboard.vue`
   - 顶部工具栏有「范围选择」按钮，用于切换日历的范围模式。
   - 使用 `Calendar` 组件（`accounting-web/src/components/Calendar.vue`），当前支持普通模式和 `rangeMode` 范围模式。
   - 下方按日期分组展示交易列表，使用 `TransactionDetail` 组件。
   - 交易列表默认收起，点击日期头部展开/收起；`TransactionDetail` 点击后展开显示分录。

3. **交易表单**：`accounting-web/src/views/TransactionForm.vue`
   - 顶部字段：日期时间、备注、标签、渠道。
   - 分录行：账户树选择、货币选择、类型选择（normal/refund/reimbursement）、金额输入。
   - 当类型为 refund/reimbursement 时，显示 `linked_posting_id` 数字输入框。
   - 提交时将分录映射为 `account` 全名 + `commodity` 符号 + `amount` 字符串 + `kind` + `linked_posting_id`。

4. **交易详情**：`accounting-web/src/components/TransactionDetail.vue`
   - 展示交易摘要、金额、成员、时间、渠道。
   - 展开后显示每个分录的账户、货币、金额。
   - 已支持显示「退」「报」标记和 `linked_posting_id`。

### 缺失

- `postings` 表没有「可报销」标记字段。
- Dashboard 没有退款/报销模式切换。
- 没有多选分录并进入专用录入界面的流程。
- 没有独立的退款/报销录入界面。
- 后端缺少冲减金额上限校验（`abs(冲减金额) + reversal_total <= abs(原金额)`），本次一并补充。

## 设计目标

1. 用户不再需要知道或输入 `linked_posting_id`。
2. 退款和报销有独立的入口和筛选逻辑。
3. 可报销支出在首次录入时即可标记，并在主界面用特殊颜色提示。
4. 退款/报销录入界面与现有「记一笔」界面保持高度一致，降低学习成本。
5. **重构**：将 `kind` 从 `postings` 表移到 `transactions` 表，退款/报销成为交易级概念（`linked_posting_id IS NOT NULL` 标识冲减分录）。
6. 新增 `is_reimbursable` 字段到 `postings`。

## 总体方案

### 1. 数据模型（含重构）

#### 1.1 `kind` 从 postings 迁移到 transactions

```sql
-- transactions 表新增
ALTER TABLE transactions ADD COLUMN kind INTEGER NOT NULL DEFAULT 1 CHECK(kind BETWEEN 1 AND 3);

-- postings 表删除 kind 列（SQLite 不支持 DROP COLUMN，需重建表或通过迁移脚本处理）
-- postings 删除 kind 后，通过 linked_posting_id IS NOT NULL 判断是否为冲减分录
```

- `transactions.kind`: 1=Normal, 2=Refund, 3=Reimbursement。
- `postings.linked_posting_id IS NOT NULL` 标识该分录为冲减分录；具体是 Refund 还是 Reimbursement 由所属交易的 `kind` 决定。
- 冲减分录的 `link_posting_id` 指向被冲减的原分录；非冲减分录的 `link_posting_id` 为 NULL。

对应 Rust 层改动：

- `accounting/src/transaction.rs` 的 `Transaction` 结构体新增 `kind: TransactionKind`。
- `accounting/src/posting.rs` 的 `Posting` 结构体删除 `kind` 字段。
- `PostingKind` 枚举重命名为 `TransactionKind`，移动到 `accounting/src/transaction.rs` 模块。

#### 1.2 新增 `is_reimbursable` 字段

在 `postings` 表新增字段：

```sql
is_reimbursable INTEGER NOT NULL DEFAULT 0
```

对应地：

- `accounting/src/posting.rs` 的 `Posting` 结构体新增 `is_reimbursable: bool`（同时删除 `kind`）。
- `accounting-sql/src/repo/posting.rs` 的查询/插入/更新适配新字段读写（同时删除对 `kind` 列的读写）。
- `accounting-api/src/dto.rs` 的 `PostingDto` / `PostingRequest` 新增 `is_reimbursable: bool`（同时删除 `kind`）；`TransactionDto` / `TransactionDetailDto` 新增 `kind: String`；`CreateTransactionRequest` 新增 `kind: String`。
- `accounting-web/src/stores/transaction.ts` 的 `Posting` / `PostingInput` 接口新增 `is_reimbursable`（删除 `kind`）；`Transaction` / `CreateTransactionData` 接口新增 `kind`。

### 2. 记一笔/编辑交易：可报销标记

在 `TransactionForm.vue` 中：

- **交易级 kind**：当路由为 `/transaction/refund` 或 `/transaction/reimbursement` 时，自动设置交易的 `kind` 为对应值。普通「记一笔」和编辑交易时 `kind` 默认为 `normal`，不显示 kind 选择器（UI 上退款/报销由独立入口区分）。
- **分录行「报销」按钮**：每个分录行右侧增加一个 2 状态按钮：
  - 默认状态：灰色边框 + 灰色文字「报销」。
  - 激活状态：蓝色背景（`#1890ff`）+ 白色文字「报销」。
  - 点击切换，绑定到该分录的 `is_reimbursable` 字段。
  - 只有 `account_type` 为 `Expense` 的分录才显示该按钮。

### 3. Dashboard 模式切换

在 `Dashboard.vue` 顶部工具栏将现有「范围选择」改为 4 个模式按钮：

- 普通
- 范围
- 退款
- 报销

使用单一状态 `mode: 'normal' | 'range' | 'refund' | 'reimbursement'` 管理。

#### 普通模式

保持现有行为。

#### 范围模式

保持现有 `rangeMode` 行为。

#### 退款模式

- 日历保持正常日期筛选功能（用户可通过点击日期/范围筛选交易，方便定位原始分录）。
- 交易列表显示所有交易。
- 展开交易后，所有分录行可点击选中/取消。
- 选中的分录以高亮样式（如 `#bae7ff` 背景 + 蓝色边框）显示。

#### 报销模式

- 日历保持正常日期筛选功能。
- 交易列表自动筛选：只显示包含 `is_reimbursable == true` 分录的交易。
- 展开交易后，只有 `is_reimbursable == true` 的分录可点击选中；其他分录置灰不可点。
- 在普通/范围模式下，可报销分录以淡蓝色背景（`#e6f7ff`）标示，并可选择性地显示「报」字标签。

### 4. 底部抽屉

在退款/报销模式下，有选中分录时从右下角向左弹出一个抽屉：

- 最左侧：
  - 上方：选中分录数量。
  - 下方：展开/收起三角。收起时三角朝左，展开时三角朝右。
- 中间：
  - 上方：绿色「确定」按钮，点击进入对应模式（退款/报销）的录入界面。
  - 下方：灰色「取消」按钮，清空选中并退出当前模式。
- 右侧：
  - 横向排列的已选分录卡片。
  - 卡片高度与两个按钮总高一致。
  - 隐藏水平滚动条；实现阶段补充鼠标拖拽和滚轮事件以横向滚动卡片列表。
  - 每个卡片显示：账户名、日期、交易描述、金额。
- 收起后仅露出最左侧的数量 + 三角区域。

### 5. 退款/报销录入界面

复用 `TransactionForm.vue` 的 UI 结构，通过路由参数或状态区分模式：

- 标题：退款模式显示「录入退款」，报销模式显示「录入报销」。
- 交易 `kind` 根据路由自动设置（Refund 或 Reimbursement），不在界面上显示选择器。
- 顶部字段与现有「记一笔」完全一致：日期时间、备注、标签、渠道。
- 分录区域按每个选中的原分录分组：
  - 组头显示原分录完整信息：账户名、日期、交易描述、原金额、已冲减金额（`reversal_total`）。
  - 第一项为自动生成的只读冲减分录：
    - 账户与原分录一致。
    - 货币与原分录一致，只读显示不显示选择框。
    - 金额为负，等于该组下方普通分录金额之和的相反数。
    - `linked_posting_id` 指向原分录 ID（这是该分录是冲减分录的唯一标识，交易级 `kind` 决定它是 Refund 还是 Reimbursement）。
  - 下方可手动添加普通资产分录：
    - 账户通过树选择器选择 Asset 类型账户。
    - 货币默认与原分录一致，不显示选择框。
    - 金额可编辑。
  - 每组保留「+ 添加分录」按钮，与现有「记一笔」一致。
- 保存时整体校验：
  - 交易平衡（同币种金额和为 0）。
  - 每组冲减金额不超过原分录金额（`abs(冲减金额) + reversal_total <= abs(原金额)`）。
  - 冲减分录金额方向与原分录相反。

### 6. 路由与状态

- `Dashboard.vue` 维护 `mode` 和 `selectedPostings` 状态。
- 点击「确定」后，根据当前模式跳转到独立路由：
  - 退款模式：`/transaction/refund?posting_ids=1,2,3`
  - 报销模式：`/transaction/reimbursement?posting_ids=4,5,6`
- 现有路由保持不变：`/transaction`（新建普通交易）、`/transaction/:id`（编辑交易）。
- `TransactionForm.vue` 通过路由路径检测专用模式：
  - 路径为 `/transaction/refund` 或 `/transaction/reimbursement` 时进入专用录入模式。
  - `posting_ids` 参数传入选中的分录 ID 列表。
  - 调用 API 查询原分录详情（`GET /api/postings/:id`），为每个原分录生成分组。

### 7. 后端 API 变更

#### 新增/修改字段

- `TransactionDto` / `TransactionDetailDto` / `CreateTransactionRequest` 新增 `kind: String`（"normal" | "refund" | "reimbursement"）。
- `PostingDto` / `PostingRequest` 删除 `kind` 字段，新增 `is_reimbursable: bool`。
- `create_transaction` / `update_transaction` 构造 `Transaction` 时携带 `kind`；构造 `Posting` 时不再携带 `kind`，改为通过 `linked_posting_id` 判断是否为冲减分录。
- `validate_reversal_direction` 改为通过 `linked_posting_id.is_some()` 识别冲减分录（不再依赖 `kind`）。

#### 查询接口增强

为支持报销模式下的筛选，做两处调整：

1. 在 `accounting/src/transaction_filter.rs` 的 `TransactionFilter` 中新增字段：

```rust
pub struct TransactionFilter {
    // ... 现有字段
    /// 只包含可报销分录的交易
    pub has_reimbursable: Option<bool>,
}
```

1. 在 `accounting-api/src/handlers/transaction.rs` 的 `TxQuery` 中增加对应参数：

```rust
pub struct TxQuery {
    // ... 现有字段
    pub reimbursable: Option<bool>,
}
```

当 `reimbursable=true` 时，设置 `filter.has_reimbursable = Some(true)`；`TransactionRepo` 的列表查询需联表 `postings` 过滤出包含 `is_reimbursable = 1` 分录的交易。

#### 原分录详情接口

新增 `GET /api/postings/:id` 返回单个分录详情（含所属交易日期、描述、账户名、金额、货币、已冲减金额）。

## 详细改动清单

### 后端

| 文件 | 改动 |
|------|------|
| `accounting-sql/src/schema.rs` | `transactions` 表新增 `kind INTEGER NOT NULL DEFAULT 1 CHECK(kind BETWEEN 1 AND 3)`；`postings` 表删除 `kind` 列，新增 `is_reimbursable INTEGER NOT NULL DEFAULT 0`；新增索引 `idx_transactions_kind`、`idx_postings_reimbursable`；删除索引 `idx_postings_kind`。 |
| `accounting/src/transaction.rs` | `Transaction` 结构体新增 `kind: TransactionKind`。 |
| `accounting/src/posting.rs` | `Posting` 结构体删除 `kind`，新增 `is_reimbursable: bool`。 |
| `accounting/src/validation.rs` | `validate_reversal_direction` 改为通过 `linked_posting_id.is_some()` 识别冲减分录；新增校验：退款/报销交易必须包含至少一个冲减分录，普通交易禁止包含冲减分录。 |
| `accounting-sql/src/repo/transaction.rs` | `TransactionRow` 新增 `kind` 列读写；`list` 查询支持 `has_reimbursable` 过滤联表。 |
| `accounting-sql/src/repo/posting.rs` | `insert`、`get`、`list_by_transaction`、`list_by_account` 删除 `kind` 列读写，新增 `is_reimbursable` 列读写。 |
| `accounting-service/src/transaction_service.rs` | 构造 `Transaction` 时携带 `kind`；去掉 `Posting` 构造中的 `kind`；新增冲减金额上限校验 `abs(冲减金额) + reversal_total <= abs(原金额)`；被指向分录校验改为 `linked_posting_id IS NULL`。 |
| `accounting-api/src/dto.rs` | `TransactionDto` / `TransactionDetailDto` 新增 `kind: String`；`CreateTransactionRequest` 新增 `kind: String`（默认 "normal"）；`PostingDto` 删除 `kind`，新增 `is_reimbursable: bool`；`PostingRequest` 删除 `kind`，新增 `is_reimbursable: Option<bool>`。 |
| `accounting-api/src/handlers/transaction.rs` | 所有 handler 适配字段变更；`TxQuery` 新增 `reimbursable` 参数。 |
| `accounting-api/src/handlers/`（新增或复用） | 新增 `GET /api/postings/:id` 返回单个分录详情。 |

### 前端

| 文件 | 改动 |
|------|------|
| `accounting-web/src/stores/transaction.ts` | `Posting` / `PostingInput` 删除 `kind`，新增 `is_reimbursable`；`Transaction` / `CreateTransactionData` 新增 `kind`；`fetchTransactions` 支持 `reimbursable` 参数；新增 `fetchPosting(id)`。 |
| `accounting-web/src/views/TransactionForm.vue` | 分录行删除 `kind` 选择器，增加「报销」2 状态按钮（仅 Expense 类型显示）；支持独立路由进入退款/报销专用模式（自动设置交易 `kind`）；自动生成分组冲减分录（`linked_posting_id` 赋值）；提交时交易级携带 `kind`。 |
| `accounting-web/src/components/Calendar.vue` | 将 `rangeMode` boolean prop 重构为 `mode` prop（'normal' \| 'range' \| 'refund' \| 'reimbursement'），仅 `range` 时启用日期范围选择。 |
| `accounting-web/src/components/TransactionDetail.vue` | 支持 `selectable` 模式：分录行可点击选中；通过 `linked_posting_id` + 交易 `kind` 显示「退」「报」标记（不再读分录级 `kind`）；可报销分录显示淡蓝色背景；非可选分录置灰。 |
| `accounting-web/src/views/Dashboard.vue` | 工具栏增加「普通」「范围」「退款」「报销」模式切换；管理 `selectedPostings`；报销模式筛选交易；渲染底部抽屉；处理「确定」跳转和「取消」清空。 |
| `accounting-web/src/router/index.ts` | 新增 `/transaction/refund` 和 `/transaction/reimbursement` 路由，指向 `TransactionForm.vue`。 |
| `accounting-web/src/App.vue` | 补充新增组件的暗色主题样式（如抽屉、可报销背景、选中态）。 |

## 验证规则

1. **可报销标记**：仅在 `account_type == 'Expense'` 的分录上允许设置。
2. **交易类型一致性**：`kind` 在 `transactions` 表，schema 级别保证一个交易只有一个类型，无需额外校验。
3. **退款/报销金额上限**：每组冲减金额 + 原分录已有 `reversal_total` 不得超过原分录金额绝对值。
4. **方向正确**：冲减分录金额方向必须与原分录相反（通过 `linked_posting_id.is_some()` 识别冲减分录）。
5. **交易平衡**：同币种分录金额之和为 0。
6. **货币一致**：冲减分录与原分录使用相同货币；手动添加的资产分录也使用相同货币。

## 测试要点

1. 在「记一笔」中标记分录为可报销，保存后重新打开编辑，标记状态保留。
2. 报销模式下只显示含可报销分录的交易，且只有可报销分录可点。
3. 退款模式下可选任意分录。
4. 选中分录后底部抽屉正确显示数量和卡片，确定后跳转专用界面。
5. 专用界面自动生成冲减分录（`linked_posting_id` 正确赋值），手动添加资产分录后金额自动平衡。
6. 超出原分录金额的报销/退款被后端拒绝。
7. 交易 `kind` 为 `normal` 时提交包含 `linked_posting_id` 的分录被拒绝。
8. 交易 `kind` 为 `refund`/`reimbursement` 时缺少冲减分录被拒绝。
9. 暗色主题下各新增 UI 元素颜色正确。

## 范围说明

本设计专注于 UI 流程改造和必要的 `is_reimbursable` 字段支持，不涉及：

- 退款/报销统计规则变更（保持现有 `kind` + `linked_posting_id` 语义）。
- 后端 `reversal_total` 触发器变更。
- 新增独立的报销状态工作流（如「已提交报销」「已到账」等）。

# 账户名称重构：full_name → name + parent_id

## 现状

当前 `Account` 结构体存储 `full_name: String`，如 `"Assets:Bank:Checking"`，用 `:` 分隔的层次路径表示账户在树中的位置。数据库中对 `full_name` 设置了 `UNIQUE` 约束。

## 问题

### 1. full_name 是闭包表的冗余派生数据

`full_name` 的层级信息完全可以从 `parent_id` + 闭包表推导出来。闭包表已经完整存储了层级关系，`full_name` 是冗余的派生信息。

### 2. GUI 侧 full_name 是负担而非便利

前端代码中对 `full_name` 最常见的操作是 `split(':')` 切出最后一段当本名显示：

| 文件 | 代码 | 目的 |
|------|------|------|
| `AccountPicker.vue:90` | `acc.full_name.split(':').pop()` | 菜单项只显示本名 |
| `AccountCards.vue:26` | `shortName(account.full_name)` | 卡片只显示短名 |
| `AccountTree.vue:180` | `selectedAccount.full_name.split(':').pop()` | 重命名时取最后一段 |
| `TransactionDetail.vue:181` | `lastSegment(acc.full_name)` | 交易详情只显示短名 |
| `useAccountTree.ts:13` | `acc.full_name.split(':')` | 构建树时拆分层级 |

GUI 上定位账户依赖树状选择器，天然是多级结构，用不上 `full_name` 的路径拼接。

### 3. 重命名中间节点需要级联更新所有后代

当前 `rename()` 只更新了本节点的 `full_name`，没有级联更新子节点——这是一个现有 bug。如果重命名 `"Assets"` 为 `"Holdings"`，所有 `"Assets:Bank:Checking"` 等子节点的 `full_name` 前缀都应同步更新，但当前代码没有做。

### 4. CLI 侧按名查找的性能不是瓶颈

CLI 每次执行都是新进程，启动时就要打开数据库、初始化 schema、加载 i18n。即使定位 `"Assets:Bank:Checking"` 需要按 `parent_id` 逐级查 3 次，相比进程启动开销可忽略。

## 重构方案

### 数据模型变更

```rust
// 改前
pub struct Account {
    pub full_name: String,       // "Assets:Bank:Checking"
    // ...
}

// 改后
pub struct Account {
    pub name: String,            // "Checking"
    // ...
}
```

### 数据库变更

```sql
-- 改前
full_name TEXT NOT NULL UNIQUE

-- 改后
name TEXT NOT NULL,
UNIQUE(parent_id, name)   -- 同级不重名，跨级可重名
```

去掉 `full_name` 列和 `UNIQUE` 约束，改为 `name` 列 + `(parent_id, name)` 联合唯一约束。

### 按名查找的替代方案

CLI 的 `get_by_name("Assets:Bank:Checking")` 改为逐级查找：

1. 查 `name = "Assets" AND parent_id IS NULL`
2. 查 `name = "Bank" AND parent_id = <上一步id>`
3. 查 `name = "Checking" AND parent_id = <上一步id>`

或者用递归 CTE 一条 SQL 完成。

### 重命名中间节点

改为 `name` 后，重命名中间节点只需更新一条记录的 `name` 字段，零级联。闭包表不需要更新（层级关系没变）。

### 移动子树

改为 `name` + `parent_id` 后，移动子树只需：
1. 更新目标节点的 `parent_id`
2. 重新维护闭包表

不需要重写任何后代的名称。

### API 层变更

- 请求/响应 DTO 中的 `full_name` 字段可保留或改为 `name` + `path`（可选，供显示用）
- 创建账户时不再需要传 `full_name`，改为传 `name` + `parent_id`
- 路径信息可在 API 层通过闭包表查询动态组装

### 前端变更

- 账户树组件直接使用 `name` 显示，去掉所有 `split(':')` 操作
- 创建/重命名账户时只操作 `name`，不再拼接路径

## 影响范围

| 模块 | 变更 |
|------|------|
| `accounting/src/account.rs` | `full_name` → `name` |
| `accounting/src/account_type.rs` | `from_prefix()` 不再需要（类型由树根决定，见 account-type-refactor.md） |
| `accounting/src/closure.rs` | `AccountNode.full_name` → `name` |
| `accounting-sql/src/schema.rs` | 表结构变更 + 种子数据调整 |
| `accounting-sql/src/repo/account.rs` | `get_by_name()` 改为逐级查找或递归 CTE |
| `accounting-sql/src/repo/posting.rs` | 测试中的 `full_name` 引用 |
| `accounting-service/src/account_service.rs` | `create_cascading()` 重写，不再拼接路径 |
| `accounting-service/src/report_service.rs` | 显示名调整 |
| `accounting-cli/src/cmd/` | 命令参数和输出格式调整 |
| `accounting-api/src/dto.rs` | DTO 字段调整 |
| `accounting-api/src/handlers/` | handler 逻辑调整 |
| `accounting-web/src/` | 去掉所有 `split(':')` 操作 |

## 结论

将 `full_name` 改为 `name` + `parent_id` 唯一约束。`full_name` 的路径信息是冗余派生数据，在 GUI 侧是负担（到处 `split`），在 CLI 侧的查询优势不构成性能瓶颈。改名后消除了重命名中间节点的级联更新问题（当前是 bug），并使移动子树成为可能操作。

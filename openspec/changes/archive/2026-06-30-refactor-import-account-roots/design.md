## Context

当前系统为导入交易维护了一个独立的 `Import/导入` 根账户类型。支付宝等适配器输出的无映射分录会落到：

```
导入:支付宝:资产:蚂蚁宝藏信用卡
导入:支付宝:收支:餐饮美食
```

为了向 beancount 导出，代码里不得不把 `导入:` 前缀替换成 `Equity:Import:`，并打上 `account_type: "Import"` metadata。`Equity:Import` 不是 beancount 原生账户类型，造成兼容性和可移植性都差。

经过讨论，决定将导入 fallback 账户直接挂到标准 beancount 根账户下，并统一使用英文根名，使导出文件无需转换即可被标准 beancount 工具识别。

## Goals / Non-Goals

**Goals:**
- 移除 `Import/导入` 系统根账户和 `AccountType::Import` 枚举。
- 导入无映射账户的 fallback 路径使用标准 beancount 根名：`Assets`、`Income`、`Expenses`。
- 普通收支侧分录按金额符号归入 `Expenses:Import:<channel>:<category>`（正）或 `Income:Import:<channel>:<category>`（负）。
- 退款行（category 为 "退款" / "Refund"）归入 `Expenses:Import:<channel>:<category>`，金额为负，即“负的支出”。
- 账户映射 key 与账户路径使用同一套 beancount 根名：`Assets:`、`Income:`、`Expenses:`。
- beancount 导出不再做 `Import → Equity:Import` 转换。
- beancount 导入不再识别 `Equity:Import:` / `account_type: Import`。

**Non-Goals:**
- 不迁移或兼容旧数据库中已存在的 `导入:` 账户。
- 不引入 `Liabilities` 根账户。
- 不改变已映射账户的目标路径规则。
- 不修改支付宝适配器对原始 CSV 的解析逻辑（仅改变后续 fallback 路径构建）。

## Terminology

- **账户路径（account path）**：账户在树中的真实位置，用于记账、余额、beancount 导出。例如 `Expenses:Import:Alipay:餐饮美食`。
- **映射 key（mapping key）**：`account_mappings` 表中用于 `(member_id, channel_id, category) → account_id` 查找的字符串。它只包含角色前缀和原始分类，**不包含** `Import` 段和渠道名（渠道已作为 `channel_id` 单独存储）。例如 `Expenses:餐饮美食`。

两者关系：

```
映射 key      = <Role>:<原始分类>
账户路径      = <Role>:Import:<渠道名>:<原始分类>
```

## Decisions

### 1. 系统根账户统一使用英文 beancount 名称
- **选择**：种子数据在所有语言环境下都创建 `Assets`、`Income`、`Expenses`、`Equity` 作为根账户名，不再按语言分 `资产/收入/支出/权益`。
- **理由**：与 beancount 路径保持一致，避免导出时再翻译根名；同时让账户路径与映射 key 都基于同一套英文角色名，减少歧义。
- **影响**：中文 UI 里的根账户显示也会变成英文，符合用户“展示保持英文”的要求。

### 2. 按金额符号与退款标记区分 fallback
- **选择**：
  - `Asset` 角色：`Assets:Import:<channel>:<category>`。
  - `IncomeExpense` 角色且为退款（category 为 "退款" / "Refund"）：`Expenses:Import:<channel>:<category>`，金额为负。
  - `IncomeExpense` 角色金额 > 0（支出）：`Expenses:Import:<channel>:<category>`。
  - `IncomeExpense` 角色金额 < 0 且非退款（收入）：`Income:Import:<channel>:<category>`。
- **理由**：退款视为对支出的冲减，归入 `Expenses` 并以负数表示，符合“负的支出”的直观理解；普通收入/支出仍按金额符号区分。
- **替代方案**：把退款按金额符号归入 `Income`。用户明确要求退款映射到支出。

### 3. 映射 key 与账户路径使用同一套 beancount 根名
- **选择**：映射 key 采用 `Assets:<category>`、`Income:<category>`、`Expenses:<category>`，与 fallback 账户路径使用的根名完全一致。退款也使用 `Expenses:<category>`。
- **理由**：与 beancount 账户模型保持一致，避免再引入一套独立的概念；CLI 帮助和输出无需再做本地化映射 key 的翻译。
- **替代方案**：保留中文 key 或使用小写前缀。这会让映射 key 与账户路径不一致，增加用户理解成本。

### 4. 忽略旧 `Equity:Import` 备份
- **选择**：beancount 导入移除对 `Equity:Import:` 的识别；旧备份重新导入时，含有 `Equity:Import:` 的账户会按普通 `Equity:Import:` 路径创建（或报错，取决于根账户校验）。
- **理由**：用户明确选择不兼容旧数据，代码可以保持最小。
- **替代方案**：增加临时兼容层，把旧 `Equity:Import:<channel>:资产/收支:xxx` 翻译成新的 `Asset/Income/Expenses:Import:<channel>:xxx`。这会增加一次性复杂度，且旧数据最终还是要被放弃。

### 5. 使用 `ensure_cascading` 创建 fallback 账户
- **选择**：`ImportService` 仍通过 `AccountService::ensure_cascading` 创建 fallback 账户路径。
- **理由**：`ensure_cascading` 本来就接受任意系统根账户，`Assets`/`Income`/`Expenses` 都是系统根，无需引入新机制。

## Risks / Trade-offs

- **[Risk] 现有中文用户看到英文根账户名会不习惯** → Mitigation：这是本次重构明确接受的设计选择；后续如需本地化显示，可在 UI 层做别名，不改动账户路径。
- **[Risk] 旧 beancount 备份无法平滑重新导入** → Mitigation：用户已确认忽略旧数据；如需恢复旧备份，建议手动用文本替换把 `Equity:Import:` 改成对应标准根账户。
- **[Risk] 映射 key 变更导致已有 `account_mappings` 表数据失效** → Mitigation：新数据库重新初始化；旧数据不迁移。
- **[Risk] 收入/支出/退款分类依赖金额符号和 category 判断，未来若出现金额为 0 或 category 边界模糊会难以归类** → Mitigation：目前适配器不会产生这种情况；若未来出现，可在代码中显式断言/文档说明，默认非退款归入 `Expenses`。

## Migration Plan

1. 新数据库初始化时不再创建 `Import/导入` 根账户，根账户统一为 `Assets`、`Income`、`Expenses`、`Equity`（`Equity` 仍作为标准根保留，但退款不单独使用）。
2. 开发者/测试环境重新创建数据库；不编写旧数据迁移脚本。
3. 文档和 CLI help 中的示例路径同步改为英文。

## Open Questions

（暂无）

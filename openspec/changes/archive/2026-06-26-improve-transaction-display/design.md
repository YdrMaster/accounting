## Context

当前 `TransactionView.vue` 显示账单时，通过 `computeAmount()` 函数从 asset 类型的分录计算金额，通过 `getIncomeExpenseAccounts()` 和 `getAssetAccounts()` 提取账户名。对于从其他系统导入的账单，所有分录的 account_type 可能是自定义类型（如 "导入:"），不属于 asset/income/expense，导致计算结果为 0，显示误导用户。

现有数据结构已包含完整的分录信息（`TransactionDto.postings`），无需后端改动。

## Goals / Non-Goals

**Goals:**
- 纯导入账单不显示误导性的 ¥0.00 和空账户信息
- 所有账单支持展开查看完整分录列表
- 展开/折叠有平滑动画过渡
- 每个账单独立控制展开状态

**Non-Goals:**
- 不修改日分组/月汇总的统计逻辑（0 元贡献是正确的）
- 不修改后端 API 或数据模型
- 不做分录的编辑功能

## Decisions

### 1. 纯导入检测：基于 account_type 判断

检测函数 `isPureImport(tx)`：当账单的所有分录的 `account_type` 都不在 `['asset', 'income', 'expense']` 中时返回 true。

**理由：** 基于类型而非账户路径前缀判断，更健壮。即使将来导入账户的命名约定改变，只要类型系统正确，逻辑仍然有效。

### 2. 展开状态：使用 Set<number> 存储

用 `ref<Set<number>>` 存储当前展开的账单 ID。点击时 toggle ID 是否在集合中。

**理由：** Set 提供 O(1) 查找和去重，比数组更高效。独立状态管理符合需求（每个账单独立控制）。

### 3. 动画实现：CSS transition + max-height

使用 Vue `<Transition>` 组件配合 CSS `max-height` 过渡实现展开/折叠动画。

**替代方案：**
- `height: auto` 过渡：浏览器不支持原生动画，需要 JS 计算
- `scaleY` 变换：会导致内容变形

**选择 max-height 的理由：** 纯 CSS 实现，性能好，设置一个足够大的 max-height 值即可。

### 4. 分录显示：复用现有样式

分录列表使用与现有 tx-card 一致的样式风格，显示账户名（短名）、商品和金额。金额正负用颜色区分（绿/红）。

## Risks / Trade-offs

- [max-height 动画在内容很高时可能有轻微延迟] → 设置合理的 max-height 值（如 500px），过渡时间 0.3s
- [纯导入判断依赖 account_type 的正确性] → 这是后端导入逻辑的责任，当前系统已有明确的类型分类

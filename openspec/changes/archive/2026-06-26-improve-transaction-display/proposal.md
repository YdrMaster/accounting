## Why

导入的账单所有账户都属于"导入:"类型，无法区分资产和收支，导致 UI 计算金额时返回 0，显示误导性的"¥0.00"。需要改进显示逻辑，对纯导入账单隐藏无意义的统计信息，同时提供查看分录详情的能力。

## What Changes

- 添加纯导入账单检测逻辑：当账单不涉及任何 asset/income/expense 类型的分录时，判定为纯导入账单
- 纯导入账单不显示收支账户、金额、资产账户摘要，仅保留备注、成员和标签
- 所有账单（包括正常账单和纯导入账单）支持点击展开查看分录详情
- 分录显示区域使用平滑的展开/折叠动画
- 分录显示不显示"分录："标题行，直接展示分录列表
- 每个账单的展开状态独立控制

## Capabilities

### New Capabilities
- `transaction-entry-display`: 账单分录展开显示功能，包括纯导入账单检测、分录列表展示、展开/折叠动画

### Modified Capabilities

## Impact

- 前端代码：`accounting-web/src/views/TransactionView.vue` 需要重构显示逻辑
- 组件状态：需要添加展开状态管理（使用 Vue ref 存储展开的账单 ID 集合）
- 样式：需要添加分录显示区域的样式和动画过渡效果
- 无后端 API 变更，所有数据已在 TransactionDto 和 PostingDto 中提供

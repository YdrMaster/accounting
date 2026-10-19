## MODIFIED Requirements

### Requirement: 交易卡片删除入口
系统 SHALL 提供交易删除入口。触摸端 SHALL 支持在交易卡片上右滑触发删除，并在滑动过程中提供越过阈值的视觉反馈；非触摸端 SHALL 通过编辑表单底部的删除按钮触发删除（见 `transaction-form`）。删除操作 SHALL 先经确认对话框，且为硬删除、不提供撤销。

#### Scenario: 右滑删除
- **WHEN** 用户在触摸端交易卡片上右滑超过阈值（60px）
- **THEN** 弹出确认对话框

#### Scenario: 滑动过程反馈
- **WHEN** 用户在触摸端交易卡片上向右滑动
- **THEN** 卡片呈现滑动反馈，越过阈值时高亮提示将触发删除

#### Scenario: 非触摸端经编辑表单删除
- **WHEN** 用户在非触摸端打开某交易的编辑表单并点击底部删除按钮
- **THEN** 弹出确认对话框

#### Scenario: 确认删除
- **WHEN** 用户在确认对话框中点击删除
- **THEN** 调用 DELETE /api/transactions/:id，成功后从列表移除该交易

#### Scenario: 取消删除
- **WHEN** 用户在确认对话框中点击取消
- **THEN** 对话框关闭，交易保持不变

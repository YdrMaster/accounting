# transaction-form 增量

## MODIFIED Requirements

### Requirement: 交易表单字段
交易表单 SHALL 包含以下字段：日期时间、备注（可选）、成员选择、标签（可选多选）、渠道链路（可选）、分录列表。

#### Scenario: 必填字段
- **WHEN** 用户提交交易
- **THEN** 系统验证日期时间已填写、至少有两笔有效分录

#### Scenario: 备注字段
- **WHEN** 用户输入备注
- **THEN** 备注保存为交易描述（description）

#### Scenario: 成员选择
- **WHEN** 表单加载
- **THEN** 成员选择默认未选中（显示占位提示），用户必须手工选择成员后才能提交

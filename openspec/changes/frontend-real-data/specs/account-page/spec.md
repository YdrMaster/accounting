## ADDED Requirements

### Requirement: 新建账户入口
账户页 SHALL 提供新建账户按钮，点击后打开账户创建抽屉。

#### Scenario: 显示新建按钮
- **WHEN** 用户在账户页
- **THEN** 页面提供新建账户按钮

#### Scenario: 打开创建抽屉
- **WHEN** 用户点击新建账户按钮
- **THEN** 底部滑出创建抽屉，包含名称、账户类型、父账户字段

### Requirement: 创建账户
系统 SHALL 支持通过抽屉创建新账户。

#### Scenario: 填写创建信息
- **WHEN** 用户在创建抽屉中输入名称、选择账户类型和父账户
- **THEN** 表单实时验证名称不为空

#### Scenario: 提交创建
- **WHEN** 用户点击确认
- **THEN** 系统调用 POST /api/accounts 创建账户，成功后刷新账户列表并关闭抽屉

#### Scenario: 创建失败
- **WHEN** API 返回错误（如名称重复）
- **THEN** 抽屉内显示错误信息

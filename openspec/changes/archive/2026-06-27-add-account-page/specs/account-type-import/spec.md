## ADDED Requirements

### Requirement: Import 账户类型
系统 SHALL 支持 Import 类型的根账户，用于标记通过导入方式创建的账户。

#### Scenario: 创建 Import 类型根账户
- **WHEN** 系统初始化或手动创建 Import 类型的根账户
- **THEN** 账户的 account_type 为 Import，在账户页面中显示在"导入"栏下

#### Scenario: Import 类型账户的显示
- **WHEN** 账户页面渲染
- **THEN** Import 类型的根账户显示在"导入"栏目下，与其他类型账户并列展示

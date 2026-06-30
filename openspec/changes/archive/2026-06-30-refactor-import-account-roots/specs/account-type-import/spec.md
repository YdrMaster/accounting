# account-type-import

## REMOVED Requirements

### Requirement: Import 账户类型
系统不再支持 Import 类型的根账户。

#### Scenario: 系统初始化不再创建 Import 根账户
- **WHEN** 系统初始化种子数据
- **THEN** 不存在名为 `Import` 或 `导入` 的系统根账户

#### Scenario: 创建账户时无法选择 Import 类型
- **WHEN** 用户或代码尝试以 `Import` 作为根账户创建账户
- **THEN** 系统拒绝或报错，提示根账户类型无效

**Reason**: 导入 fallback 账户已合并到 `Asset / Income / Expenses` 标准根账户下，`Import` 类型与 beancount 模型不兼容，因此移除。

**Migration**: 新数据库直接使用 `Assets:Import:<channel>`、`Income:Import:<channel>`、`Expenses:Import:<channel>` 下的子账户；旧 `导入:` 数据不迁移。

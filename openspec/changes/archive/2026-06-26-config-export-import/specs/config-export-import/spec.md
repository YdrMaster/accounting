## ADDED Requirements

### Requirement: 配置导出范围
配置导出功能 SHALL 导出数据库中除账本数据以外的所有配置信息。账本数据包括 `transactions`、`postings`、`channel_paths`、`attachments`、`transaction_tags`，这些数据 SHALL NOT 被导出。

#### Scenario: 成功导出全部配置
- **WHEN** 用户执行 `accounting config export config.yaml`
- **THEN** 系统生成一个 YAML 文件，包含 `version`、`settings`、`commodities`、`members`、`channels`、`tags`、`accounts`、`account_owners`、`account_mappings`、`budgets`
- **AND** 文件中 SHALL NOT 包含 `transactions`、`postings`、`channel_paths`、`attachments`、`transaction_tags` 相关内容

### Requirement: 配置导入范围
配置导入功能 SHALL 读取符合格式的 YAML 文件，并将其中的配置合并到目标数据库。导入 SHALL 覆盖 `settings`、`commodities`、`members`、`channels`、`tags`、`accounts`、`account_owners`、`account_mappings`、`budgets`。

#### Scenario: 成功导入配置文件
- **WHEN** 用户执行 `accounting config import config.yaml`
- **THEN** 系统读取 YAML 文件中的配置
- **AND** 将配置按自然键合并到目标数据库
- **AND** 导入完成后目标数据库的账本数据保持不变

### Requirement: 导出文件使用自然键
导出的 YAML 文件 SHALL 使用自然键表示实体，SHALL NOT 包含数据库自增 ID。账户使用完整路径（如 `Assets:Bank:Checking`）表示；币种使用 `symbol`；成员、渠道、标签使用 `name`。

#### Scenario: 导出文件不包含数据库 ID
- **WHEN** 用户查看导出的 YAML 文件
- **THEN** 文件中不存在任何形如 `id: 123` 的数据库自增 ID 字段
- **AND** 账户以 `path: Assets:Bank:Checking` 形式出现

### Requirement: 导入按自然键合并更新
导入过程 SHALL 按自然键 upsert：若目标数据库已存在相同自然键的记录，则更新其属性；若不存在，则创建新记录。导入 SHALL NOT 删除目标数据库中已存在但 YAML 文件中未出现的配置记录。

#### Scenario: 合并更新现有配置
- **GIVEN** 目标数据库已存在 `commodity symbol = CNY`
- **WHEN** 导入的 YAML 中 `CNY` 的 `name` 发生变化
- **THEN** 目标数据库中 `CNY` 的 `name` 被更新
- **AND** 目标数据库中其他未在 YAML 中出现的 commodity 保持不变

#### Scenario: 新增配置记录
- **GIVEN** 目标数据库不存在 `commodity symbol = USD`
- **WHEN** 导入的 YAML 中包含 `USD`
- **THEN** 系统在目标数据库中创建 `USD` commodity

### Requirement: 账户路径作为唯一标识
账户的自然键 SHALL 为其完整路径。修改 YAML 中的账户路径 SHALL 被视为创建新账户，而非重命名现有账户。

#### Scenario: 修改账户路径产生新账户
- **GIVEN** 目标数据库已存在账户 `Assets:Bank:Checking`
- **WHEN** 导入的 YAML 中将该账户路径改为 `Assets:Bank:SalaryCard`
- **THEN** 系统创建新账户 `Assets:Bank:SalaryCard`
- **AND** 原账户 `Assets:Bank:Checking` 保持不变

### Requirement: 自动创建缺失的父账户
导入账户时，若其路径中的父账户不存在，系统 SHALL 自动创建所需的父账户。

#### Scenario: 父账户不存在时自动创建
- **GIVEN** 目标数据库仅存在 `Assets`
- **WHEN** 导入的 YAML 中包含 `Assets:Bank:Checking`
- **THEN** 系统自动创建 `Assets:Bank` 作为中间账户
- **AND** 创建 `Assets:Bank:Checking`

### Requirement: 忽略 is_system 标记
导出和导入 SHALL 忽略 `is_system` 标记。导出文件中 SHALL NOT 包含 `is_system` 字段；导入时 SHALL NOT 修改目标数据库中系统内置记录的 `is_system` 状态。

#### Scenario: 导出文件不包含 is_system
- **WHEN** 用户查看导出的 YAML 文件
- **THEN** 文件中不存在 `is_system` 字段

### Requirement: 导入时检查语言一致性
导入前 SHALL 检查 YAML 中 `settings.language` 是否与目标数据库当前语言一致。若不一致，或 YAML 中缺失 `settings.language`，系统 SHALL 拒绝导入并返回错误。

#### Scenario: 语言不一致拒绝导入
- **GIVEN** 目标数据库语言为 `zh-CN`
- **WHEN** 导入的 YAML 中 `settings.language` 为 `en`
- **THEN** 系统拒绝导入并提示语言不一致

#### Scenario: 缺失语言设置拒绝导入
- **WHEN** 导入的 YAML 中不包含 `settings.language`
- **THEN** 系统拒绝导入并提示缺少语言设置

### Requirement: 导入为原子操作
配置导入 SHALL 在单个数据库事务中完成。若导入过程中任何步骤失败，整个导入 SHALL 回滚，目标数据库状态保持不变。

#### Scenario: 部分失败回滚
- **GIVEN** 导入 YAML 中包含非法 budget limit 引用
- **WHEN** 系统执行导入并在处理 budgets 时失败
- **THEN** 之前已写入的 commodities、members、accounts 等数据 SHALL 被回滚
- **AND** 目标数据库保持导入前状态

### Requirement: 金额以字符串形式存储
YAML 文件中的金额数值（如 budget limits）SHALL 使用字符串表示，以避免浮点精度误差。

#### Scenario: budget limit 使用字符串
- **WHEN** 用户查看导出的 YAML 文件
- **THEN** budget limits 显示为 `"3000.00"` 等字符串形式
- **AND** 导入时系统使用定点数精确解析该字符串

### Requirement: 导出文件包含版本号
导出的 YAML 文件 SHALL 在顶部包含 `version` 字段，用于未来 schema 变更时进行兼容性处理。

#### Scenario: 导出文件包含 version
- **WHEN** 用户执行导出
- **THEN** 生成的 YAML 文件第一行或顶部包含 `version: "1.0"`

### Requirement: 闭包表自动重建
导入完成后，系统 SHALL 根据账户树重新计算并更新 `account_ancestors` 闭包表。

#### Scenario: 导入后闭包表正确
- **WHEN** 导入完成并提交事务
- **THEN** `account_ancestors` 表中的记录与导入后的账户树一致

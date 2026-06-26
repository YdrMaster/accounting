## 1. 基础准备

- [ ] 1.1 在 workspace `Cargo.toml` 中添加 YAML 序列化依赖（如 `serde_yaml` 或 `yaml-rust2`），并在 `accounting-service` 中引用。
- [ ] 1.2 在 `accounting-service/src` 下新建 `config` 模块，定义 YAML DTO 类型：`ConfigExport`、`ConfigImport`、`YamlCommodity`、`YamlMember`、`YamlChannel`、`YamlTag`、`YamlAccount`、`YamlAccountOwner`、`YamlAccountMapping`、`YamlBudget` 等，并为它们派生 `Serialize`/`Deserialize`。
- [ ] 1.3 定义 `ConfigService` 的 public 接口：`export(&self) -> Result<ConfigExport, Error>` 和 `import(&self, data: &ConfigImport) -> Result<(), Error>`。

## 2. 数据访问层补齐

- [ ] 2.1 在 `accounting-sql` 中为 `commodities` 补充 `commodity_list_all` 和按 `symbol` 查找/创建方法。
- [ ] 2.2 在 `accounting-sql` 中为 `members` 补充 `member_list_all` 和按 `name` 查找/创建方法。
- [ ] 2.3 在 `accounting-sql` 中为 `channels` 补充 `channel_list_all` 和按 `name` 查找/创建/更新方法。
- [ ] 2.4 在 `accounting-sql` 中为 `tags` 补充 `tag_list_all` 和按 `name` 查找/创建/更新方法。
- [ ] 2.5 在 `accounting-sql` 中为 `accounts` 补充 `account_list_all`、按完整路径查找/创建方法，以及根据路径自动创建父账户的辅助方法。
- [ ] 2.6 在 `accounting-sql` 中为 `account_owners` 补充列出全部关系和按 `(account_path, member_name)` 设置关系的方法。
- [ ] 2.7 在 `accounting-sql` 中为 `account_mappings` 补充列出全部映射和按 `(member_name, channel_name, category)` upsert 的方法。
- [ ] 2.8 在 `accounting-sql` 中为 `budgets` 和 `budget_limits` 补充列出全部预算和按 `name` upsert 的方法。
- [ ] 2.9 在 `accounting-sql` 中提供 `account_rebuild_ancestors` 方法，用于导入完成后重建闭包表。
- [ ] 2.10 在 `accounting-sql` 的 `SqliteDatabase` 上暴露上述新方法。

## 3. Service 层实现

- [ ] 3.1 实现 `ConfigService::export`：读取所有配置表，转换为 YAML DTO，组装 `ConfigExport`（包含 `version: "1.0"`）。
- [ ] 3.2 实现 `ConfigService::import`：开启数据库事务，按拓扑顺序执行导入。
- [ ] 3.3 实现导入前的语言一致性检查：读取目标数据库 `settings.language`，与 YAML 中的 `settings.language` 对比，不一致或缺失则返回错误。
- [ ] 3.4 实现引用解析：在导入 `channels`、`account_owners`、`account_mappings`、`budgets` 时，将自然键（路径、名称、symbol）解析为数据库 ID。
- [ ] 3.5 实现账户路径解析与自动创建父账户：给定 `Assets:Bank:Checking`，自动确认或创建 `Assets`、`Assets:Bank`、`Assets:Bank:Checking`。
- [ ] 3.6 实现导入完成后的事务提交与 `account_ancestors` 重建。
- [ ] 3.7 实现导入失败时的统一回滚与错误报告。

## 4. CLI 集成

- [ ] 4.1 在 `accounting-cli/src/cmd` 下新建 `config.rs`，定义 `config export <file>` 和 `config import <file>` 子命令。
- [ ] 4.2 实现 `export` 命令：调用 `ConfigService::export`，将结果序列化为 YAML 并写入指定文件。
- [ ] 4.3 实现 `import` 命令：读取指定 YAML 文件，反序列化为 `ConfigImport`，调用 `ConfigService::import`。
- [ ] 4.4 在 `accounting-cli/src/cmd/mod.rs` 中注册 `config` 子命令，并在 `main.rs` 的分发逻辑中加入 `Commands::Config` 处理。
- [ ] 4.5 为 CLI 添加适当的错误提示（如文件不存在、YAML 解析失败、语言不一致等）。

## 5. 测试

- [ ] 5.1 为 `ConfigService::export` 编写测试：准备内存数据库，插入配置数据，验证导出的 YAML DTO 结构和字段。
- [ ] 5.2 为 `ConfigService::import` 编写测试：从空数据库导入配置，验证各表是否正确创建；从已有数据库导入，验证合并更新行为。
- [ ] 5.3 为账户自动创建父账户功能编写测试。
- [ ] 5.4 为导入失败回滚编写测试。
- [ ] 5.5 为语言一致性检查编写测试。
- [ ] 5.6 运行 `cargo test` 全量测试，确保不破坏现有功能。

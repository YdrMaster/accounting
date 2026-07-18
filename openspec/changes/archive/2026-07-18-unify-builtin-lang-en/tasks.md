# Tasks: unify-builtin-lang-en

## 1. Schema 与名字表

- [x] 1.1 `accounting-sql/src/schema.rs`：新增六张同构名字表（`account_names`、`tag_names`、`channel_names`、`commodity_names`、`member_names`、`budget_names`），字段含 `lang`、`name`、`is_system`、`is_display`，外键级联删除；`UNIQUE(entity_id, lang, name)`
- [x] 1.2 六类实体表删除 name 列（`commodities.symbol` 保留）；相关唯一约束移除
- [x] 1.3 seed 重写：内置实体 + en/zh-CN 双语言系统名字（均设显示名，译名以 design.md 对照表为准）；`insert_seed_data` 删除 `lang` 参数与语言分支
- [x] 1.4 删除 `settings.language` 的写入与读取（`database.rs`、CLI `main.rs`、API `main.rs`），语言不再落库
- [x] 1.5 定义语言标签规范：`en` / `zh-CN`（`zh-*` 归一）/ `und`

## 2. 领域模型与 repo 层

- [x] 2.1 `accounting` crate：六类领域模型去 name 字段，名字改为独立 `EntityName` 类型 + 按语言解析接口
- [x] 2.2 `accounting-sql`：名字表 repo——按实体集合批量解析显示名（`CASE lang ... rowid LIMIT 1` 回退链），禁止逐条查询
- [x] 2.3 名字写入路径（创建/改名/加名字/设显示名/删名字）：命名空间唯一性校验（账户父级作用域，其余全局；`COLLATE NOCASE`）；系统名字不可删/不可改文本校验
- [x] 2.4 名字命中查询：任意名字、大小写不敏感；账户路径逐段按父级作用域命中
- [x] 2.5 改写所有引用实体 name 列的既有 repo 查询（账户、标签、渠道、币种、成员、预算）
- [x] 2.6 测试辅助函数：统一创建"实体 + 名字"的 fixture  builder，改写全部受影响测试

## 3. 服务层

- [x] 3.1 `import_service.rs`：pending 标签按系统名 `pending` 单名查询，删除 `待处理` 探测
- [x] 3.2 `channel.rs`：删除 `Alipay/alipay/支付宝` 硬编码别名表，渠道解析走名字表命中
- [x] 3.3 导入自动创建账户（`Expenses:Import:...`）的名字标注为 `und`
- [x] 3.4 mapping、报表等 service 中所有名字展示/解析接入新接口

## 4. API 与 CLI

- [x] 4.1 `accounting-api`：实体列表/详情接口增加语言参数（默认 `en`），响应返回解析后的显示名；错误信息等 rust_i18n 文案改用 `t!("key", locale = ...)` 按请求语言渲染，不再依赖进程级 `set_locale`
- [x] 4.2 `accounting-cli`：显示语言由 `--lang` 决定（默认 `en`），透传至显示名解析与 rust_i18n locale；账户/标签/渠道/成员/预算/币种的输入解析走名字命中
- [x] 4.3 CLI 创建/管理命令：支持为实体添加多语言名字、设置显示名（账户、标签优先，其余实体至少保证创建路径可用）

## 5. 导出与前端

- [x] 5.1 `accounting-beancount`：导出按当前显示语言取名；补充回归测试（任意语言名字可重新导入）
- [x] 5.2 `accounting-web`：安装 vue-i18n，建 `src/locales/{en,zh-CN}.ts`，语言切换入口 + localStorage 持久化
- [x] 5.3 前端 API 调用携带语言参数；实体名直接渲染 API 返回值，删除前端对内置名的任何假设
- [x] 5.4 按视图分批迁移硬编码中文 UI 文案到 i18n key
- [x] 5.5 更新前端测试 fixture 并补充语言切换测试

## 6. 验证

- [x] 6.1 `cargo test`（workspace）与 `cargo clippy` 全绿
- [x] 6.2 名字解析性能检查：交易列表/报表批量场景无 N+1
- [x] 6.3 Web 端测试与构建通过
- [x] 6.4 手动端到端：新库初始化（无语言参数，库内无语言设置）；CLI `--lang zh-CN` 显示 `资产:现金`、错误信息中文渲染，默认 `en`；输入 `现金`/`assets:cash` 均可命中；系统名不可删但可设非显示；Web 切换语言即时生效且刷新保持；beancount 导出/导入往返成功

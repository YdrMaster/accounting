# unify-builtin-lang-en

## Why

数据库 seed 的内置数据（系统子账户、系统标签、内置渠道）目前按建库语言二选一入库，语言成为"库的基本属性"，建库时一次性固化。这无法支持多人使用同一库且各自偏好不同语言的场景，并迫使代码出现双语探测逻辑（pending 标签双名查询、渠道别名表）。名字应当是"按语言管理的数据"，而不是实体的单值属性。

## What Changes

- 引入按语言管理的名字表：`accounts`、`tags`、`channels`、`commodities`、`members`、`budgets` 六类实体删除自身 name 列，全部名字由各自的名字表（`account_names`、`tag_names`、`channel_names`、`commodity_names`、`member_names`、`budget_names`）管理。每条名字记录标注语言、是否系统内置、是否为该语言的显示名。**BREAKING**：schema 结构性变更，不兼容存量数据库，不提供迁移。
- 输出语义：按选定语言取该语言的显示名；该语言无名字时按 英文 → 中文 → 其他语言（插入序）回退。
- 输入语义：不关心显示名，命中实体的任意名字即命中对象。命名空间：账户按父账户作用域（根账户为全局），其余实体全局；命名空间内所有名字不区分大小写不得重复；英文名字保存时保留大小写形式，匹配不分大小写。
- 系统内置实体为每种受支持语言（en、zh-CN）提供系统内置名字并设为显示名；系统名字不可删除，但可被设为非显示名（用户可用自定义名字替代显示）。
- 名字语言标注支持 `und`（未标注），用于导入账单自动创建的名字等无法可靠判定语言的场景。
- 语言完全由调用方参数决定：API 列表/详情接口引入语言参数，服务端按参数解析显示名、错误信息按请求语言渲染；CLI 以 `--lang` 决定显示语言（默认 `en`）；Web 语言选择持久化于 localStorage。**删除 `settings.language`**——语言不再是库的任何属性，初始化不再接收语言参数。
- Web 引入 vue-i18n 管理 UI 文案并支持语言切换（实体名由 API 按语言返回，前端不再维护内置名映射）。
- 消除语言耦合逻辑：`import_service` pending 标签双名探测改为按系统实体查询；渠道硬编码别名表由名字表取代。
- **BREAKING**：`builtin-channel`、`built-in-tags-exclude` 的"按语言种子"需求废止；`cli-natural-key-resolution` 的解析语义改为名字表命中。

## Capabilities

### New Capabilities

- `entity-names-i18n`: 六类实体（账户、标签、渠道、币种、成员、预算）的名字按语言存储与管理——名字表结构、每语言显示名与回退链、命名空间唯一性、系统名字保护、输入命中语义、API 语言参数、Web 端语言切换。
- `builtin-data-english-storage`: 内置实体及其系统名字的种子策略——实体与规范以英文为基准，各受支持语言提供系统名字；不提供存量库迁移。

### Modified Capabilities

- `builtin-channel`: 渠道不再按语言种子中文名；内置渠道为单实体 + 多语言系统名字。
- `built-in-tags-exclude`: 内置排除标签不再双语种子；为单实体 + 多语言系统名字。
- `cli-natural-key-resolution`: 自然键解析改为名字表命中语义（任意名字命中、命名空间作用域、大小写不敏感）。

## Impact

- **Schema**：六类实体表删 name 列，新增六张名字表；唯一性约束从实体表移到名字表（账户为父级作用域，应用层校验）。
- **代码**：`accounting-sql`（schema + 全部 repo 查询改写、批量显示名解析）、`accounting`（领域模型 name 字段改为按语言解析）、`accounting-service`（导入、映射等按 id 关联，去掉名字语言假设）、`accounting-api`（语言参数）、`accounting-cli`（显示/输入接入）、`accounting-web`（vue-i18n + API 语言参数）、`accounting-beancount`（导出按显示语言取名）。
- **API**：列表/详情接口增加语言参数；响应中的名字为已解析的显示名。
- **依赖**：`accounting-web` 新增 `vue-i18n`。
- **测试**：全部测试 fixture 改写（实体创建需同时写名字行）；前端 fixture 更新。
- **不迁移存量库**：老版本数据库无法用新版本打开，用户自行导出重建。

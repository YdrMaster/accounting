# Design: unify-builtin-lang-en

## Context

当前 `insert_seed_data(conn, lang)` 按建库语言在 ZH/EN 两套 seed 间二选一（`accounting-sql/src/schema.rs:61-79`），语言在建库时一次性固化进存储值。账户/标签/渠道等实体的 name 是单值列，从 DB 原样流经 CLI、API、beancount 导出，无翻译层。代码中存在双语探测（pending 标签、`Alipay/支付宝` 别名表）。Web 前端零 i18n 基础设施。这无法支持多人同库、各自偏好不同语言的场景。

经讨论确定的方向：名字是"按语言管理的数据"，而非实体的单值属性。语言是客户端启动时的选择，通过 API 语言参数传递，不是库的属性。

## Goals / Non-Goals

**Goals:**

- 六类实体（账户、标签、渠道、币种、成员、预算）的名字全部按语言存储，实体表不再保存名字。
- 输出按选定语言取显示名，带 英文 → 中文 → 其他（插入序）回退。
- 输入命中实体任意名字即命中对象；命名空间内不区分大小写唯一。
- 系统内置实体每种受支持语言有系统名字（不可删除、可设为非显示）。
- API 引入语言参数；CLI/Web 启动时决定显示语言。
- Web 引入 vue-i18n 管理 UI 文案。

**Non-Goals:**

- 存量数据库迁移（schema 不兼容，用户自行导出重建）。
- `attachments.filename` 等多语言化（文件工件，非显示名）。
- 账单原文数据（分类名等）的翻译。
- 币种符号多语言化（符号固定英文大写，仅币种"名称"多语言）。

## Decisions

### D1: 每实体一张名字表，而非统一名字表

六张同构名字表（以 `account_names` 为例）：

```sql
CREATE TABLE account_names (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    lang TEXT NOT NULL,              -- 'en' / 'zh-CN' / ... / 'und'
    name TEXT NOT NULL,
    is_system INTEGER NOT NULL DEFAULT 0,
    is_display INTEGER NOT NULL DEFAULT 0,  -- 每个 (account_id, lang) 至多一条为 1
    UNIQUE(account_id, lang, name)
);
```

**备选**：单张统一 `names(entity_type, entity_id, ...)` 表。否决——无法建立外键与级联删除，且账户的父级作用域唯一性与全局唯一性混在一表更难表达。六张表符合项目既有风格（`account_ancestors`、`transaction_tags` 等均为每实体关联表），Rust 侧用泛型/trait 共享查询逻辑消除重复。

唯一性索引只能表达"同一实体内不重复"；**命名空间唯一性（账户按父级、其余全局）由应用层校验**，因为账户作用域涉及 parent_id 无法用简单唯一索引表达。

### D2: 每语言一个显示名，回退链取名的输出语义

每个 (entity, lang) 至多一条 `is_display=1`；某语言有名字时必须恰有一条显示名。显示名解析：

```
resolve_display(entity, lang):
  1. 该 lang 的显示名
  2. 'en' 的显示名
  3. 'zh-CN' 的显示名
  4. 其余名字按插入序（rowid）取第一条
```

SQL 上可用 `ORDER BY CASE lang ... END, rowid LIMIT 1` 单次查询完成；列表场景按实体集合批量取回，禁止逐条查询（N+1）。

设为非显示名是用户操作（系统名字不可删但可降级）；当用户把某语言系统名降级且未指定新显示名时，该语言显示名空缺 → 走回退链，可接受。

### D3: 输入语义——任意名字命中，命名空间 + 大小写规则

- 命中实体任意名字（不限显示名、不限语言）即命中对象。
- 命名空间：账户为父账户作用域（根账户之间全局）；标签、渠道、币种、成员、预算为全局。
- 命名空间内所有名字不区分大小写唯一（SQLite 侧 `COLLATE NOCASE` 比较，覆盖 ASCII 大小写；中文不受影响）。
- 英文名字保存用户输入的大小写形式用于显示，匹配一律大小写不敏感。
- 账户路径解析：按 `:` 切段，逐段在当前父级作用域内做大小写不敏感命中。
- 创建/改名/加名字时校验命名空间唯一性；撞系统内置名（任何语言）同样拒绝。

### D4: 系统名字的保护与种子

系统实体（`is_system=1` 的账户/标签/渠道）seed 时为每种受支持语言（en、zh-CN）插入系统名字（`is_system=1`）并设为该语言显示名。系统名字不可删除、不可编辑文本，但可设为非显示名。删除保护沿用实体级 `is_system` 规则。

语言标签规范：英文用 `en`，中文用 `zh-CN`（`zh-*` 一律归一为中文档），无法判定语言的名字用 `und`。用户手动创建实体时，名字标注为其当前显示语言。

#### 内置名字对照表

实现与测试 SHALL 以下表为准，不得另译：

| 实体 | 路径/标识 | en | zh-CN |
|------|-----------|-----|-------|
| 根账户 | `Assets` | `Assets` | `资产` |
| 根账户 | `Equity` | `Equity` | `权益` |
| 根账户 | `Income` | `Income` | `收入` |
| 根账户 | `Expenses` | `Expenses` | `支出` |
| 子账户 | `Equity:OpeningBalances` | `OpeningBalances` | `期初余额` |
| 子账户 | `Expenses:Fees` | `Fees` | `手续费` |
| 子账户 | `Expenses:Discounts` | `Discounts` | `折扣` |
| 子账户 | `Expenses:InstallmentFees` | `InstallmentFees` | `分期手续费` |
| 子账户 | `Assets:Cash` | `Cash` | `现金` |
| 子账户 | `Assets:Cashback` | `Cashback` | `返现` |
| 标签 | `repayment` | `repayment` | `还款` |
| 标签 | `pending` | `pending` | `待处理` |
| 标签 | `exclude-from-income-statement` | `exclude-from-income-statement` | `不计收支` |
| 标签 | `exclude-from-budget` | `exclude-from-budget` | `不计预算` |
| 渠道 | `Alipay` | `Alipay` | `支付宝` |
| 币种名称 | `CNY` | `Chinese Yuan` | `人民币` |

根账户类型名（Asset/Equity/Income/Expense → 资产/权益/收入/支出）已有 `AccountType::display_name()` 机制，与上表保持一致。

### D5: 语言完全由调用方参数决定，废除 DB 语言设置

所有显示内容——实体显示名、UI 文案、错误信息——SHALL 由调用方传入的语言参数决定，数据库不再存储或参与决定显示语言：

- **删除 `settings.language`**：语言不再是库的任何属性。`insert_seed_data` 不再接收 `lang` 参数（seed 内容本就与语言无关）。
- **CLI**：`--lang`（全局参数）决定本次启动的显示语言，未提供时默认 `en`；rust_i18n 文案 locale 同样只跟 `--lang` 走。
- **API**：实体列表/详情接口接受语言参数（query param，如 `?lang=zh-CN`），未提供时默认 `en`。错误信息等 rust_i18n 文案按**每次请求**的语言渲染——`rust_i18n::set_locale` 是进程全局的，不能用于按请求切换，handler 须使用 `t!("key", locale = ...)` 显式传 locale。
- **Web**：语言选择持久化在 localStorage（客户端状态），作为 API 语言参数的来源。

这不同于被否决的"按 Accept-Language 翻译存储值"：存储本就多语言，服务端只是按参数选名，语义干净。

### D6: Web vue-i18n 只管 UI 文案

实体名全部来自 API 已解析的显示名；vue-i18n 负责界面文案（按钮、标签、提示）与语言切换入口，语言选择持久化 localStorage，并作为 API 语言参数的来源。既有硬编码中文文案按视图分批迁移。

### D7: beancount 导出用显示名，不追求稳定锚点

导出按当前显示语言取名。名字随显示偏好变化是可接受的——只要名字未被删除，含任意名字的导出文件都能重新导入（输入语义 D3）。名字被删除导致无法导入视为用户责任（与删除账户同级）。这消除了"规范名"特例，模型更纯粹。

### D8: 消除语言耦合逻辑

- `import_service.rs` pending 标签：改为按系统标签实体查询（id/系统名 `pending`），删除双名探测。
- `channel.rs` 的 `Alipay/alipay/支付宝` 硬编码别名表删除，由名字表命中天然取代。
- `AccountType::from_str` 的双语接受保留（根账户类型解析，属输入便利）。

## Risks / Trade-offs

- [N+1：列表/报表逐条解析显示名] → D2 明确批量取回；tasks 含性能检查点。
- [命名空间唯一性靠应用层校验，绕过 repo 直写 DB 可破坏] → 单用户 SQLite 场景可接受；校验集中在 repo 写路径。
- [测试 fixture 大面积改写（实体创建需附带名字行）] → 提供测试辅助函数统一创建"实体+名字"。
- [六张同构表的 schema 样板] → Rust 泛型共享逻辑；表结构保持完全一致。
- [混合语言路径的观感（如显示语言下部分实体只有 `und` 名字）] → 回退链保证总能显示，混合显示是可接受的已知行为。
- [老库完全无法打开] → 已在 proposal 声明 BREAKING，用户自行导出重建。

## Migration Plan

无数据迁移。实施顺序：schema 与名字表 repo → 领域模型与 service → API 语言参数 → CLI 接入 → beancount 导出 → Web vue-i18n → 全量验证。回滚即 revert。

## Open Questions

- API 语言参数的具体形式（query `lang` vs header）——倾向 query param，简单可见；实现时按 API 现有风格定。

## Context

当前导入系统将所有交易放入 Import 树下的扁平分类账户（如 `Import:支付宝:餐饮美食`），资产侧统一归入 `Import:支付宝`，丢失了"付款方式"信息。用户需要手动将 Posting 移动到正式账户树，这是导入流程的主要瓶颈。

现有 `Channel` 模型有 `account_id` 字段可关联默认资产账户，但映射系统不使用此字段做 fallback——只查映射表，有映射用映射，无映射走 Import。

Import 树当前结构为两层（`Import:支付宝:餐饮美食`），改造后变为三层（`Import:支付宝:收支:餐饮美食`），按角色分区。

现有支付宝适配器的金额方向存在错误：收支侧为负、资产侧为正，与复式记账惯例相反。本次改造一并修正。

## Goals / Non-Goals

**Goals:**
- 实现绑定在 (成员, 渠道) 上的分类字符串→账户编号映射表
- 支持收支侧和资产侧两类映射，映射 key 格式为 `"收支:<分类>"` / `"资产:<付款方式>"`
- 改造 BillPosting 结构，适配器输出 role + category 替代 account_path
- 导入时查询映射表，有映射用正式账户，无映射走 Import fallback
- Import fallback 路径增加角色层级：`Import:<渠道>:收支:<分类>` / `Import:<渠道>:资产:<付款方式>`
- 修正支付宝适配器的金额方向
- 提供 CLI mapping 子命令（set / list / delete）
- 删除账户时检查映射引用

**Non-Goals:**
- 不使用 Channel.account_id 做 fallback
- 不实现 Web API / UI（暂缓）
- 不改变"待处理" Tag 的行为——无论映射是否完全，所有导入交易仍挂此 Tag
- 不实现自动创建映射目标账户（set 时目标账户必须已存在）
- 不做通用化的映射系统（专门用于导入，不扩展到快速记账等场景）

## Decisions

### 1. BillPosting 用 role + category 替代 account_path

**选择**: `role: PostingRole` + `category: String`，替代 `account_path: String`

**替代方案**: 保留 account_path 但让适配器输出带角色前缀的路径（如 `"Import:支付宝:收支:餐饮美食"`）
- 缺点：适配器仍然需要知道 Import 根账户名称，耦合了路径组装逻辑

**理由**: 适配器只负责分类（从原始数据提取 category），不负责路径组装。路径组装和映射查询都是 ImportService 的职责，这样适配器更纯粹，新增渠道时只需关注数据提取。

### 2. 映射 key 格式为 `"角色:原始分类"`

**选择**: category 字段存储 `"收支:餐饮美食"` / `"资产:蚂蚁宝藏信用卡"` 格式

**替代方案 A**: 拆为两个 `TEXT` 列 `role` + `category_name`
- 缺点：增加一列，主键变四元组，增加了 DDL 和查询复杂度

**替代方案 B**: 只存原始分类，由应用层根据上下文区分角色
- 缺点：同一分类名可能同时出现在收支侧和资产侧（虽然少见），无法区分

**理由**: 前缀格式用单个 TEXT 列实现了角色标注，与现有映射查询的 key 构造逻辑自然对齐（`format!("{}:{}", role.prefix(), category)`），DDL 简洁。

### 3. 映射 set 使用账户路径字符串而非 ID

**选择**: `mapping_set` 接受 `account_path: String`，服务端 resolve → AccountId

**替代方案**: 直接接受 AccountId 数字
- 缺点：用户体验差，用户通常不知道账户 ID

**理由**: 路径字符串（如 `"Expenses:餐饮"`）是用户认知中的账户标识，CLI 也能直接传入。服务端通过 find_by_path 查找 AccountId，账户不存在则报错——不会自动创建。

### 4. 映射 set 为 upsert 语义

**选择**: 同一 (member_id, channel_id, category) 重复设置时覆盖 account_id

**替代方案**: 插入时若已存在则报错
- 缺点：用户需要先 delete 再 set，操作繁琐

**理由**: 映射是配置性质的数据，重复设置覆盖更符合直觉。使用 `INSERT OR REPLACE` 或 `ON CONFLICT ... DO UPDATE` 实现。

### 5. 删除账户时检查映射引用（RESTRICT）

**选择**: `account_id` 外键不声明 `ON DELETE CASCADE`，依赖默认 RESTRICT 行为；应用层在删除前检查并提供友好错误信息

**理由**: 如果级联删除映射，用户可能在不知情的情况下丢失映射配置。拒绝删除并提示用户先处理映射更安全。应用层检查提供中文错误信息，SQLite 外键 RESTRICT 作为兜底。

### 6. 金额方向修正

**选择**: 支出交易中，收支侧为正、资产侧为负；收入交易中，收支侧为负、资产侧为正

**当前错误**: 支付宝适配器中 `signed_amount` 逻辑导致收支侧为负、资产侧为正，与"支出账户增加、资产账户减少"的语义相反

**修正方式**: 在 `parse_alipay_row` 中调整 signed_amount 逻辑，确保收支侧金额符合账户类型的自然增减方向

## Risks / Trade-offs

- [数据兼容：Import 树结构变更] → 新导入交易进入 `Import:支付宝:收支:餐饮美食` 而非 `Import:支付宝:餐饮美食`，旧数据不受影响但两套结构共存。缓解：旧数据存量有限（开发阶段），可手动清理
- [映射配置成本] → 用户需要逐一配置映射，初期可能比手动移动 Posting 更慢。缓解：mapping list 让用户看到哪些 category 还没配置，逐步完善
- [支付宝适配器金额修正] → 已有测试按错误方向编写，修正金额后需同步更新测试。缓解：影响范围仅限 alipay.rs 及其测试
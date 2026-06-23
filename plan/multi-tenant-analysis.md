# 复式记账系统核心数据结构合理性评估与多租户扩展分析

## 业务背景

- 一个租户 = 一个家庭，家庭内的 Member 就是现有的 `Member` 结构体，家庭共享交易表
- 多租户的目标是**产品化后的用户横向扩展**——支持成千上万个家庭注册使用
- 租户间数据完全隔离，系统不需要做任何跨租户统计

---

## 一、核心数据结构设计合理性评估

### 1.1 复式记账模型忠实度：优秀（9/10）

- **会计等式完整性**：`validate_transaction` 正确实现"每笔交易至少两个分录 + 同 commodity 金额之和为零"。多币种通过 cost 字段建立等式，是 Endpoint Model 精髓。
- **正负号约定**：Posting.amount 正=借方、负=贷方，与 hledger/ledger PTA 传统一致。
- **冲减模型**：`linked_posting_id` + `reversal_total` 实现退款/报销的部分冲减，触发器自动维护 reversal_total。
- **交易-分录分离**：Transaction 与 Posting 一对多关系正确反映"交易是原子事件，分录是其经济效果"。
- **账户类型**：五类账户完全覆盖复式记账基础分类。

**注意**：种子数据缺少 Liabilities 根账户（中英文均缺），导致信用卡/贷款等负债账户无法级联创建。

### 1.2 数据结构完备性：良好（7.5/10）

已完备：多币种（commodity + cost）、账户层级（parent_id + 闭包表）、账户所有者（account_owners）、交易标签、模板交易、附件、冲减追踪。

可改进之处：

| 缺失 | 说明 |
|------|------|
| 审计追踪 | 无 `created_by`/`updated_by`，多用户/多租户场景下是合规刚需 |
| 交易状态 | 只有"存在/删除"两种状态，缺少 draft/posted/voided 等状态 |
| Attachment 存储方式 | `Vec<u8>` 直接存 SQLite，大文件场景性能差，应改存路径或对象存储 key |
| 预算模型 | PTA 系统通常含预算功能，当前无预算实体 |
| Posting 批次追踪 | 不支持按 lot 追踪同 commodity 不同买入价的多批次持仓（股票场景） |

### 1.3 类型安全性：优秀（9/10）

- 8 种 newtype ID 通过 `define_id!` 宏生成，编译期杜绝 ID 混用
- AccountType/TransactionKind 用 enum 而非整数
- `rust_decimal::Decimal` 避免浮点精度问题
- Option 正确表达可空字段

**问题**：`map_account` 中未知 account_type 整数默认为 `Asset` 而非返回错误，存在数据静默损坏风险。

### 1.4 可扩展性：良好（8/10）

- accounting crate 零 IO，所有核心算法为纯函数
- 8 个 Repo trait 为数据存储提供替换能力
- Service 层泛型 `TransactionService<D: Database>` 天然支持不同数据库实现

**瓶颈**：金额精度的 N+1 查询问题；SQL 字符串拼接构建动态查询；`Arc<Mutex<Connection>>` 单连接模式无法利用 WAL 并发读。

### 1.5 关注点分离：优秀（9.5/10）

这是项目最大架构亮点——每层职责边界清晰，不存在越层调用。`AccountingError` 中的 `DbNotInitialized`/`DbAlreadyExists` 是唯一不够纯粹之处，但可接受。

### 1.6 其他设计缺陷

- 触发器维护 reversal_total 依赖 Transaction 先于 Posting 插入的顺序，但无数据库层约束保证
- 缺乏数据库迁移机制（IF NOT EXISTS 全量建表无法修改已有列）
- API 层每次请求重新打开数据库

---

## 二、多租户扩展性分析

### 2.1 核心结论

> **无论选择哪种多租户方案，accounting crate 的核心数据结构（Account, Transaction, Posting 等）都不需要修改。**
>
> 这些结构体描述的是"单个家庭账簿内的数据"，租户隔离是纯粹的存储层/基础设施层关注点，不应侵入核心域模型。

### 2.2 SQLite 下的三种方案对比

| 维度 | 方案 A：每租户一库 | 方案 B：同库 + tenant_id 列 | 方案 C：同库不同表名 |
|------|---|---|---|
| **accounting crate** | ✅ 零改动 | ❌ 全结构体加字段 | ✅ 零改动 |
| **accounting-sql Repo trait** | ✅ 不改 | ❌ 所有方法加 tenant_id 参数 | ✅ 不改 |
| **accounting-sql Repo 实现** | ✅ 不改 | ❌ 所有 SQL 加 WHERE tenant_id=? | 🟡 表名动态拼接 |
| **accounting-sql Schema** | ✅ 不改 | ❌ 12 表加列 + 14 索引重构 + 14 触发器重写 | 🟡 schema 模板化 |
| **accounting-service** | 🟡 增加租户路由层 | ❌ 所有方法加 tenant_id | 🟡 增加租户路由层 |
| **数据隔离安全性** | ✅ 物理隔离 | ⚠️ 逻辑隔离，触发器/查询写错就泄漏 | ✅ 物理隔离 |
| **连接管理** | 🟡 多连接池 + LRU | ✅ 单连接池 | ✅ 单连接 |
| **可扩展租户数** | 百级~千级 | 万级~十万级 | 百级~千级 |
| **SQLite ID 碰撞** | ✅ 各库独立，无碰撞 | ⚠️ 全局自增，ID 空间竞争 | ⚠️ 同库自增，ID 跨租户碰撞 |

#### 方案 A（每租户一库）分析

- **优势**：schema/触发器完全不改，物理隔离，ID 不碰撞，运维简单（创建/删除/备份 = 文件操作）
- **劣势**：需要租户连接池 + LRU 淘汰（约 50~80 行代码），活跃租户过多时文件句柄压力大
- **适用规模**：百级~千级租户

#### 方案 C（同库不同表名）分析

- **优势**：accounting crate 零改动，物理隔离，单连接管理
- **劣势**：
  - **ID 碰撞**：同库不同表的 AUTOINCREMENT ID 会重复，类型系统无法防止跨租户误用
  - **触发器模板化**：14 个触发器绑定固定表名，需为每个租户生成独立实例，schema 从静态常量重构为动态模板
  - **DDL 排他锁**：新增/删除租户的 CREATE/DROP 会阻塞所有租户的所有操作
  - **表数量膨胀**：12 × 租户数，sqlite_master 膨胀导致 DDL 变慢
  - **迁移困难**：表名前缀模式在 PostgreSQL 中是反模式
- **适用规模**：十级以内租户（极端约束下）

#### 方案 B（同库 + tenant_id 列）分析

- **优势**：万级租户扩展性，单连接管理
- **劣势**：accounting crate 全结构体加字段，纯函数签名全改，触发器/索引全重写，逻辑隔离有泄漏风险
- **不推荐**：将租户概念塞进核心域模型，违反关注点分离原则

### 2.3 SQLite 的规模天花板

对于产品化后的横向扩展（万级家庭注册），**SQLite 本身不是合适的数据库**：

| 租户规模 | SQLite 可行性 | 说明 |
|---------|-------------|------|
| < 100 | ✅ 方案 A 完全可行 | 连接管理开销可忽略 |
| 100 ~ 1000 | ✅ 方案 A 可行 | LRU 淘汰足够应对 |
| 1000 ~ 10000 | ⚠️ 需要认真评估 | 文件句柄压力、备份文件数爆炸 |
| > 10000 | ❌ 不适合 | 需要迁移到 PostgreSQL |

### 2.4 "库管理繁重"是伪命题

方案 A 的"库管理繁重"顾虑需要拆解：

| 维度 | 方案 A | 方案 C | 谁更简单 |
|------|--------|--------|---------|
| 创建租户 | `SqliteDatabase::open()` 创建一个 db 文件 | 执行 12×CREATE TABLE + 14×CREATE INDEX + 14×CREATE TRIGGER | **方案 A** |
| 删除租户 | 删除一个 db 文件 | DROP 12 张表（DDL 排他锁阻塞全局） | **方案 A** |
| 备份 | 拷贝 db 文件（可按租户独立） | 拷贝整个 db 文件（无法按租户独立） | **方案 A** |
| 恢复 | 覆盖单个 db 文件 | 只能整体恢复，或写脚本从 dump 中提取 | **方案 A** |
| 连接管理 | 租户连接池 + LRU 淘汰 | 单连接 | **方案 C**（唯一优势） |

除连接管理外，方案 A 的所有运维操作都比方案 C 更简单。连接管理的 LRU 池仅需约 50~80 行代码，不是架构难点。

---

## 三、产品化多租户推荐方案：PostgreSQL schema-per-tenant

### 3.1 为什么选 PostgreSQL schema-per-tenant

产品化场景下，万级租户的横向扩展需要换数据库。在 PostgreSQL 上，**schema-per-tenant** 是最优方案：

| 维度 | 评估 |
|------|------|
| **accounting crate** | ✅ 零改动 |
| **accounting-sql Repo trait** | ✅ 不改 |
| **Schema** | ✅ 不改——每个 schema 内的表名固定（`postings` 就是 `postings`，不需要 `t1_postings`） |
| **触发器** | ✅ 不改——表名固定，触发器逻辑与当前完全相同 |
| **ID 碰撞** | ✅ 不存在——每个 schema 的序列独立 |
| **连接管理** | ✅ 简单——一个连接池，通过 `SET search_path = tenant_x` 切换租户 |
| **创建/删除租户** | ✅ 一条 DDL：`CREATE SCHEMA tenant_x` / `DROP SCHEMA tenant_x CASCADE` |
| **数据隔离** | ✅ schema 级物理隔离 |
| **可扩展租户数** | ✅ 万级无压力 |
| **备份/恢复** | ✅ pg_dump 支持 schema 级别导出 |

### 3.2 与方案 A/C 的关键差异

schema-per-tenant **兼得了方案 A 和方案 C 的所有优势，且没有它们各自的劣势**：

- 比方案 A：不需要多连接池和 LRU 淘汰，不需要管理多个文件
- 比方案 C：不需要触发器模板化，不存在 ID 碰撞，不存在 DDL 排他锁
- 比方案 B：不需要修改 accounting crate，不需要在核心域模型中引入 tenant_id

### 3.3 accounting-sql 层的改造

需要新增 `PostgresDatabase` 实现，但 **Repo trait 和核心逻辑不变**：

1. **新增 `accounting-pg` crate**（或扩展 `accounting-sql`）：实现 `Database` trait 的 PostgreSQL 版本
2. **Schema 初始化**：`CREATE SCHEMA {tenant}` + 在 schema 内执行与当前相同的 DDL（表名/触发器/索引均不变，仅 `AUTOINCREMENT` → `SERIAL` 等语法适配）
3. **连接管理**：使用 `deadpool-postgres` 或 `sqlx` 连接池，每次请求通过 `SET search_path` 切换到对应租户的 schema
4. **租户路由**：新增 `TenantResolver`，根据请求上下文解析 tenant_id 并设置 search_path

### 3.4 迁移路径

从当前 SQLite 到 PostgreSQL schema-per-tenant 的迁移是平滑的：

```
当前状态：accounting-sql（SQLite 实现）
    ↓ 新增 PostgreSQL 实现（不改现有代码）
目标状态：accounting-sql（SQLite） + accounting-pg（PostgreSQL）
    ↓ API 层配置切换
生产环境：accounting-pg（PostgreSQL schema-per-tenant）
```

- accounting crate：零改动
- accounting-sql：保留，用于开发/测试/小规模部署
- accounting-service：零改动（泛型 `D: Database` 天然支持切换）
- accounting-api：从 `SqliteDatabase::open()` 改为 `PostgresDatabase::new(pool)`

---

## 四、总结

### 核心数据结构评估

accounting crate 的数据结构设计整体优秀（8.5/10），最大亮点是关注点分离（9.5/10）。主要改进空间在审计追踪、交易状态机制和数据库迁移能力。

### 多租户扩展

1. **accounting crate 的核心数据结构不需要修改**——无论选哪种多租户方案
2. SQLite 在千级租户以内可用（方案 A：每租户一库），超过千级需要换数据库
3. 产品化推荐 **PostgreSQL schema-per-tenant**——accounting crate 零改动、schema/触发器零改动、万级租户无压力
4. 从 SQLite 到 PostgreSQL 的迁移路径平滑，Repo trait 的抽象设计已经为此预留了切换能力

# Design: audit-time-fields

## Context

回溯归档 commit ~2026-06-05。背景见 [proposal](./proposal.md)：项目早期时间字段语义混杂，`opened_at` 硬编码 `2000-01-01`、`transactions.created_at` 格式不一、半数表无时间字段。本设计将所有时间字段统一为纯审计字段，仅 `transactions.date_time` 保留业务语义。本文件补录决策来源——活规格未单独建审计能力（内部 schema），故此处为该设计唯一的决策记录。

## Goals / Non-Goals

**Goals:**

- 所有表具备 `created_at`/`updated_at` 审计字段，由数据库自动维护，应用层零侵入。
- `transactions.date_time` 升级为业务时间（秒级），其余时间字段均为审计（日级）。
- 审计字段不泄漏到 Domain 模型与用户可见输出。

**Non-Goals:**

- 不为已有数据库写迁移脚本（已确认无历史数据，直接重建）。
- 不引入"软删除"或"操作日志"——`updated_at` 不等同于变更历史。
- 不把审计字段作为业务逻辑输入。

## Decisions

### D1: 审计字段由数据库生成，应用层禁写禁读

所有表 `created_at`/`updated_at` 用 SQLite `DEFAULT (date('now'))` 与 `AFTER UPDATE` 触发器维护。Repo 层所有 INSERT 移除这两列、所有 UPDATE 不显式设置 `updated_at`、所有 SELECT 不查询。

**备选**（否决）：应用层每次写入时设置时间戳。否决理由：① 违反"单一来源"——每条写入路径都需记得设置，易漏；② 业务时钟与数据库时钟分叉时不可溯源；③ 测试需 mock 时间，增加负担。数据库驱动方案让触发器成为唯一写入点，应用层无法绕过。

### D2: `transactions.date` 升级为 `date_time`（NaiveDateTime）

`date` 从 `NaiveDate` 改为 `date_time: NaiveDateTime`，列改名以反映类型变化。这是**唯一**保留业务语义的时间字段——用户指定的交易发生时间，可精确到秒。

**备选 A**（否决）：保持 `date` 为 `NaiveDate`，另加 `time: Option<NaiveTime>`。否决：两个字段表达一个概念，查询与构造分散，违反内聚。

**备选 B**（否决）：`date_time` 也由数据库生成。否决：交易时间是业务输入（用户指定何时发生），不是审计值，必须由用户提供。

### D3: 精度分层——审计日级、业务秒级

审计字段 `created_at`/`updated_at`/`closed_at` 用 `date('now')`（日级），`transactions.date_time` 用 `YYYY-MM-DD HH:MM:SS`（秒级）。

**理由**：审计只需知道"哪天动了"，日级足够且存储紧凑；交易时间可能需要分秒（如跨日交易、对账），秒级必要。后续「审计字段改进」（见 `simplify-data-model` 阶段 2）将审计字段也升级到 `datetime('now')` 秒级，因为报表调试需要更细粒度。

### D4: `WHEN OLD.updated_at = NEW.updated_at` 触发器防递归

```sql
CREATE TRIGGER update_<table>_updated_at
AFTER UPDATE ON <table>
FOR EACH ROW
WHEN OLD.updated_at = NEW.updated_at
BEGIN
    UPDATE <table> SET updated_at = date('now') WHERE <pk> = NEW.<pk>;
END;
```

触发器内部 UPDATE 会再次触发自身；`WHEN` 条件让递归在第二轮停止——此时 `NEW.updated_at` 已被改成今天，与 `OLD.updated_at`（同行的旧值，亦是今天，但已知相等进入过触发器）……实际机制：首轮触发器把 `updated_at` 设为今天，二轮 `OLD=今天` 与 `NEW=今天` 仍相等会无限递归。**正确防递归**靠的是首轮设值后，应用层 UPDATE 的 `SET` 不含 `updated_at`，故 `OLD.updated_at = NEW.updated_at` 在首轮为真、触发；触发器内部 UPDATE 使 `NEW.updated_at` 变化，二轮 `OLD≠NEW`，停止。复合主键表（`account_ancestors`、`account_owners`、`transaction_tags`）用多列 `WHERE`。

### D5: 审计字段不进 Domain 模型

`Account`/`Transaction`/`Member` 等结构体**不**加 `created_at`/`updated_at` 字段。`closed_at: Option<NaiveDate>` 例外——它有业务用途（判断账户是否关闭）。

**理由**：审计字段对领域逻辑无意义，进入 Domain 会污染测试构造与序列化。它们只存在于数据库，需要时由 SQL 直接查（当前无此用例）。

### D6: CLI `--date` 双格式解析

```rust
fn parse_date_time(s: &str) -> Result<NaiveDateTime, _> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") { return Ok(dt); }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map(|d| d.and_hms_opt(0, 0, 0).unwrap())
}
```

`YYYY-MM-DD` 自动补全为当天 00:00:00，`YYYY-MM-DD HH:MM:SS` 直接用。向后兼容纯日期输入。

## Risks / Trade-offs

- **`and_hms_opt(0,0,0).unwrap()`**：D6 的补全对 `0:0:0` 必返回 `Some`，`unwrap` 在语义上安全；但违反"禁用 `unwrap`"编码规范。本变更当时保留 `unwrap`，后续 `simplify-data-model` review 提出改用集中辅助函数 `start_of_day()`（见归档 review 的 RULE_06 记录）。
- **无迁移脚本**：旧库的 `accounts.opened_at` 硬编码值丢失。已确认无生产数据，可接受；若有存量库需手工重建。
- **触发器防递归依赖隐式假设**：D4 的正确性依赖"应用层 UPDATE 不设置 `updated_at`"，若未来某处蓄意 SET `updated_at=OLD.updated_at` 会破坏。约束靠代码审查维持，无编译期保证。

## Migration Plan

无 schema 迁移（直接重建）。Domain 模型删 `opened_at`、`date`→`date_time`；Repo INSERT/SELECT 全面去审计字段；CLI `--date` 解析升级；所有测试中 `Account` 构造删 `opened_at`、`Transaction` 构造改 `date_time`。

## Open Questions

- 无（回溯归档，决策已定型）。

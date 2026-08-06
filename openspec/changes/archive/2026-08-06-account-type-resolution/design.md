# Design: account-type-resolution

## Context

账户类型（`AccountType` 4 变体）不在账户行上，靠「根账户 en 显示名 → `AccountType::from_str`」推导。关键现状：

- **推导路径**：`repo/account.rs:519` 的 `account_find_root_name(account_id, lang)` 沿闭包表找根账户名；`AccountType::from_str`（`accounting/src/account_type.rs:37`）按 en/zh 名字大小写不敏感匹配。
- **调用点**：
  - `report/mod.rs:48` `load_account_types`——预算/攒钱计划 create/update 校验用，**逐账户一次查询（N+1）**，拉全量账户。
  - `account_service.rs:~274` 账户关闭校验（单账户，一次查询）。
  - `report/cash_flow.rs:~105` 现金流量表分组（逐根账户查询，根账户仅 4 个，开销小）。
- **结构保护已有先例**：`account_service.rs:360` `cannot_move_root_account`——根账户不可移动，结构维度已受保护；名字维度无保护。
- 系统根账户：`parent_id IS NULL AND is_system=1`，en/zh 名均为 `is_system=1` 种子（schema.rs）。

## Goals / Non-Goals

**Goals:**

- `load_account_types` 批量化为单次查询往返，消除 N+1。
- 系统根账户的显示名不可修改（任何语言），类型推导锚点稳定。
- 对系统根账户的改名请求返回明确的本地化错误（CLI/API 一致）。

**Non-Goals:**

- schema 变更（不加 root_type 列、不动种子）。
- `AccountType::from_str` 的匹配规则改动。
- 非根系统账户（返现/折扣等内建账户）的改名策略（不在本次范围）。
- 现金流量表的根名调用点重构（4 个根账户、固定 en 种子，无 N+1 问题，保留现状）。

## Decisions

### D1: 改名保护而非 root_type 列

通过「禁止修改系统根账户显示名」让 en 根名成为稳定锚点，类型推导机制本身不变。

备选：accounts 表加 `root_type` 列（根账户写入、子账户创建时继承）——否决：需要 schema 变更与存量回写（my.db 虽为测试数据、迁移非硬性障碍，但零 schema 变更的方案仍然更小）；列方案的唯一增益是「连种子名都可改」，这不是需求。

### D2: 批量根名查询

SQL 层新增 `account_root_names_by_ids(account_ids, lang) -> Vec<(AccountId, String)>`：单条 SQL——`account_ancestors` 取各输入账户 depth 最大的祖先（根），join `account_names` 取指定 lang 的系统名（`is_system=1`，即种子写入的名）。`load_account_types` 改为一次调用 + 内存 `AccountType::from_str` 映射。

与旧逐账户路径的两处有意的语义差异（更安全方向，spec 场景已锁定）：

1. 旧路径经 `resolve_display` 有跨语言/非系统名回退，新路径严格 `is_system=1 AND lang`——类型推导只认种子系统根。（「自建根恰好命名为类型名」的分叉场景在实际中不可达：显示名按语言全局唯一（NOCASE），8 个类型名均已被种子占用，正常创建无法重名。）
2. 旧路径对无根名的账户整体报错，新路径静默跳过该账户（与 `load_account_types` 既有「无法推导不出现」语义一致，行为变宽松）。

实现形态二选一（执行时取简洁者）：`MAX(depth)` GROUP BY ancestor 的子查询，或 `depth = (SELECT MAX(...))` 关联子查询；账户量级下二者等价。

### D3: 保护点在 SQL 包装层

在 `database.rs` 的 `account_rename`（及 transaction.rs 对应方法）执行前检查目标账户是否为系统根（`parent_id IS NULL AND is_system=1`），是则返回 `DbError`（含语义明确的英文消息，上层映射为本地化错误）。放在 SQL 包装层而非 service：所有入口（CLI/API/service）共用此路径，一处拦截全覆盖。错误文案经现有错误映射惯例处理（参照「预算表不存在」模式：核心固定文案 + handler/CLI 本地化）。

### D4: 调用点切换范围

- 必改：`load_account_types` → 批量查询。
- 评估后保留：`account_service.rs` 账户关闭校验（单账户单查询，无 N+1）；`cash_flow.rs`（仅 4 个根账户查询）。
- 回归要求：预算/攒钱计划 create/update 的既有测试全部原样通过（行为不变，仅性能与错误语义变化）。

## Risks / Trade-offs

- [用户曾合法修改过根账户 en 名的存量库] → 该库本身已处于「类型推导失效」状态，本次变更不使其更糟；保护只防未来。
- [保护放在 SQL 层导致错误信息是英文底层文案] → 沿用既有错误映射模式（CLI/API 各自本地化）；文案固定含 "system root account" 便于上层匹配。
- [批量查询返回的根名仍依赖 is_system 种子完整性] → 种子由 initialize 幂等保证；改名保护封住唯一的破坏路径。

## Migration Plan

无 schema 变更、无数据迁移（用户已确认 my.db 为测试数据，无迁移负担）。既有库中若根账户名已被用户改过，本变更不修复其状态（属既存故障，需手工改回）；保护仅防新增破坏。

## Open Questions

- 无。

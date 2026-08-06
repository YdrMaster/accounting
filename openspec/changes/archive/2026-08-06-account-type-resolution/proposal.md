# Proposal: account-type-resolution

## Why

账户类型（Asset/Equity/Income/Expense）的判定目前依赖「根账户的英文显示名」推导（`account_find_root_name(id, 'en')` → `AccountType::from_str`），有两个已暴露的问题：

1. **脆弱性**：根账户的 en 显示名是用户可改的（`account rename --lang en`）。一旦被改成自定义文本，所有依赖类型判定的路径（预算/攒钱计划的账户校验、账户关闭校验、现金流量表分组）会集体失效，且报错（「账户不存在」）与真实原因完全不符，无法排障。
2. **N+1 查询**：`load_account_types`（`accounting-service/src/report/mod.rs`）对每个账户单独发一次根名查询，预算/攒钱计划 create/update 时全量加载，账户多时是明显的无谓开销。

该模式在 saving-plan 变更中被预算/攒钱计划校验继承，属于存量设计缺陷，单独立项修复。

## What Changes

- **类型解析批量化**：SQL 层新增单条查询（闭包表一次 join 取全部账户的根账户 en 系统名），`load_account_types` 从 N+1 改为一次往返。
- **根账户改名保护**：禁止修改 4 个系统根账户（`parent_id IS NULL AND is_system=1`）的显示名，使 en 根名成为稳定锚点，类型推导不再被用户改名破坏。（账户结构本就有 `cannot_move_root_account` 保护，本次补齐名字维度。）
- 受影响调用点统一切换到批量解析：`load_account_types`（预算/攒钱计划校验）、账户关闭校验、现金流量表分组。
- 不引入 schema 变更（备选方案 root_type 列已否决，见 design D1）。

## Capabilities

### New Capabilities

- `account-type-resolution`: 账户类型的批量解析（单 SQL 取根名）与系统根账户改名保护。

### Modified Capabilities

（无——预算/攒钱计划校验、账户关闭、现金流量表的行为不变，仅实现机制变化。）

## Impact

- `accounting-sql`：新增批量根名查询（repo + database/transaction 包装）；`account_rename` 增加系统根保护（或 service 层拦截）。
- `accounting-service`：`load_account_types` 改批量调用；账户关闭校验、现金流量表的根名调用点评估切换。
- `accounting-cli` / `accounting-api`：对系统根账户改名的请求返回明确错误（i18n 词条）。
- **兼容性**：试图重命名系统根账户的操作从「静默生效并埋雷」变为明确报错——属行为收紧，但该类操作本身就是故障源。
- **数据库**：无 schema 变更、无迁移。

## 备注

本提案由 saving-plan 终审沉淀（2026-07-31），2026-08-06 立项实施。

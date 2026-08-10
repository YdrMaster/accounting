# accounting-service

业务层：Service 封装与事务编排。位于分层架构第三层，依赖 [`accounting`](../accounting) 与 [`accounting-sql`](../accounting-sql)，为 `accounting-cli` / `accounting-api` 提供领域服务。

## 职责

- `AccountService`：账户创建/关闭/重开（含闭包表维护与级联操作）。
- `TransactionService`：交易提交/更新/删除（含核心库 `validate_transaction` 验证、退款/报销冲减分录的关联维护）。
- `CommodityService`、`TagService`、`MemberService`、`MappingService`、`ImportService`：各领域 CRUD 与导入。
- 报表（`report/`）：资产负债表、现金流量表、损益、每日汇总、净值趋势、攒钱计划、预算执行等。
- 配置（`config/`）与导入（`import/`）支持。
- 事务边界：写操作必走事务、读操作不走事务；服务层是事务编排的唯一入口。

## 设计文档

完整的服务接口、事务编排模式、错误处理、测试策略见 [`../spec/service.md`](../spec/service.md)。各报表能力的活规格见 `cash-flow-report`、`net-worth-trend-report`、`category-breakdown-report`、`transaction-summary-api`、`balance-sheet`、`budget-report`、`saving-plan-report` 等。数据模型精简重构的决策来源见归档 [`simplify-data-model`](../openspec/changes/archive/2026-06-23-simplify-data-model/design.md)。

## 分层上下文

见根 [`README.md`](../README.md) 的"分层架构"与"各 crate 文档"。

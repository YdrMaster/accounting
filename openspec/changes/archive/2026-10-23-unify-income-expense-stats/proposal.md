# Proposal: unify-income-expense-stats

## Why

同一个「收入/支出」概念，仓库里存在三套互不相同的口径，导致同一天、同一段时间的收支数字随页面而变：

- 日历页每日统计（后端 `posting_daily_summary`）：按**每条资产分录**正负拆分，纯转账被计成「收入 100 + 支出 100」。
- 交易列表日分组头 / 交易卡片（前端 `sumAssetAmount`）：按**每笔交易的资产分录净额**符号拆分，纯转账净额为 0、不计入。
- 交易页月收支汇总（前端 `TransactionView.vue`）：按 **Income/Expenses 分类账户**分录求和，与上述两套在期初余额、退款上又不同。

日历页与交易列表并排展示同一日的收支却给出不同金额（典型触发：转账，见 `transaction-transfer-and-delete` 中被显式标注「另行处理」的遗留项）。本变更把**收支汇总统计**统一到一套口径，并清理随之失效的死代码。

## What Changes

- 定义唯一收支口径 **「资产口径·逐交易净额」**（沿用交易列表现有算法）：对每笔交易取其全部资产类分录金额之和记为净额 `S`；`S > 0` 计入收入 `S`，`S < 0` 计入支出 `|S|`，`S = 0` 不计入（含纯转账）；非资产类分录不参与。
- 后端 `posting_daily_summary` 查询由「逐分录正负拆分」改为「逐交易净额拆分」，`GET /api/reports/daily-summary` 响应结构不变、数值口径改变。
- 前端交易页月收支汇总（月支出 / 月收入 / 月结余）由分类账户口径改为同一个 `sumAssetAmount` 口径，使页内月汇总与日分组头一致。
- 删除死代码 `PostingSummary` 结构与 `posting_summary` 查询（仅被单测引用；其唯一出口 `/api/reports/summary` 端点已在 `restructure-report-module` 中移除），同步删除 facade 声明与相关测试断言。
- 清理 `transaction-summary-api` 中已不存在的 `/api/reports/summary` 端点与 `summary` Service 的陈旧要求。
- **非目标**：现金流量报表（太阳图 / 收支变动明细）与预算执行报表按定义必须按 Income/Expenses 分类账户拆分到账户级明细、对账户设置限额，无法用资产口径表达，保持其分类账户口径不变（详见 design）。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `transaction-summary-api`: 收支汇总计算规则由「资产根逐分录正负拆分」改为「资产根逐交易净额拆分」，转账（净额 0）不再计入收支；移除已不存在的 `/api/reports/summary` 与此相关的 `summary` Service 要求。
- `transaction-list-ui`: 将月收支汇总纳入既有「金额计算规则」，明确月汇总与日分组头使用同一「资产账户分录逐交易净额」口径。

## Impact

- 后端：`accounting-sql/src/repo/posting.rs`（`posting_daily_summary` SQL、删除 `PostingSummary`/`posting_summary` 及其测试）、`accounting-sql/src/repo/facade.rs`（删除 facade 声明）、`accounting-service/src/report/daily_summary.rs`（文档注释与转账单测期望）。
- 前端：`accounting-web/src/views/TransactionView.vue`（月汇总改用 `sumAssetAmount`）。
- API：`GET /api/reports/daily-summary` 数值口径变化（请求/响应结构不变）；`/api/reports/summary` 不存在，无运行时影响。
- 数据：不涉及 schema 变更、无数据迁移。

## 1. 后端收支汇总 Service 和 API

- [x] 1.1 在 `accounting-service/src/report_service.rs` 新增 `summary` 方法，接受日期范围，查询资产类分录的收支汇总
- [x] 1.2 在 `accounting-api/src/handlers/report.rs` 新增 `summary` handler，解析 `from`/`to` 查询参数，调用 service
- [x] 1.3 在 `accounting-api/src/router.rs` 注册 `GET /api/reports/summary` 路由
- [x] 1.4 编译验证后端代码

## 2. 后端 DTO 扩展

- [x] 2.1 修改 `TransactionDto`，新增 `tags: Vec<String>` 字段
- [x] 2.2 修改 `TransactionDto`，新增 `member_name: Option<String>` 字段
- [x] 2.3 修改 `PostingDto`，新增 `account_type: String` 字段
- [x] 2.4 更新 `list_transactions` handler，填充新字段（查询标签名、成员名、账户类型）
- [x] 2.5 更新 `get_transaction` handler，填充新字段
- [x] 2.6 编译验证后端代码

## 3. 前端基础设施

- [x] 3.1 安装依赖：`pinia`、`decimal.js`、`@types/decimal.js`
- [x] 3.2 创建 `src/api/client.ts`，封装 fetch 请求（base URL、错误处理）
- [x] 3.3 创建 `src/types/api.ts`，定义后端 DTO 的 TypeScript 类型
- [x] 3.4 创建 `src/stores/transaction.ts`，Pinia store（交易列表、加载状态）
- [x] 3.5 创建 `src/stores/report.ts`，Pinia store（收支汇总）

## 4. 交易列表 UI 重构

- [x] 4.1 移除 TransactionView 中的硬编码假数据和月度预算卡片
- [x] 4.2 实现月收支汇总展示区域（从后端 API 获取，大字号月支出、小字号月收入和结余）
- [x] 4.3 实现按日分组逻辑（从交易列表按 date_time 分组，计算每日收支）
- [x] 4.4 实现交易卡片组件（收支账户、成员、备注、金额、资产账户）
- [x] 4.5 实现金额计算逻辑（普通交易 vs 转账交易）
- [x] 4.6 实现退款交易样式（灰色标题、绿色金额）
- [x] 4.7 实现标签展示（红色小圆角矩形）
- [x] 4.8 调整字号和样式匹配设计稿

## 5. 集成测试

- [x] 5.1 启动后端服务，验证 `/api/reports/summary` 端点
- [x] 5.2 启动后端服务，验证 API 返回新字段
- [x] 5.3 启动前端开发服务器，验证交易列表展示
- [x] 5.4 验证月收支汇总从后端获取并正确显示
- [x] 5.5 验证按日分组展示正确
- [x] 5.6 验证转账交易显示正确
- [x] 5.7 验证退款交易样式正确

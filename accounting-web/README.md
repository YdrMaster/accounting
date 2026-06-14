# accounting-web

基于 Vue 3 + Ant Design Vue 的 Web 记账前端，通过 HTTP API 与 `accounting-api` 交互。

## 技术栈

| 用途 | 选型 | 说明 |
|------|------|------|
| 框架 | Vue 3 | 组合式 API |
| 构建工具 | Vite | 开发服务器与打包 |
| 语言 | TypeScript | 类型安全 |
| UI 组件库 | Ant Design Vue 4 | 企业级 UI |
| 状态管理 | Pinia | 全局状态 |
| 路由 | Vue Router 4 | 单页路由 |
| HTTP 客户端 | axios | API 通信 |

## 快速开始

### 1. 启动 API 服务

Web 前端依赖 `accounting-api` 提供后端接口。先确保 API 服务已启动：

```bash
cargo run --bin accounting-api -- --db my.db --port 3000
```

API 服务启动时会自动完成数据库初始化（schema + seed 数据），无需先用 CLI 初始化。

可以通过 `--lang` 指定初始化数据库时使用的语言（影响内置账户、标签等 seed 数据的名称），默认从环境变量 `LANG` 检测，回退为 `en`：

```bash
cargo run --bin accounting-api -- --db my.db --port 3000 --lang zh-CN
```

### 2. 安装依赖

```bash
cd accounting-web
npm install
```

### 3. 开发模式

```bash
npm run dev
```

默认访问 <http://localhost:5173，Vite> 开发服务器会自动代理 `/api` 请求到 `http://localhost:3000`。

### 4. 生产构建

```bash
npm run build
```

构建产物输出到 `dist/` 目录。`accounting-api` 启动时通过 `--static-dir` 参数指定该目录即可一并提供前端静态文件服务：

```bash
cargo run --bin accounting-api -- --db my.db --static-dir accounting-web/dist --lang zh-CN
```

## 页面功能

### 首页（Dashboard）

- **日历视图**：按月展示每日收支，支持单日期选择与日期范围选择
- **当月概览**：收入、支出、结余统计
- **交易列表**：按日期分组展示交易，支持展开查看分录详情
- **筛选**：按账户、成员、标签、关键词筛选交易
- **快捷记账**：选中日期后可直接跳转记账页面

### 记一笔（TransactionForm）

- **创建交易**：填写日期、描述、分录（账户 + 货币 + 金额）
- **多币种支持**：换汇交易自动识别 cost
- **标签与渠道**：为交易添加标签和支付渠道
- **成员关联**：自动关联当前选中的成员
- **编辑/删除**：支持修改已有交易或删除

### 账户（AccountTree）

- **账户树**：层级展示所有账户（Assets / Liabilities / Equity / Income / Expenses）
- **创建账户**：输入 `:` 分隔的完整名称自动级联创建父级（如 `Assets:Bank:Card`）
- **设置所有者**：为账户指定归属成员
- **账单日/还款日**：信用卡等账户可设置账单日与还款日

### 标签与渠道（Tags）

- **标签管理**：添加、删除自定义标签
- **渠道管理**：添加、删除支付渠道（如支付宝、微信、银行卡）

### 报表（Reports）

- **资产负债表（BS）**：按账户类型汇总资产与负债
- **损益表（IS）**：收入与支出汇总

## 全局功能

### 成员切换

侧边栏（PC）或底部抽屉（移动端）可切换当前成员。所有新交易默认关联当前成员。支持动态添加新成员。

### 主题切换

支持亮色 / 暗色主题，切换后自动同步 Ant Design Vue 主题与自定义样式。

### 响应式布局

- **PC**：左侧固定侧边栏导航
- **移动端**：底部标签栏导航，顶部显示当前成员

## 开发配置

### Vite 代理

开发环境下，`vite.config.ts` 已配置代理，将 `/api` 请求转发到后端：

```typescript
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:3000',
      changeOrigin: true,
    },
  },
}
```

### 环境变量

如需修改 API 地址，可在项目根目录创建 `.env.local`：

```
VITE_API_BASE_URL=http://localhost:3000/api
```

并在 `src/api/client.ts` 中调整 `baseURL`。

## 与 CLI 的关系

`accounting-web` 与 `accounting-cli` 共用同一个 SQLite 数据库文件，通过 `accounting-api` 访问数据。你可以在 CLI 中录入交易，然后在 Web 中查看报表；反之亦然。

```bash
# CLI 录入（注意 CLI 二进制名为 accounting-cli）
cargo run --bin accounting-cli -- my.db tx add \
  --date 2024-06-01 \
  --description "午餐" \
  --posting "Assets:Cash:-50;Expenses:Food:50"

# Web 查看报表
open http://localhost:5173/reports
```

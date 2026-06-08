# accounting

一个基于 Rust + SQLite 的复式记账系统，采用 PTA（Plain Text Accounting）风格的端点图模型。

## 设计思路

### 核心模型：Kleppmann 端点图（Endpoint Model）

每一笔交易由一组 **Posting（端点/分录）** 组成，每个 Posting 描述一笔资金从"某处"流向"某处"。与 T-account 或 Pacioli 分组模型不同，端点模型天然支持多币种和复杂的资金流动追踪。

系统中没有独立的"换汇"概念——多币种交易通过 `cost` 字段表达双边等式。例如，买入 100 USD 花费 678 CNY：

```plaintext
Assets:Foreign:USD    +100 USD  (cost = 678 CNY)
Assets:Cash           -678 CNY
```

> 注：数据库存储时按 commodity 的 precision 缩放为整数（如 CNY precision=2 时，678 CNY 存储为 67800）。

### 分层架构

```plaintext
accounting-cli      ← CLI 入口（clap + tokio + tabled）
    ↓
accounting-service  ← 业务层（Service + 事务编排）
    ↓
accounting-sql      ← 数据库层（Repository traits + SQLite）
    ↓
accounting          ← 核心库（纯数据模型 + 算法）
```

- **核心库（accounting）**：定义数据模型（Account, Transaction, Posting 等）和纯算法（交易验证、余额计算、闭包表计算），零 IO，可独立测试
- **数据库层（accounting-sql）**：Repository 模式 + SQLite 实现。Schema 严格关系化（11 张表），账户层次通过 **闭包表（closure table）** 维护，支持高效的后代聚合查询
- **业务层（accounting-service）**：Service 封装事务边界。AccountService 处理账户创建/关闭/重开（含闭包表维护和级联操作），TransactionService 处理交易提交/更新/删除（含核心库验证）
- **CLI（accounting-cli）**：面向用户的命令行接口，支持 `--format table|json` 两种输出

### 关键设计决策

| 决策 | 说明 |
|------|------|
| **SQLite** | 单机使用，零运维成本，严格关系化 schema |
| **闭包表** | `account_ancestors` 表维护账户层次，支持 `O(1)` 后代查询 |
| **硬删除** | 交易和分录采用级联硬删除，符合复式记账的审计要求 |
| **内置节点** | `Equity:OpeningBalances`, `Income:Uncategorized` 等 7 个系统账户 + `repayment` 标签 |
| **精度动态化** | Posting 金额按 commodity 的实际 precision 缩放存储 |

## CLI 用法

详见 [`accounting-cli/README.md`](accounting-cli/README.md)。

## Web 用法

详见 [`accounting-web/README.md`](accounting-web/README.md)。

## Workspace 结构

```plaintext
.
├── accounting/          # 核心库：模型 + 算法
├── accounting-sql/      # 数据库层：Repository + SQLite
├── accounting-service/  # 业务层：Service + 事务
├── accounting-cli/      # CLI 入口
├── accounting-api/      # HTTP API 服务（axum）
├── accounting-web/      # Web 前端（Vue 3 + Ant Design Vue）
├── spec/                # 设计文档
├── plan/                # 实现计划
└── docs/                # 项目文档
```

## 快速开始

```bash
# 编译
cargo build --release

# 初始化数据库
./target/release/accounting my.db initialize

# 创建账户
./target/release/accounting my.db account add Assets:Cash --type asset
./target/release/accounting my.db account add Expenses:Food --type expense

# 记账
./target/release/accounting my.db tx add \
  --date 2024-06-01 \
  --description "午餐" \
  --posting "Assets:Cash:-50;Expenses:Food:50"

# 查余额
./target/release/accounting my.db report bs
```

## 技术栈

- Rust Edition 2024
- rusqlite（SQLite）
- rust_decimal（精确金额计算）
- chrono（日期处理）
- clap（CLI）
- tokio（async runtime）
- tabled（表格输出）

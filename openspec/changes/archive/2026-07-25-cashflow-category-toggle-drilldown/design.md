# cashflow-category-toggle-drilldown 设计

## Context

现金流量 tab 目前由两类数据支撑：`/api/reports/category-breakdown`（收支账户各层级汇总，喂两张并排旭日图）和 `/api/reports/cash-flow`（资产账户 inflow/outflow/net，喂底部表格）。两个报表口径并存、数据重复；cash-flow 的 inflow/outflow 还是按净额符号伪拆分的。前端旭日图树靠解析显示名路径字符串（`split(':')`）构建，钻入点击事件未外抛。

约束：

- 旭日图数据与其他控件、后端关联一律使用账户序号 id，禁止使用名字或路径字符串做逻辑关联。
- 聚合查询沿用现有 `sum_by_account_with_descendants`（已在 category-breakdown 中验证）。
- 排除"不计预算"标签的口径不变。

## Goals / Non-Goals

**Goals:**

- 单一"收支变动"报表：Income/Expenses 两根下各层级账户的周期金额汇总，一个端点同时喂旭日图与详情列表。
- API 明细项携带 `account_id` / `parent_id`，前端按 id 建树、按 id 联动。
- 前端单页旭日图 + 支出/收入 toggle；钻入与树状详情列表联动。
- CLI cash-flow 命令同口径。

**Non-Goals:**

- 不改动资产趋势图、资产负债表等其他 tab 内容。
- 不引入流入/流出拆分（退款按净额自然冲抵）。
- 不做列表行展开/收起交互（树状全量缩进展示）。

## Decisions

### D1：重定义 cash-flow 吞并 category-breakdown，而非反向

保留 `cash_flow.rs` 文件名与 `/api/reports/cash-flow` 端点名（CLI 命令名、路由不变，降低改名面），内部逻辑替换为 category-breakdown 的聚合方式；删除 `category_breakdown.rs` 与 `/api/reports/category-breakdown`。

替代方案：保留 category-breakdown 删除 cash-flow——路由与命令都要改名，调用方改动面更大，弃。

### D2：数据结构——各层级单金额（绝对值），废掉 inflow/outflow/net

```rust
pub struct CashFlowItem {
    pub account: Account,   // 含每一层祖先的汇总行
    pub amount: Decimal,    // 周期内净额绝对值
}

pub struct CashFlowReport {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub income: Vec<CashFlowItem>,   // Income 根下各层级
    pub expense: Vec<CashFlowItem>,  // Expenses 根下各层级
}
```

收支账户的单向性（支出为正、收入为负、退款/冲正反向净额冲抵）使 inflow/outflow 拆分没有展示场景（截图布局也只有单金额），同时顺带消除原有的伪拆分失真。

### D3：API DTO 以 id 为关联键，name 仅展示

```jsonc
// GET /api/reports/cash-flow
{
  "period_start": "2026-07-01",
  "period_end": "2026-07-31",
  "income":  [{ "account_id": 12, "parent_id": 3,  "name": "工资", "amount": "15000.00" }],
  "expense": [{ "account_id": 20, "parent_id": 4,  "name": "餐饮", "amount": "3200.00" },
              { "account_id": 21, "parent_id": 20, "name": "外卖", "amount": "1800.00" }]
}
```

- `account_id`：旭日图节点 id、钻入事件载荷、列表筛选键。
- `parent_id`：前端 `Map<id, node>` 链接成树的唯一依据；根账户的 `parent_id` 为 `null`。
- `name`：后端按请求语言解析（沿用现有回退链）的叶名，仅用于渲染。

替代方案：继续返回路径字符串——同名账户、名字含冒号、多语言切换都会断链，弃。

### D4：前端按 id 建树，钻入状态 = 账户 id

- `utils/sunburst.ts` 改为消费 `{ account_id, parent_id, name, amount }[]`：`Map<id, node>` + parent_id 链接；保留 1% 扇区过滤、伪子节点（中间层直接分录）等既有行为，伪子节点继承父 id 加标记（不参与钻入）。
- `CategorySunburst` 节点 `data` 携带 `account_id`，监听 echarts click 事件 `emit('drill', account_id | null)`（点中心返回时抛父级 id 或 null）。
- `CashFlowPanel` 持有 `side: 'expense' | 'income'` 与 `drillId: number | null`；toggle 切换、周期/日期变化时 `drillId` 重置。
- 详情列表：以 `drillId ?? 该侧根id` 为根，输出其自身 + 各级后代，树状缩进，每级按金额降序；百分比与比例条相对当前层级兄弟总额（根级相对该侧总额）。图表 1% 过滤只影响扇区渲染，列表显示全部后代（含 <1% 账户）。

### D5：CLI 同步口径

`accounting-cli/src/cmd/report.rs` cash-flow 输出改为分 Income/Expenses 两节、树状缩进、各层级金额，文案走现有 locales。

## Risks / Trade-offs

- [API breaking：旧前端/外部调用方解析失败] → Web 前端与 API 同仓库同步发布；CLI 同步改。
- [伪子节点（中间层直接分录）钻入语义模糊] → 伪子节点不响应钻入，仅作视觉补全；其金额体现在父账户汇总行。
- [id 关联依赖响应中 parent_id 完整覆盖] → 后端对 Income/Expenses 两根全层级返回（现有聚合本就如此），spec 中固化为要求。

## Migration Plan

后端服务 + API + CLI + 前端一次性同构切换，无数据迁移；旧端点 `/api/reports/category-breakdown` 随发布直接移除。

## Open Questions

（无）

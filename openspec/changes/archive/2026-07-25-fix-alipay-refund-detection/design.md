# Design: 支付宝导入按交易状态判定退款

## Context

支付宝账单中"退款成功"行的交易分类有两种形态：约 2/3 为字面"退款"，约 1/3 为真实消费分类（餐饮美食、日用百货等，本质是部分退款/售后退款的原始分类）。当前适配器以 `category == "退款"` 判定，漏掉后者；且所有退款成功行的"收/支"列为"不计收支"，漏判后落入不计收支分支按支出记账，方向完全错误。

下游 `PostingRole::to_key` / `fallback_root` 通过 `is_refund_category`（分类名嗅探）决定退款是否使用 `Expenses` 根——退款语义是"负支出、冲减消费"，不能走 `Income` 根。判定依据改为交易状态后，分类名嗅探失去信息来源（真实分类不含"退款"二字），必须把退款标记显式化。

## Goals / Non-Goals

**Goals:**

- 所有 `交易状态 == "退款成功"` 的行被识别为退款：收支侧金额为负（负支出），资产侧金额为正
- 退款行使用 `Expenses` 根生成映射 key 与 fallback 路径，无论分类名是什么
- 既有映射 key 不变：`Expenses:退款`（旧形态）与 `Expenses:<真实分类>`（与误判期的 key 一致）

**Non-Goals:**

- 不处理"还款成功/失败""提现"等其他状态的语义细化
- 不修正历史已导入的错误数据（用户自行调整，或另开 change 做迁移工具）
- 不改适配器其他字段的解析逻辑

## Decisions

### 决策 1：精确匹配 `status == "退款成功"`

两份真实账单中状态值干净（无"部分退款成功"等变体），精确匹配简单且无歧义。`contains` 反而可能误匹配未来出现的复合状态。

### 决策 2：`BillPosting` 增加 `is_refund: bool`，删除 `is_refund_category` 嗅探

退款是源数据的事实属性，应由适配器在解析时显式标记，而不是让下游从分类名猜测。`BillPosting` 只有 alipay 一个适配器构造（`alipay.rs` 两处），`to_key`/`fallback_root` 只有 `import_service.rs:302/315` 两个调用点，签名变更影响面可控。`is_refund_category` 只服务旧判定路径，删除避免"退款"二字在系统里存在两种语义来源。

### 决策 3：签名改为 `to_key(category, amount, is_refund)` / `fallback_root(amount, is_refund)`

`fallback_root` 的 IncomeExpense 分支规则变为：`is_refund || amount > 0 → Expenses`，否则 `Income`。`to_key` 同步传递。`fallback_root` 不再需要 `category` 参数（实现时移除）。保持纯函数、易测试。

### 决策 4：日期解析兼容新版导出格式（实现中追加）

试导入发现 2026 年新版支付宝账单日期为斜杠、月日不补零、无秒（`2026/7/25 11:43`），旧解析仅支持带秒格式，整份文件 270 行全部被跳过。经用户确认并入本 change：`parse_datetime` 按候选格式列表依次尝试（短横线/斜杠 × 带秒/无秒）。

## Risks / Trade-offs

- [未来支付宝出现新的退款相关状态（如"部分退款成功"）漏判] → 精确匹配是用户确认的决策；若出现新状态，在适配器加一个状态常量即可，改动点集中。
- [历史已误导入的 19 行数据仍为错误方向] → 本 change 不迁移历史数据；在 tasks 中提示用户手动检查调整。
- [其他适配器将来构造 BillPosting 需理解 is_refund] → 字段语义简单（源数据是否退款事件），文档注释说明即可。

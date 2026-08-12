# Tasks: transaction-card-collapsed-redesign

## 1. 后端 API：pending 判定字段

- [x] 1.1 `accounting-api` 的 `TransactionDto`（`dto.rs`）新增只读 `pending: bool` 字段
- [x] 1.2 `handlers/transaction.rs` 组装 DTO 处计算 `pending`：交易关联系统 `pending` 标签时为 true，否则 false（复用系统标签解析路径，`tx.tags` 保持本地化显示名不变）
- [x] 1.3 更新 `accounting-api` 交易 handler 测试，覆盖「有关联系统 pending 标签的交易 pending=true」「无标签/普通标签支付交易 pending=false」

## 2. 前端类型与卡片组件

- [x] 2.1 `accounting-web/src/types/api.ts` 的 `TransactionDto` 增加 `pending: boolean`
- [x] 2.2 `TransactionCard.vue` 折叠态重写为两行锚定结构；描述为空时合并为单行、摘要充当主标题
- [x] 2.3 实现摘要对位计算：按 `posting.amount` 符号分侧（非正值 `、` 连接于左 / 正值 `、` 连接于右，中间 ` → `，账户取短名 leaf）；移除 `isTransfer` / `isPureImport` / `getIncomeExpenseAccounts` / `getAssetAccounts`
- [x] 2.4 实现主标题回退链：`描述 → 账户摘要`
- [x] 2.5 实现 pending 琥珀渐变：`tx.pending` 为真时卡片背景 `linear-gradient(90deg, rgba(245,158,11,0.20), transparent 65%)`，文字色不变，无「待处理」文字徽标
- [x] 2.6 实现多币种金额符号：`postings` 币种仅一种时金额无符号；多种时金额带主币种符号（取 CommodityDto.symbol，缺省回退币种代码）、次要币种代码标于摘要行尾
- [x] 2.7 单行溢出保护：主标题列 `min-width: 0` + `text-overflow: ellipsis`，保证长文本不把金额/展开指示挤出视口
- [x] 2.8 确认展开态、双击编辑、左右滑手势、展开动画行为不变

## 3. 测试与清理

- [x] 3.1 新增/更新 `TransactionCard` 组件测试：普通收支（两行）、描述空（合并单行）、转账摘要对位（`工商 → 招行`）、pending 渐变类名、多币种符号、金额必现（含 `:Import:` 分录）
- [x] 3.2 检查全库确认无 `isPureImport` / `isTransfer` 残留引用，清理相关断言
- [x] 3.3 运行 `accounting-api` 与 `accounting-web` 测试套件及 lint，全部通过

## 4. 端到端验证与收尾

- [x] 4.1 真实环境冒烟（HTTP 层）：登录 → 创建带系统 pending 标签交易与普通交易 → 断言 `pending`：pending=true / false 均正确；前端 DOM 行为（渐变类、单行合并、摘要对位、多币种）由组件测试覆盖。浏览器像素级视觉因环境无 headless 浏览器未覆盖，留待本地确认
- [x] 4.2 运行 `openspec validate` 通过
# Tasks: polish-ui

## 1. 全局样式地基(style.css)

- [x] 1.1 添加输入控件 reset:`input, textarea, select { font: inherit; line-height: normal }`(或固定行高基准),解决文本不垂直居中
- [x] 1.2 补齐/统一 `--text-body` 变量,并新增语义色变量(如 `--color-income`/`--color-expense`/`--color-warning`)
- [x] 1.3 将 `#app` 的 `width: 100vw` 改为 `width: 100%`,消除横向滚动条根因
- [x] 1.4 抽取共享输入框 class(如 `.field-input`),供 ConfigPanel、ChannelMappingSection、AccountDrawer、TransactionFormOverlay、TransactionFilterDrawer 复用,替换各自重复定义

## 2. 抽屉滚动条闪现修复

- [x] 2.1 `ConfigPanel.vue` 的 `.drawer-container` 加 `overflow: hidden`
- [x] 2.2 `AccountDrawer.vue`(及 AccountCreateDrawer 若同模式)同样处理,验证滑入全程无滚动条

## 3. 颜色收敛

- [x] 3.1 列出全部硬编码颜色清单并归类(主色/语义色/中性色):TransactionCard、AccountDrawer、AssetsView、CalendarGrid、CashFlowDetailList、BalanceSheetPanel、TransactionView hero 渐变等
- [x] 3.2 逐一替换为主题变量;AssetsView 激活 tab 从 `#4ade80` 改为 `--accent`,全应用激活态统一

## 4. 观感细节

- [x] 4.1 实现应用内确认/提示组件(复用 toast 样式约定),替换 TransactionView、CalendarView、AccountDrawer、TransactionFormOverlay 共 4 处 `confirm()`/`alert()`
- [x] 4.2 统一 font-size 梯度(收敛 0.6875~0.9375rem 的碎片化字号到有限几档)
- [x] 4.3 ConfigPanel 成员/渠道/标签列表补空状态提示
- [x] 4.4 删除 `AccountDrawer.vue` input 上无效的 `display:flex` 样式

## 5. 死代码清理

- [x] 5.1 移除空 routes 的 vue-router 挂载(router/index.ts、main.ts、package.json 依赖),或确认保留意图后加注释说明

## 6. 验证

- [x] 6.1 逐视图目视回归:5 个 pane + 全部抽屉/表单,确认文本居中、无滚动条闪现、颜色一致、空状态正常
- [x] 6.2 运行前端测试与构建(`npm run build` / 既有 test 脚本),确认无回归

## 7. 交易表单打磨

- [x] 7.1 备注框:禁止拖动调整大小、宽度固定、高度随内容自适应、两端对齐
- [x] 7.2 标签栏改为从已有标签中选择(下拉),不再手工输入
- [x] 7.3 渠道链路列表改为横排,层级用 ▸ 隔开
- [x] 7.4 成员下拉与日期栏同一横排,成员在左占较少宽度
- [x] 7.5 渠道链路改为线性顺序链:选中一个才能选下一个、不重复、无占位节点、仅链尾可删;重写 ChannelPathInput 并清理死代码与 i18n 键
- [x] 7.6 控件右侧图标统一:所有相关 select 用自定义箭头固定在右侧内缩约 0.5ch,芯片 × 距右缘固定 0.5ch,内联 select 为幽灵芯片风格
- [x] 7.7 金额改为限数字普通文本框(type=text + inputmode=decimal + 输入过滤)
- [x] 7.8 下拉不显示箭头:原生 appearance:none,移除自定义 SVG 箭头及预留内边距
- [x] 7.9 成员框宽度由内部文本自适应,日期占满同行剩余空间
- [x] 7.10 标签控件与渠道链路同一视觉规格(横排芯片 + 虚线幽灵下拉,无 ▸);修复 `.field select` 优先级覆盖幽灵下拉样式的问题
- [x] 7.11 新建交易只自动创建一个分录;存在未填完分录(分类或金额为空)时"添加分录"按钮禁用

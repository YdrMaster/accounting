# ui-polish（增量）

## ADDED Requirements

### Requirement: 浏览器标签与应用标识

应用的浏览器标签呈现 SHALL 体现正式产品标识：`<title>` MUST 为「图论记账」；favicon MUST 使用「字母 A × 节点图」图形（全紫 `#646cff`、透明底、顶点中空带透明间隙环），不再出现 Vite 默认图标与脚手架标题。用户切换界面语言后，`document.documentElement.lang` MUST 同步为目标语言；应用初始加载时 SHALL 按初始语言设置。

#### Scenario: 标签栏标题与图标

- **WHEN** 用户在浏览器中打开应用任意页面
- **THEN** 标签显示「图论记账」与 A 字节点图 favicon，无 "try-frontend" 与 Vite 图标残留

#### Scenario: 切换界面语言

- **WHEN** 用户在设置中将界面语言从中文切换为英文（或反向）
- **THEN** `document.documentElement.lang` 立即更新为对应语言代码

#### Scenario: 首次加载

- **WHEN** 用户首次打开应用（含刷新）
- **THEN** `document.documentElement.lang` 与初始界面语言一致

#### Scenario: 亮暗标签栏可读性

- **WHEN** 浏览器处于浅色或深色主题
- **THEN** 透明底 favicon 的笔画（`#646cff`）在两种标签栏下均可辨识

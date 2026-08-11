## Purpose

`accounting-web` 渲染到用户的文本 SHALL 经 vue-i18n 本地化；SHALL NOT 以本地化文案作为跨边界的机器契约。前端错误展示依赖服务端返回的本地化 message 与结构化 code，自身不再持有需与服务端文案匹配的字面常量。

## ADDED Requirements

### Requirement: 模板文本经 vue-i18n 本地化
所有渲染到用户的文本（标签、表头、按钮、占位符、对话框消息）SHALL 经 vue-i18n 的 `t`/`$t` 本地化。SHALL NOT 在模板或 TS 中硬编码面向用户的中文/英文文案字面量。

#### Scenario: 界面标签随语言切换
- **WHEN** 用户切换界面语言
- **THEN** 所有标签、表头、按钮文本随之切换为对应语言

### Requirement: 不以本地化文案作跨边界契约
前端 SHALL NOT 维护需与服务端本地化 message 字面匹配的常量用于逻辑分支。需要按错误类型分支时，SHALL 依据服务端返回的结构化 `code` 字段。

#### Scenario: 认证错误分支依据 code
- **WHEN** 服务端返回验证码错误，`code: "bad_totp"`
- **THEN** 前端按 `code` 判定为"验证码错误"分支，不解析 message 文案

### Requirement: 错误展示依赖服务端本地化 message
前端错误展示组件 SHALL 直接展示服务端返回的本地化 `error`/`message`，SHALL NOT 自行对服务端错误串做语言假定或二次拼接固定语言文本。

#### Scenario: 展示服务端本地化错误
- **WHEN** 服务端返回英文本地化错误 message，前端界面语言为中文
- **THEN** 错误展示区呈现服务端返回的英文 message，包围文案（如"保存失败"）按前端语言本地化

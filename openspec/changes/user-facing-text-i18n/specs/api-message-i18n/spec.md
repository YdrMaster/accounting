## Purpose

`accounting-api` HTTP JSON 响应体中所有用户可见文本（错误 message、逐行错误 detail、人类可读字段）SHALL 可按请求语言本地化；错误 SHALL 结构化（稳定 code + 本地化 message），HTTP 状态码与客户端分支判定 SHALL NOT 依赖本地化字面量。

## ADDED Requirements

### Requirement: API 用户可见文本可按请求语言本地化
所有出现在 HTTP JSON 响应体中、面向用户的文本 SHALL 经 `rust-i18n` 的 `t!` 按请求语言环境组装。原始数据库/系统错误文本 SHALL NOT 直接进入响应体；SHALL 以通用本地化文案面向用户，原始详情仅记服务端日志。

#### Scenario: 英文 session 下的校验错误
- **WHEN** 客户端以 `lang=en` 发起会触发预算校验失败的请求（如空名称）
- **THEN** 响应体的 `error` 字段为英文本地化文案，且不含中文

#### Scenario: 中文 session 下的校验错误
- **WHEN** 客户端以 `lang=zh-CN` 发起同一请求
- **THEN** 响应体的 `error` 字段为中文本地化文案

#### Scenario: 数据库内部错误不外泄原文
- **WHEN** 请求触发底层 sqlite 错误
- **THEN** 响应体返回通用本地化的"数据库错误"文案，原始 sqlite 文本不出现在响应体
- **AND** 原始错误详情经服务端日志记录

### Requirement: 错误结构化携带稳定 code
错误响应 SHALL 携带稳定的机器可读 `code` 字段，与本地化 `message` 解耦。客户端 SHALL 依据 `code` 而非本地化文案进行分支判定。

#### Scenario: 认证验证码错误返回结构化 code
- **WHEN** 用户提交错误的 TOTP 验证码
- **THEN** 响应包含 `code: "bad_totp"` 与本地化 `message`
- **AND** 客户端按 `code` 判定分支，不解析 `message` 文案

### Requirement: 状态码判定不依赖本地化字面量
HTTP 状态码的选择 SHALL 基于错误变体（如"不存在"→404、"参数非法"→400），SHALL NOT 基于对本地化 message 的子串匹配。

#### Scenario: 资源不存在判定 404
- **WHEN** 请求引用不存在的预算表 id
- **THEN** 系统按错误变体判定返回 404，与当前语言环境无关

#### Scenario: 参数非法判定 400
- **WHEN** 请求携带非法参数（如空名称、非法金额）
- **THEN** 系统按错误变体判定返回 400，与当前语言环境无关

### Requirement: 导入逐行错误 detail 可本地化
账单导入的响应 `errors[].detail` SHALL 按请求语言环境本地化；`detail` SHALL 携带或由结构化数据派生，SHALL NOT 是预字符串化的固定语言文案。

#### Scenario: 逐行解析错误的本地化 detail
- **WHEN** 导入在某行金额解析失败，请求语言为英文
- **THEN** `errors[].detail` 为英文本地化文案，含行号与失败值

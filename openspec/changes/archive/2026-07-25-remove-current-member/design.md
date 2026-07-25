# remove-current-member 设计

## Context

两套"当前用户"机制均为死代码：

- 服务端：`GET/PUT /api/me`（`accounting-api/src/handlers/me.rs`）读写 settings 表 `current_member_id`。前端从未调用该端点（`client.ts` 无 `/me` 请求），settings 键无任何写入方，解析永远 fallback 到成员列表第一项
- 前端：member store 的 `currentMemberId`（`accounting-web/src/stores/member.ts`），内存态、不持久化，恒等于 `members[0]`，唯一消费方是 `TransactionFormOverlay.vue:45` 的表单默认值

系统的真实语义一直是"显式指定成员"：CLI 导入用 `--member`，交易创建 API body 带 `member_id`，交易列表用 `?member=` 过滤。本变更把形式与语义对齐。

## Goals / Non-Goals

**Goals:**

- 删除服务端 `/api/me` 及 `current_member_id` 的所有代码路径
- 删除前端 `currentMemberId`
- 交易表单成员默认未选中、强制手工选择

**Non-Goals:**

- 不删除 settings 表本身或其他键（`current_member_id` 存量数据自然废弃，无需迁移）
- 不改变成员 CRUD（`/api/members`）
- 不引入任何替代性的"当前成员"机制（包括 localStorage 记忆上次选择）

## Decisions

### D1: 直接删除 /api/me，无兼容期

已确认无任何调用方（前端未使用、CLI 不走 HTTP、无其他客户端）。保留死端点只会延续假象。

### D2: settings 键不做迁移清理

`current_member_id` 是 settings 表中的一行惰性数据，删除它需要一次专门的迁移，收益为零（无人读取）。自然废弃。

### D3: 表单成员默认未选中，而不是默认第一项

"默认第一项"仍是一种隐式选择——用户可能没注意就提交了错误成员。表单已有占位符（`txForm.memberPlaceholder`）与必填校验（`TransactionFormOverlay.vue:113`），默认 `null` 即可强制显式选择，零新增 UI。

### D4: i18n key 随代码删除

`parse_member_id_failed`、`no_members`、`member_not_found` 已验证仅 me.rs 引用，一并从两个 locale 文件删除，不留死文案。

## Risks / Trade-offs

- [有未知外部脚本调用 /api/me] → 该端点返回的只是成员列表第一项的包装，无独立信息价值；如真有调用方，`GET /api/members` 第一项等价
- [单成员用户每次填表多一次选择] → 有意为之的显式语义；若未来觉得繁琐，可在 UI 层做"记住上次选择"，那是独立变更

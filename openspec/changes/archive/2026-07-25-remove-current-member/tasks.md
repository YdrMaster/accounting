# remove-current-member 任务

## 1. 后端：删除 /api/me

- [x] 1.1 删除 `accounting-api/src/handlers/me.rs`；`handlers/mod.rs` 移除 `pub mod me`；`router.rs` 移除 `me::router()` 的 merge
- [x] 1.2 `accounting-api/src/dto.rs` 删除 `MeDto` 与 `SetMeRequest`
- [x] 1.3 `accounting-api/locales/zh-CN.yaml` 与 `en.yaml` 删除 `parse_member_id_failed`、`no_members` 两个 key（`member_not_found` 保留，导入端点校验成员时使用）
- [x] 1.4 全仓 grep 确认 `current_member_id`、`resolve_current_member_id`、`/api/me` 在 Rust 代码中无残留；`cargo test -p accounting-api` 与 `cargo clippy` 通过

## 2. 前端：删除 currentMemberId

- [x] 2.1 `accounting-web/src/stores/member.ts` 删除 `currentMemberId`
- [x] 2.2 `accounting-web/src/components/layout/TransactionFormOverlay.vue` 删除第 45 行默认值赋值（`memberId` 初值保持 `null`，走既有占位符与必填校验）
- [x] 2.3 全仓 grep `currentMemberId`、`/api/me` 确认前端无残留；更新受影响的测试

## 3. 验证

- [x] 3.1 `cargo test --workspace` 通过
- [x] 3.2 `cd accounting-web && npx vitest run && npm run lint` 通过
- [x] 3.3 e2e 冒烟：起服务验证 `GET /api/me` 返回 404；交易表单新建时成员为未选中占位态

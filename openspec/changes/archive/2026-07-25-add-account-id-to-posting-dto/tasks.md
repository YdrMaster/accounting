# Tasks: PostingDto 增加 account_id

## 1. 后端

- [x] 1.1 在 `accounting-api/src/dto.rs` 的 `PostingDto` 中新增 `account_id: i64` 字段
- [x] 1.2 在 `accounting-api/src/handlers/transaction.rs` 的 `posting_to_dto` 中填充 `account_id: p.account_id.0`；用 Grep 确认无其他手工构造 PostingDto 的位置
- [x] 1.3 运行 `cargo test -p accounting-api`（及受影响 crate）确认编译和测试通过

## 2. 前端

- [x] 2.1 在 `accounting-web/src/types/api.ts` 的 `PostingDto` 中新增 `account_id: number`
- [x] 2.2 修改 `accounting-web/src/components/layout/TransactionFormOverlay.vue` 的 `loadTransaction`，用 `accountId: p.account_id` 替换硬编码的 `null`，并删除过时注释
- [x] 2.3 运行前端类型检查/构建（`npm run build` 或项目既有检查命令）确认无类型错误
- [x] 2.4 `AccountPicker.vue` 已选中时显示账户名（按 id 从 accountStore 解析，找不到时回退 `#id`），而非"账户 #id"
- [x] 2.5 修复 `TransactionFormOverlay.vue` 的 `onAccountSelect`：accountName 改用 accountStore 的完整路径（`accountPath`），替换原来的"账户 #id"（该值提交后端会解析失败）；账户 store 新增 `accountPath` 辅助函数，表单挂载时加载账户列表
- [x] 2.6 `AccountPickerOverlay.vue` 新增 `currentId` prop：打开面板时选中当前账户并展开其祖先链；`AccountPicker.vue` 传入 `modelValue`；新增 `AccountPickerOverlay.spec.ts` 覆盖两种初始状态

## 3. 验证

- [x] 3.1 手动验证：打开一笔导入交易的编辑表单，分录账户正确回显；不改动账户直接保存成功

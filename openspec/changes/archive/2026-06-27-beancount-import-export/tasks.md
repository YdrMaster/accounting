## 1. 移除 Posting.description 字段

- [x] 1.1 从 `accounting/src/posting.rs` 的 `Posting` 结构体中移除 `description` 字段
- [x] 1.2 从 `accounting-sql/src/schema.rs` 的 postings 表定义中移除 `description TEXT` 列
- [x] 1.3 更新 `accounting-sql/src/repo/posting.rs` 中所有涉及 description 的 SQL 查询和映射
- [x] 1.4 更新 `accounting-api/src/dto.rs` 和 handlers 中涉及 posting description 的字段
- [x] 1.5 更新 `accounting-cli/src/cmd/mod.rs` 中 PostingRow 的 description 字段
- [x] 1.6 更新 `accounting-service/src/import/alipay.rs` 等引用 posting description 的代码
- [x] 1.7 修复所有因移除 description 导致的编译错误和测试失败

## 2. 创建 accounting-beancount crate

- [x] 2.1 创建 `accounting-beancount/Cargo.toml`，添加依赖（chrono, rust_decimal, serde_json）
- [x] 2.2 在 workspace `Cargo.toml` 中注册新 crate
- [x] 2.3 创建 `accounting-beancount/src/lib.rs` 模块结构

## 3. 实现 beancount 文本生成器（导出用）

- [x] 3.1 实现 beancount metadata 格式化辅助函数（key-value 缩进、字符串转义、JSON 值编码）
- [x] 3.2 实现 commodity 指令生成：`commodity SYMBOL` + metadata（internal_id, name, precision）
- [x] 3.3 实现 account open/close 指令生成：处理 AccountType::Import 到 Equity:Import 的路径映射，输出 billing_day/repayment_day/account_type metadata
- [x] 3.4 实现 member/channel custom 指令生成：`custom "member" "名称"` + metadata
- [x] 3.5 实现 transaction 指令生成：date + time + `* "" "description"` + kind/member/channel_path metadata + posting 行（含 reimbursable metadata）+ #hashtag
- [x] 3.6 实现 posting 行生成：amount + currency + 可选 cost 语法 `{amount currency}` + internal_id metadata
- [x] 3.7 实现 reversal_of metadata 生成：从 linked_posting_id 构建 JSON
- [x] 3.8 实现 document 指令生成：附件路径引用
- [x] 3.9 实现完整导出编排：按依赖顺序输出（commodity → account open → member/channel → transactions by date → account close → document）

## 4. 实现 beancount 文本解析器（导入用）

- [x] 4.1 实现 beancount 文本逐行解析框架：识别日期行、posting 行、metadata 行、空行、注释行
- [x] 4.2 实现 commodity 指令解析：提取 symbol + metadata（internal_id, name, precision）
- [x] 4.3 实现 account open/close 指令解析：提取路径 + metadata（internal_id, account_type, billing_day, repayment_day）
- [x] 4.4 实现 member/channel custom 指令解析：提取名称 + metadata（internal_id, description）
- [x] 4.5 实现 transaction 指令解析：提取 date/time/payee/narration + metadata（internal_id, kind, member, channel_path, reversal_of）+ #hashtag + posting 行
- [x] 4.6 实现 posting 行解析：account path + amount + currency + 可选 cost 语法 + metadata（internal_id, reimbursable）
- [x] 4.7 实现 document 指令解析：提取 account path + 文件路径
- [x] 4.8 实现错误处理：解析失败时输出行号和错误原因

## 5. 实现导出 service 层

- [x] 5.1 在 `accounting-beancount` 中实现 `export` 函数：从 SqliteDatabase 读取全量数据，调用生成器输出 beancount 文本和附件文件
- [x] 5.2 导出 commodity：调用 `db.commodity_list()`
- [x] 5.3 导出 account：调用 `db.account_list()`，通过 `db.account_find_root_name()` 获取根节点名确定 AccountType，构建完整路径
- [x] 5.4 导出 member：调用 `db.member_list()`
- [x] 5.5 导出 channel：调用 `db.channel_list()`
- [x] 5.6 导出 transaction：遍历所有交易，对每笔交易获取 posting 列表、channel_path 列表、tag 列表、attachment 列表
- [x] 5.7 附件导出：将二进制数据写入 `<输出目录>/attachments/<id>_<filename>`

## 6. 实现导入 service 层

- [x] 6.1 在 `accounting-beancount` 中实现 `import` 函数：解析 beancount 文本，写入 SqliteDatabase
- [x] 6.2 导入 commodity：使用 `commodity_upsert_by_symbol`，建立 old_id → new_id 映射
- [x] 6.3 导入 account：使用 `account_get_or_create_by_path`（处理 Import 类型路径还原），建立 old_id → new_id 映射，更新 billing_day/repayment_day
- [x] 6.4 导入 member：使用 `member_get_or_create_by_name`，建立 old_id → new_id 映射
- [x] 6.5 导入 channel：使用 `channel_upsert_by_name`，建立 old_id → new_id 映射
- [x] 6.6 导入 transaction：创建 Transaction（合并 payee + narration → description），设置 kind/member_id
- [x] 6.7 导入 posting：通过映射表重连 account_id/commodity_id，设置 cost/cost_commodity_id/is_reimbursable/linked_posting_id
- [x] 6.8 导入 channel_path：通过映射表重连 channel_id，创建 ChannelPathNode 批量插入
- [x] 6.9 导入 tag：使用 `tag_upsert_by_name`，关联到交易
- [x] 6.10 导入 attachment：读取外部文件，创建 Attachment 记录
- [x] 6.11 导入 account close：使用 `account_close` 设置 closed_at
- [x] 6.12 实现 internal_id 去重：检查已有交易的 internal_id，跳过重复
- [x] 6.13 导入完成后输出统计摘要

## 7. CLI 集成

- [x] 7.1 在 `accounting-cli/src/cmd/` 中新增 `beancount.rs` 模块
- [x] 7.2 实现 `BeancountCmd` 子命令组（export/import）
- [x] 7.3 实现 `export` 子命令：接收输出目录参数，调用导出 service
- [x] 7.4 实现 `import` 子命令：接收输入文件参数，调用导入 service
- [x] 7.5 在 `cmd/mod.rs` 的 `Commands` 枚举中注册 `Beancount` 变体
- [x] 7.6 在 `main.rs` 的 match 中分发 `Commands::Beancount`

## 8. 测试

- [x] 8.1 为 beancount 文本生成器编写单元测试（各指令类型的输出格式）
- [x] 8.2 为 beancount 文本解析器编写单元测试（各指令类型的解析）
- [x] 8.3 编写 round-trip 集成测试：创建测试数据 → 导出 → 导入到新库 → 比对数据一致性
- [x] 8.4 编写附件 round-trip 测试
- [x] 8.5 编写错误处理测试（格式错误的输入、缺失的附件文件）
- [x] 8.6 全量编译和测试通过验证

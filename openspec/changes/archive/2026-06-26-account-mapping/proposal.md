## Why

当前导入系统将所有交易放入 Import 树下的扁平分类账户（如 `Import:支付宝:餐饮美食`），用户需要手动将 Posting 移动到正式账户树。随着导入量增长，逐条手动分类成为主要瓶颈。账户映射表让用户一次性配置"渠道分类→正式账户"的规则，导入时自动应用，逐步自动化分类流程。

## What Changes

- 新增 `account_mappings` 表，存储绑定在 `(member_id, channel_id)` 上的 `category→account_id` 映射，category 格式为 `"收支:<分类>"` 或 `"资产:<付款方式>"`
- 改造 `BillPosting` 结构：移除 `account_path: String`，替换为 `role: PostingRole` + `category: String`，适配器只输出分类信息不再拼路径
- 改造 `ImportService`：导入时查询映射表，有映射直接用正式账户，无映射走 Import fallback（`Import:<渠道>:收支:<分类>` / `Import:<渠道>:资产:<付款方式>`）
- 修正现有支付宝适配器的金额方向（当前收支侧与资产侧金额符号颠倒）
- Import 树结构从扁平（`Import:支付宝:餐饮美食`）改为分角色（`Import:支付宝:收支:餐饮美食`、`Import:支付宝:资产:蚂蚁宝藏信用卡`）
- 新增 `mapping` CLI 子命令（set / list / delete），仅 CLI 实现
- 账户删除时增加映射引用检查，被映射引用的账户禁止删除

## Capabilities

### New Capabilities
- `account-mapping`: 绑定在 (成员, 渠道) 二元组上的分类字符串→账户编号映射表，支持收支侧和资产侧两类映射

### Modified Capabilities
- `bill-import`: BillPosting 结构改造（role+category 替代 account_path）、ImportService 导入时查映射表、Import fallback 路径结构变更、修正金额方向

## Impact

- **accounting crate**: 新增 `PostingRole` 枚举、`AccountMapping` 结构体
- **accounting-sql**: schema 新增 `account_mappings` 表；repo 新增映射 CRUD 方法
- **accounting-service**: `BillPosting` 结构改造；适配器输出改造；`ImportService` 核心逻辑改造；新增 `MappingService`；`AccountService` 删除前增加映射引用检查
- **accounting-cli**: 新增 `mapping` 子命令
- **数据兼容**: 现有数据库已有 Import 树下的扁平分类账户（如 `Import:支付宝:餐饮美食`），新导入的交易会进入带角色前缀的子树（如 `Import:支付宝:收支:餐饮美食`），旧数据不受影响
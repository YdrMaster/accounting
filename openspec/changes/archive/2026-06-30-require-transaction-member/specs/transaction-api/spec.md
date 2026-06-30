## 变更的需求

### 需求：TransactionDto 包含成员名
TransactionDto 必须包含 `member_name: String` 字段，返回交易关联的成员名称。

#### 场景：查询交易成员名
- **当** 查询任意一笔交易时
- **那么** TransactionDto.member_name 为该交易关联的成员名称

### 需求：CreateTransactionRequest 必须提供成员
CreateTransactionRequest 必须包含 `member_id: i64` 字段，且该成员必须已存在。

#### 场景：创建交易时提供有效成员
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 指向一个已存在成员时
- **那么** 系统创建交易并关联该成员

#### 场景：创建交易时未提供成员
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 缺失时
- **那么** 系统拒绝请求并返回成员必填错误

#### 场景：创建交易时成员不存在
- **当** 用户提交 CreateTransactionRequest 且 `member_id` 指向不存在的成员时
- **那么** 系统拒绝请求并返回成员不存在错误

## 移除的需求

_无_

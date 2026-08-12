## REMOVED Requirements

### Requirement: 纯导入账单检测
**Reason**: 卡片改为对所有交易统一展示（见 `transaction-list-ui` 的「交易卡片展示」），不再需要区分纯导入账单与普通账单，`isPureImport(tx)` 检测随之移除。
**Migration**: 无行为迁移需求；纯导入账单在折叠态同样显示金额与账户摘要。

### Requirement: 纯导入账单显示
**Reason**: 原「纯导入账单隐藏收支账户名称、金额与资产账户」的行为被统一形式展示取代，避免待分类（pending）交易缺失金额等关键识别信息。纯导入/pending 交易现以琥珀色渐变背景标识（见 `transaction-list-ui` 的「待分类交易标识」）。
**Migration**: 纯导入账单不再隐藏任何显示区域，折叠态显示金额、账户摘要、成员与标签；待分类状态由琥珀渐变背景传达。
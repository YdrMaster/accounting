# assets-view

资产页面——展示资产负债表数据，包括总资产、总负债、净资产及各账户余额明细。

## Requirements

### Requirement: 资产总览展示
资产页 SHALL 从 GET /api/reports/balance-sheet 获取数据，展示总资产、总负债、净资产。

#### Scenario: 加载资产数据
- **WHEN** 用户切换到资产页
- **THEN** 系统调用 balance-sheet API 获取资产账户余额数据

#### Scenario: 计算总资产
- **WHEN** API 返回资产账户余额列表
- **THEN** 总资产 = 所有余额 > 0 的账户金额之和

#### Scenario: 计算总负债
- **WHEN** API 返回资产账户余额列表
- **THEN** 总负债 = 所有余额 < 0 的账户金额取绝对值之和

#### Scenario: 计算净资产
- **WHEN** 总资产和总负债已计算
- **THEN** 净资产 = 总资产 - 总负债

### Requirement: 账户余额明细
资产页 SHALL 展示各资产账户的余额明细。

#### Scenario: 显示账户余额
- **WHEN** API 返回数据
- **THEN** 每个账户显示账户名和各币种余额

#### Scenario: 过滤零余额
- **WHEN** 某账户余额为 0
- **THEN** 该账户不显示在明细中（后端已过滤）

### Requirement: 只读展示
资产页 SHALL 为只读展示，不提供编辑功能。

#### Scenario: 无编辑入口
- **WHEN** 用户查看资产页
- **THEN** 页面上没有编辑、删除或新建按钮

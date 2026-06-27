## MODIFIED Requirements

### Requirement: 交易列表按月加载
系统 SHALL 支持按月份加载交易列表，默认加载当前月份。页面顺序中交易页面位于第一位。

#### Scenario: 加载当月交易
- **WHEN** 用户打开交易列表
- **THEN** 系统加载当前月份的交易数据，交易页面为默认显示页面

#### Scenario: 切换月份
- **WHEN** 用户切换到其他月份
- **THEN** 系统加载目标月份的交易数据

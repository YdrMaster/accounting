## MODIFIED Requirements

### Requirement: 资产总览展示
资产页 SHALL 从 GET /api/reports/balance-sheet 获取数据，在「资产负债表」tab（默认 tab）中展示总资产、总负债、净资产；汇总指标上方为资产趋势折线图（见 assets-visual-reports 规格）。

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

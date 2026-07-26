## ADDED Requirements

### Requirement: 交易列表筛选入口
TransactionView header SHALL 在新建交易按钮左侧提供筛选按钮，点击后展开筛选抽屉。

#### Scenario: 筛选按钮位置
- **WHEN** 用户在交易页
- **THEN** header 右侧显示两个按钮，筛选按钮在新建交易按钮左侧

#### Scenario: 仅交易页显示
- **WHEN** 用户切换到其他面板（资产、账户、日历、预算）
- **THEN** 筛选按钮不显示

### Requirement: 月收支汇总跟随筛选
筛选激活时，顶部月收支汇总 SHALL 基于筛选后的交易数据计算，并标注筛选状态。

#### Scenario: 筛选后汇总变化
- **WHEN** 用户激活标签筛选"餐饮"
- **THEN** 月支出/收入/结余仅统计筛选结果中的交易

#### Scenario: 筛选状态标注
- **WHEN** 筛选条件激活中
- **THEN** 汇总区域的月份标签旁显示"已筛选"标记

#### Scenario: 清除筛选后恢复
- **WHEN** 用户重置所有筛选条件
- **THEN** 汇总恢复为已加载范围内全部交易的统计，"已筛选"标记消失

## MODIFIED Requirements

### Requirement: 交易列表新建交易入口
TransactionList 所在页面 SHALL 提供新建交易按钮，点击后打开交易表单覆盖层。ViewPanel header SHALL 支持渲染多个 action 按钮，每个按钮可携带自定义图标。

#### Scenario: 交易页新建按钮
- **WHEN** 用户在交易页
- **THEN** 页面提供新建交易按钮

#### Scenario: 日历页新建按钮
- **WHEN** 用户在日历页
- **THEN** 页面提供新建交易按钮

#### Scenario: 多按钮渲染
- **WHEN** 某 view 注册了多个 panel action
- **THEN** header 按注册顺序从右到左渲染所有按钮，各按钮独立响应点击

#### Scenario: 按钮图标
- **WHEN** panel action 指定了 icon 字段
- **THEN** 按钮显示对应图标而非默认 `+` 前缀

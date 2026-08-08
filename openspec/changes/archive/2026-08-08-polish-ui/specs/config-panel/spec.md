# Spec Delta: config-panel

## MODIFIED Requirements

### Requirement: 配置面板覆盖层行为
配置面板 SHALL 渲染为底部抽屉覆盖层，覆盖整个视口，带半透明背景。抽屉内容 SHALL 从底部滑入，`max-height: 66vh`。背景 SHALL 在面板打开时阻止与底层内容的所有交互。滑入动画全过程 MUST NOT 导致页面出现纵向或横向滚动条。

#### Scenario: 覆盖层阻止底层交互
- **WHEN** 配置面板打开时
- **THEN** 点击背景区域不会触发底层页面的任何操作

#### Scenario: 抽屉动画
- **WHEN** 配置面板打开时
- **THEN** 抽屉内容从底部平滑滑入，且动画期间与结束后页面均不出现滚动条

## ADDED Requirements

### Requirement: 配置列表空状态
成员、渠道、标签列表为空时 SHALL 展示明确的空状态提示文案，引导用户添加，而非仅渲染添加操作行。

#### Scenario: 成员列表为空
- **WHEN** 当前没有任何成员
- **THEN** 成员 tab 显示空状态提示与添加入口

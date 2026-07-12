## ADDED Requirements

### Requirement: 初始加载最新交易
系统 SHALL 在页面加载时从当日往回获取最新 100 笔交易，并记录已加载的时间范围。

#### Scenario: 今天有交易
- **WHEN** 用户打开交易列表页面，今天是 2026-07-12
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-12&limit=100`
- **THEN** 返回 100 笔交易，最新是 07-12 09:00，最老是 07-05 13:00
- **THEN** loadedRange 设置为 `{ from: "2026-07-05", to: "2026-07-12" }`
- **THEN** 页面显示这 100 笔交易

#### Scenario: 今天没有交易
- **WHEN** 用户打开交易列表页面，今天是 2026-07-12，但今天没有交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-12&limit=100`
- **THEN** 返回 100 笔交易，最新是 07-10 15:00（今天之前的最新交易）
- **THEN** loadedRange 设置为 `{ from: "2026-07-03", to: "2026-07-10" }`

#### Scenario: 没有任何交易
- **WHEN** 数据库中没有任何交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-12&limit=100`
- **THEN** 返回空数组
- **THEN** loadedRange 保持为 null
- **THEN** 页面显示"暂无交易记录"

### Requirement: 滚动触发加载更早数据
当用户滚动到已加载范围的最老日期时，系统 SHALL 自动触发加载更早的 100 笔交易。

#### Scenario: 滚动到 from 日期触发加载
- **WHEN** loadedRange 是 `{ from: "2026-07-05", to: "2026-07-12" }`
- **WHEN** 用户滚动列表，07-05 的交易进入视口
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-05&limit=100`
- **THEN** 返回 100 笔更早的交易（07-05 12:59 → 07-03 08:00）
- **THEN** loadedRange 更新为 `{ from: "2026-07-03", to: "2026-07-12" }`
- **THEN** 新交易追加到列表末尾

#### Scenario: 加载到最早数据
- **WHEN** loadedRange 是 `{ from: "2026-04-01", to: "2026-07-12" }`
- **WHEN** 用户滚动到 04-01 的交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-04-01&limit=100`
- **THEN** 返回 50 笔交易（04-01 09:00 → 04-01 08:00，这是最早的数据）
- **THEN** loadedRange 更新为 `{ from: "2026-04-01", to: "2026-07-12" }`（from 不变）
- **THEN** 不再触发更多加载（已到达最早数据）

#### Scenario: 加载中不重复触发
- **WHEN** 用户快速滚动到 from 日期
- **WHEN** 系统正在加载更早数据（loading = true）
- **THEN** 不触发新的加载请求
- **THEN** 等待当前加载完成后再判断是否需要继续加载

### Requirement: 同一天数据膨胀
如果加载的 100 笔交易都在同一天，系统 SHALL 自动将 limit 翻倍直到查询结果跨天。

#### Scenario: 100 笔在同一天，翻倍到 200
- **WHEN** 系统请求 `GET /api/transactions?to=2026-07-05&limit=100`
- **WHEN** 返回 100 笔交易，最新是 07-05 12:59，最老是 07-05 06:00（同一天）
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-05&limit=200`
- **WHEN** 返回 200 笔交易，最新是 07-05 12:59，最老是 07-04 22:00（跨天）
- **THEN** loadedRange.from 更新为 "2026-07-04"
- **THEN** 停止膨胀

#### Scenario: 多次翻倍直到跨天
- **WHEN** 系统请求 `GET /api/transactions?to=2026-07-05&limit=100`
- **WHEN** 返回 100 笔，都在 07-05（同一天）
- **THEN** 请求 limit=200，返回 200 笔，还在 07-05
- **THEN** 请求 limit=400，返回 400 笔，还在 07-05
- **THEN** 请求 limit=800，返回 800 笔，最新 07-05 12:59，最老 07-04 01:00（跨天）
- **THEN** 停止膨胀，loadedRange.from = "2026-07-04"

#### Scenario: 同一天交易极多
- **WHEN** 某天有 10000 笔交易
- **THEN** limit 依次翻倍：100→200→400→800→1600→3200→6400→12800
- **THEN** 共 8 次请求后跨天
- **THEN** 不设上限，直到跨天为止

### Requirement: 日历按需加载
当用户点击日历上的某天，如果该天不在已加载范围内，系统 SHALL 加载该天全部交易。

#### Scenario: 点击范围内的日期
- **WHEN** loadedRange 是 `{ from: "2026-07-05", to: "2026-07-12" }`
- **WHEN** 用户点击日历上的 2026-07-08
- **THEN** 07-08 在 loadedRange 内
- **THEN** 直接从主列表过滤该天交易，无需 API 请求
- **THEN** 页面显示 07-08 的所有交易

#### Scenario: 点击范围外的日期
- **WHEN** loadedRange 是 `{ from: "2026-07-05", to: "2026-07-12" }`
- **WHEN** 用户点击日历上的 2026-06-15
- **THEN** 06-15 不在 loadedRange 内
- **THEN** 系统请求 `GET /api/transactions?from=2026-06-15&to=2026-06-15`
- **THEN** 返回 06-15 的所有交易（无 limit）
- **THEN** 交易存储到 calendarDays["2026-06-15"]
- **THEN** 页面显示 06-15 的所有交易

#### Scenario: 重复点击同一天
- **WHEN** 用户点击 2026-06-15，已加载到 calendarDays
- **WHEN** 用户再次点击 2026-06-15
- **THEN** 检查 calendarDays 已有该天数据
- **THEN** 不触发 API 请求
- **THEN** 直接显示缓存的交易

#### Scenario: 自由翻月
- **WHEN** 用户点击日历的 ← 按钮翻到 2026-05
- **WHEN** 用户点击 2026-05-20
- **THEN** 系统请求 `GET /api/transactions?from=2026-05-20&to=2026-05-20`
- **THEN** 返回 05-20 的所有交易
- **THEN** 页面显示该天交易

### Requirement: 数据合并展示
系统 SHALL 在展示时合并主列表和日历加载的数据，按时间倒序排列，去重。

#### Scenario: 合并主列表和日历数据
- **WHEN** 主列表有 200 笔交易（07-05 → 07-12）
- **WHEN** calendarDays 有 "2026-06-15": [5 笔交易]
- **THEN** allTransactions 合并两者，按 date_time 倒序排列
- **THEN** 07-12 的交易在最前面，06-15 的交易在中间，07-05 的交易在最后
- **THEN** 无重复交易（按 ID 去重）

#### Scenario: 日历数据与主列表重叠
- **WHEN** 主列表包含 07-08 的 10 笔交易
- **WHEN** 用户点击日历 07-08，加载到 calendarDays
- **THEN** 合并时优先使用主列表数据（已加载范围内）
- **THEN** calendarDays["2026-07-08"] 被忽略（避免重复）

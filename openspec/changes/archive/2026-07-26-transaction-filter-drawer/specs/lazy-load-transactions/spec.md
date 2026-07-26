## MODIFIED Requirements

### Requirement: 初始加载最新交易
系统 SHALL 在页面加载时从当日往回获取最新 100 笔交易，并记录已加载的时间范围。若存在激活的筛选条件，请求 SHALL 携带全部筛选参数。

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

#### Scenario: 带筛选条件加载
- **WHEN** 筛选条件为 `{ accounts: [1,2], tags: ["餐饮"], from: "2026-07-01" }`，今天是 2026-07-12
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-12&from=2026-07-01&account=1&account=2&tag=餐饮&limit=100`
- **THEN** 返回符合筛选条件的交易
- **THEN** loadedRange 基于返回结果设置

#### Scenario: 筛选条件变化后重新加载
- **WHEN** 用户变更筛选条件（300ms 防抖后）
- **THEN** 系统清空当前列表和 loadedRange，以新条件重新执行初始加载
- **THEN** 列表滚动位置重置到顶部

### Requirement: 滚动触发加载更早数据
当用户滚动到已加载范围的最老日期时，系统 SHALL 自动触发加载更早的 100 笔交易，请求携带当前激活的筛选参数。若筛选条件指定了 from 日期，翻页到该日期即停止。

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

#### Scenario: 带筛选条件翻页
- **WHEN** 筛选条件含 `tags: ["餐饮"]`，loadedRange.from 为 2026-07-05
- **WHEN** 用户滚动到 07-05 的交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-05&tag=餐饮&limit=100`
- **THEN** 仅返回符合筛选条件的更早交易

#### Scenario: 翻页到筛选 from 日期停止
- **WHEN** 筛选条件含 `from: "2026-07-01"`，loadedRange.from 为 2026-07-02
- **WHEN** 用户滚动到 07-02 的交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-02&from=2026-07-01&limit=100`
- **THEN** 加载完成后 loadedRange.from <= 2026-07-01
- **THEN** 不再触发更多加载（已到达筛选范围起点）

### Requirement: 同一天数据膨胀
如果加载的交易都在同一天，系统 SHALL 自动将 limit 翻倍直到查询结果跨天。筛选激活时翻倍次数 SHALL 不超过 3 次（limit 上限 800）。

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

#### Scenario: 同一天交易极多（无筛选）
- **WHEN** 某天有 10000 笔交易，无激活筛选条件
- **THEN** limit 依次翻倍：100→200→400→800→1600→3200→6400→12800
- **THEN** 共 8 次请求后跨天
- **THEN** 不设上限，直到跨天为止

#### Scenario: 筛选激活时翻倍上限
- **WHEN** 筛选条件激活，请求 limit=100 返回同一天数据
- **THEN** limit 依次翻倍：100→200→400→800
- **THEN** 800 后即使仍在同一天也停止膨胀
- **THEN** loadedRange.from 设为该天日期

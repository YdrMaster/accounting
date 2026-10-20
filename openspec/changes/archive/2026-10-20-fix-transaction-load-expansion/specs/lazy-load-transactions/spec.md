## MODIFIED Requirements

### Requirement: 初始加载最新交易
系统 SHALL 在页面加载时以整日模式（`full_days=true`）从当日往回获取最新 100 笔交易，返回结果 SHALL 落在完整的日期边界上；系统 SHALL 记录已加载的整天范围。若存在激活的筛选条件，请求 SHALL 携带全部筛选参数。

#### Scenario: 今天有交易
- **WHEN** 用户打开交易列表页面，今天是 2026-07-12
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-12&limit=100&full_days=true`
- **THEN** 返回不少于 100 笔交易，最新是 07-12 09:00，最旧是 07-05 06:00（07-05 为完整的一天）
- **THEN** loadedRange 设置为 `{ from: "2026-07-05", to: "2026-07-12" }`
- **THEN** 页面显示这些交易

#### Scenario: 今天没有交易
- **WHEN** 用户打开交易列表页面，今天是 2026-07-12，但今天没有交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-12&limit=100&full_days=true`
- **THEN** 返回不少于 100 笔交易，最新是 07-10 15:00（今天之前的最新交易）
- **THEN** loadedRange 设置为 `{ from: "2026-07-03", to: "2026-07-10" }`

#### Scenario: 没有任何交易
- **WHEN** 数据库中没有任何交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-12&limit=100&full_days=true`
- **THEN** 返回空数组
- **THEN** loadedRange 保持为 null
- **THEN** 页面显示"暂无交易记录"

#### Scenario: 带筛选条件加载
- **WHEN** 筛选条件为 `{ accounts: [1,2], tags: ["餐饮"], from: "2026-07-01" }`，今天是 2026-07-12
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-12&from=2026-07-01&account=1&account=2&tag=餐饮&limit=100&full_days=true`
- **THEN** 返回符合筛选条件、且页尾对齐整天的交易
- **THEN** loadedRange 基于返回结果设置

#### Scenario: 筛选条件变化后重新加载
- **WHEN** 用户变更筛选条件（300ms 防抖后）
- **THEN** 系统清空当前列表和 loadedRange，以新条件重新执行初始加载
- **THEN** 列表滚动位置重置到顶部

### Requirement: 滚动触发加载更早数据
当用户滚动到已加载范围的最老一天时，系统 SHALL 以整日模式加载更早的一页，游标为当前最旧一天的前一天；请求 SHALL 携带当前激活的筛选参数。若筛选条件指定了 from 日期，翻页到该日期即停止；返回空数组表示已到达最早数据。

#### Scenario: 滚动到 from 日期触发加载
- **WHEN** loadedRange 是 `{ from: "2026-07-05", to: "2026-07-12" }`
- **WHEN** 用户滚动列表，07-05 的交易进入视口
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-04&limit=100&full_days=true`
- **THEN** 返回更早的、页尾对齐整天的一页交易（不少于 100 笔）
- **THEN** loadedRange 更新为 `{ from: "<该页最旧一天>", to: "2026-07-12" }`
- **THEN** 新交易追加到列表末尾

#### Scenario: 加载到最早数据
- **WHEN** loadedRange 是 `{ from: "2026-04-01", to: "2026-07-12" }`
- **WHEN** 用户滚动到 04-01 的交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-03-31&limit=100&full_days=true`
- **THEN** 返回空数组（03-31 及更早没有交易）
- **THEN** loadedRange 保持 `{ from: "2026-04-01", to: "2026-07-12" }`
- **THEN** 不再触发更多加载（已到达最早数据）

#### Scenario: 加载中不重复触发
- **WHEN** 用户快速滚动到最旧一天
- **WHEN** 系统正在加载更早数据（loading = true）
- **THEN** 不触发新的加载请求
- **THEN** 等待当前加载完成后再判断是否需要继续加载

#### Scenario: 带筛选条件翻页
- **WHEN** 筛选条件含 `tags: ["餐饮"]`，loadedRange.from 为 2026-07-05
- **WHEN** 用户滚动到 07-05 的交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-04&tag=餐饮&limit=100&full_days=true`
- **THEN** 仅返回符合筛选条件的更早交易

#### Scenario: 翻页到筛选 from 日期停止
- **WHEN** 筛选条件含 `from: "2026-07-01"`，loadedRange.from 为 2026-07-02
- **WHEN** 用户滚动到 07-02 的交易
- **THEN** 系统请求 `GET /api/transactions?to=2026-07-01&from=2026-07-01&limit=100&full_days=true`
- **THEN** 加载完成后 loadedRange.from <= 2026-07-01
- **THEN** 不再触发更多加载（已到达筛选范围起点）

### Requirement: 日历按需加载
当用户点击日历上的某天，如果该天不在已加载范围内，系统 SHALL 以整日模式加载该天的全部交易。

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
- **THEN** 系统请求 `GET /api/transactions?from=2026-06-15&to=2026-06-15&full_days=true`
- **THEN** 返回 06-15 的全部交易（整天）
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
- **THEN** 系统请求 `GET /api/transactions?from=2026-05-20&to=2026-05-20&full_days=true`
- **THEN** 返回 05-20 的全部交易
- **THEN** 页面显示该天交易

## REMOVED Requirements

### Requirement: 同一天数据膨胀
**Reason**: 整日模式下每一页天然落在完整的“天”边界上，不再需要把 `limit` 翻倍来补齐被切开的当天。
**Migration**: 由「初始加载最新交易」「滚动触发加载更早数据」「日历按需加载」的 `full_days=true` 整日分页取代；前端删除 `expandSameDay`。

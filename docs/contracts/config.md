# `config` 契约骨架

## 模块目标

定义应用最小配置结构、默认值、加载入口和校验规则。

## 当前实现基线

- `AppConfig`
- `load_default_config`
- `load_config_from_json_str`
- `validate_config`

## 需要固化的契约

### 1. 顶层配置结构

当前顶层字段：

- `timezone: String`
- `platforms: Vec<String>`（fixture 模式）
- `schedule: ScheduleConfig`
- `rss_feeds: Vec<RssFeedConfig>`（HTTP 模式）
- `hotlist_apis: Vec<HotlistApiConfig>`（HTTP 模式）
- `notification: NotificationConfig`

必填字段：

- `timezone`

默认值字段：

- `timezone` 默认值为 `Asia/Shanghai`
- `platforms` 默认值为空数组
- `schedule.collect / analyze / push` 默认值均为 `true`
- `schedule.window` 默认值为 `null`
- `rss_feeds` 和 `hotlist_apis` 默认值为空数组
- `notification` 默认值为 `NotificationConfig::default()`

可延后字段：

- 输出目标与输出开关
- 存储配置

### 2. 调度配置

当前字段：

- `schedule.collect: bool`
- `schedule.analyze: bool`
- `schedule.push: bool`
- `schedule.window.start_hour: u8`
- `schedule.window.end_hour: u8`
- `schedule.weekday.collect/analyze/push: Option<bool>`
- `schedule.weekday.window: Option<ScheduleWindowConfig>`
- `schedule.weekend.collect/analyze/push: Option<bool>`
- `schedule.weekend.window: Option<ScheduleWindowConfig>`

当前语义：

- `collect` 表示是否允许进入抓取阶段
- `analyze` 表示是否允许进入分析阶段
- `push` 表示是否允许进入推送阶段

当前保留边界：

- 时区相关调度窗口
- 工作日 / 周末覆盖规则
- 冷却周期等带状态依赖的复杂调度表达

非法配置的错误语义：

- 当前调度字段是布尔值，不引入额外校验错误
- 后续扩展复杂调度配置时，必须继续复用 `TrendRadarError::InvalidConfig`
- 错误消息需要包含具体字段名和触发条件

### 3. 数据源配置

热榜平台列表：

- `platforms` 表示热榜来源标识列表
- 当前允许空数组，空数组不视为非法配置

RSS 订阅列表：

- `rss_feeds` 包含 `RssFeedConfig { source_id, url }` 结构体数组
- 每个 RSS 订阅源需要 `source_id`（标识）和 `url`（feed 地址）
- 空数组不视为非法配置

热榜 API 列表：

- `hotlist_apis` 包含 `HotlistApiConfig { platform_id, url }` 结构体数组
- 每个热榜源需要 `platform_id`（平台标识）和 `url`（API 地址）
- 空数组不视为非法配置

输出开关与输出目标：

- 当前尚未进入实现
- 在进入 `report` 与 `app` 集成前，不阻塞 `config` 最小闭环

通知配置：

- `notification.enabled: bool`
- `notification.webhook_url: Option<String>`
- `notification.feishu_webhook_url: Option<String>`
- `notification.dingtalk_webhook_url: Option<String>`
- `notification.wecom_webhook_url: Option<String>`
- 所有通知渠道字段缺失时默认回落为 `None`

## 错误契约

解析错误：

- JSON 解析失败时返回 `TrendRadarError::InvalidConfig`
- 错误消息前缀固定为 `failed to parse config json:`

校验错误：

- `timezone` 为空时返回 `TrendRadarError::InvalidConfig`
- 当前错误消息为 `timezone must not be empty`
- 非法时区字符串返回 `TrendRadarError::InvalidConfig`
- 当前错误消息为 `timezone must be a valid IANA timezone`

错误消息要求：

- 首版至少要包含字段名或解析失败原因
- 不要求与旧系统错误文本完全一致

## 兼容要求

当前不兼容旧配置文件结构：

- Rust 首版不追求完整兼容旧配置
- 只保留主链路所需的最小配置子集

迁移策略：

- 先在契约文档中显式声明“保留 / 延后”的字段范围
- 后续如需兼容旧配置，应通过明确的映射规则扩展，而不是在 `AppConfig` 中一次性堆入所有历史字段

## 验证方式

fixture：

- [fixtures/system/config/minimal-valid.json](../../fixtures/system/config/minimal-valid.json)
- [fixtures/system/config/invalid-empty-timezone.json](../../fixtures/system/config/invalid-empty-timezone.json)
- [fixtures/system/config/invalid-unknown-timezone-window.json](../../fixtures/system/config/invalid-unknown-timezone-window.json)
- [fixtures/system/config/minimal-valid-http.json](../../fixtures/system/config/minimal-valid-http.json)

测试：

- `cargo test -p trendradar-config`
- `cargo test -p trendradar-app --test config_to_bootstrap`

快照：

- 当前不需要独立快照
- Wave 0 通过 fixture 驱动测试锁定行为

## 开放问题

- 顶层配置是否拆为模块化子结构体
- 配置加载是否只保留 JSON，还是预留 TOML / YAML
当前已进入的最小窗口表达：

- `schedule.window` 为可选字段
- `start_hour` / `end_hour` 代表本地时区小时窗口
- 当前使用半开区间 `[start_hour, end_hour)` 语义
- 当 `start_hour > end_hour` 时表示跨午夜窗口

当前校验规则：

- `start_hour` 与 `end_hour` 必须都在 `0..=23`
- `start_hour` 与 `end_hour` 不能相等
- 非法样例已覆盖相等小时与越界小时两类失败路径

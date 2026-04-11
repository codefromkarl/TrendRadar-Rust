# `domain` 契约骨架

## 模块目标

定义 Rust 内核共享的数据模型、错误类型和运行元数据。

## 当前实现基线

- `NewsItem`
- `RssItem`
- `RunContext`
- `TrendRadarError`

## 需要固化的契约

### 1. 统一内容模型

当前结构体：

- `NewsItem { title, source_id, rank }`
- `RssItem { title, feed_id, url }`

首版稳定字段：

- `title`：必填，保留原始标题文本，不在 `domain` 层做清洗
- `source_id` / `feed_id`：必填，作为来源标识，供分析、存储和输出复用
- `rank`：热榜条目必填，语义为来源内自然排名，排名越小代表位置越高
- `url`：RSS 条目必填，保留抓取到的原始链接

可后续扩展字段：

- 发布时间
- 摘要 / 描述
- 统一条目 ID
- 原始 payload 的调试字段

序列化策略：

- 首版统一使用 `serde::{Serialize, Deserialize}`
- 不自定义字段重命名，默认采用 Rust 字段名作为 JSON 键
- 只有进入对外输出契约后，才允许为兼容性引入显式 rename 规则

### 2. 运行元数据

当前字段：

- `started_at: DateTime<Utc>`
- `timezone: String`

时区与时间语义：

- `started_at` 始终使用 UTC 存储，避免各模块各自做本地时间换算
- `timezone` 保留配置中的时区字符串，作为调度和输出层解释上下文

与输出层的传递方式：

- `RunContext` 由 `app` 在运行入口创建
- `report` 负责决定哪些运行元数据进入结构化输出
- `domain` 不负责格式化显示逻辑

### 3. 共享错误

当前错误分类：

- `TrendRadarError::InvalidConfig { message }`
- `TrendRadarError::Storage { message }`

首版稳定要求：

- 配置校验失败统一进入 `InvalidConfig`
- SQLite 建库、写入、查询失败统一进入 `Storage`
- 错误消息必须包含可定位的字段或失败原因

后续预留分类：

- 抓取错误
- 解析错误
- 存储错误
- 输出错误

透传规则：

- `domain` 只定义共享错误类型，不直接做 I/O 层包装
- `config` 可以直接返回 `InvalidConfig`
- 其他模块在首版进入实现时，应优先新增结构化错误变体，而不是字符串拼接塞进现有配置错误

## 兼容要求

不要求与旧 Python 模型字段完全一致：

- Rust 首版优先保证内核模型清晰、稳定、可测试
- 只有进入对外契约基线的字段才考虑兼容

当前替代映射原则：

- 热榜类来源统一收敛到 `NewsItem`
- RSS 类来源统一收敛到 `RssItem`
- 旧系统中的展示辅助字段、缓存字段和运行环境分支字段不进入 `domain`

## 验证方式

fixture：

- [fixtures/system/domain/news-item.json](../../fixtures/system/domain/news-item.json)
- [fixtures/system/domain/rss-item.json](../../fixtures/system/domain/rss-item.json)
- [fixtures/system/domain/run-context.json](../../fixtures/system/domain/run-context.json)

测试：

- `cargo test -p trendradar-domain`
- 固定 `NewsItem`、`RssItem`、`RunContext` 的序列化 / 反序列化往返测试

快照：

- 当前不需要
- 结构由 fixture roundtrip 断言固定

## 开放问题

- `NewsItem` 与 `RssItem` 是否需要进一步收敛为统一条目模型
- 错误类型是否按配置、抓取、解析、存储、输出分层

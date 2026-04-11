# `fetch` 契约骨架

## 模块目标

定义热榜源与 RSS 源的抓取输入、统一归一化输出和错误分类。

## 当前实现基线

- `Fetcher` trait
- `FixtureHotlistFetcher` — 基于 fixture 文件的热榜抓取
- `FixtureRssFetcher` — 基于 fixture 文件的 RSS 抓取
- `HttpRssFetcher` — 基于真实 HTTP 的 RSS feed 抓取
- `HttpHotlistFetcher` — 基于真实 HTTP 的热榜 API 抓取
- `FetchError` — 含网络、HTTP 状态、解析三类错误

## 需要固化的契约

### 1. 数据源输入

- RSS 源最小参数（fixture）：
  `source_id` 与 fixture 路径
- RSS 源最小参数（HTTP）：
  `source_id` 与 feed URL
- 热榜源最小参数（fixture）：
  `platform_id` 与 fixture 路径
- 热榜源最小参数（HTTP）：
  `platform_id` 与 API URL
- 超时、重试、限流配置：
  当前 HTTP adapter 使用 `reqwest` 默认配置，后续可扩展

### 2. 归一化输出

- 输出到 `domain` 的目标模型：
  `NewsItem`
- 字段映射规则：
  热榜直接映射 `title` 与 `rank`，并把 `platform_id` 写入 `source_id`
- 缺失字段处理策略：
  RSS 以输入顺序生成 `rank = 1..n`，并把 `source_id` 固定为订阅源标识
- HTTP RSS adapter 跳过无 title 的 item

### 3. 错误分类

- 网络错误（连接失败、超时、DNS）：
  `FetchError::Network`
- HTTP 状态错误（4xx / 5xx）：
  `FetchError::Http`
- 解析错误（fixture / RSS XML / JSON）：
  `FetchError::ParseFixture`（fixture）、`FetchError::ParseResponse`（HTTP）
- 限流错误：
  当前不单独分类，后续可通过 HTTP 429 扩展

## 兼容要求

- 是否需要保留旧系统源标识：
  当前保留最小来源标识即可
- 是否允许首版只支持部分来源：
  允许，Wave 1 只要求一个 RSS 源和一个热榜源打通

## 验证方式

- fixture：
  `fixtures/system/fetch/rss-rust-blog.json`
  `fixtures/system/fetch/hotlist-weibo.json`
  `fixtures/system/fetch/empty-rss.json`
- 测试：
  `cargo test -p trendradar-fetch`（含 fixture adapter 和 HTTP adapter 测试）
- HTTP adapter 测试使用 `mockito` mock server 隔离
- 错误边界：
  非法 fixture 解析失败时返回 `FetchError::ParseFixture`
  HTTP 错误返回 `FetchError::Http`
  网络不可达返回 `FetchError::Network`
  响应内容解析失败返回 `FetchError::ParseResponse`
- 空输入边界：
  合法空 RSS fixture / 空 RSS channel / 空 JSON 数组均返回空集合
- 快照：
  当前不需要，结构由测试断言固定

## 已决策

- `Fetcher` trait 返回 `Result<Vec<NewsItem>>`，统一结果类型已采用
- RSS 与热榜 adapter 各自独立映射到 `NewsItem`，不共享中间模型
- HTTP adapter 复用 `HotlistFixtureItem` 结构体解析远程 JSON

## 待补充决策

- HTTP adapter 是否需要配置超时、重试等参数
- 是否需要为不同平台热榜实现不同的响应格式解析器

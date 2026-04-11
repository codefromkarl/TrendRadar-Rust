# `fetch` 契约骨架

## 模块目标

定义热榜源与 RSS 源的抓取输入、统一归一化输出和错误分类。

## 当前实现基线

- `Fetcher` trait
- `FixtureHotlistFetcher`
- `FixtureRssFetcher`
- `FetchError`

## 需要固化的契约

### 1. 数据源输入

- RSS 源最小参数：
  `source_id` 与 fixture 路径
- 热榜源最小参数：
  `platform_id` 与 fixture 路径
- 超时、重试、限流配置：
  Wave 1 的 fixture adapter 不涉及真实网络，暂不引入

### 2. 归一化输出

- 输出到 `domain` 的目标模型：
  `NewsItem`
- 字段映射规则：
  热榜 fixture 直接映射 `title` 与 `rank`，并把 `platform_id` 写入 `source_id`
- 缺失字段处理策略：
  RSS fixture 以输入顺序生成 `rank = 1..n`，并把 `source_id` 固定为订阅源标识

### 3. 错误分类

- 配置错误：
  当前未进入真实配置校验
- 网络错误：
  Wave 1 fixture adapter 不涉及
- 解析错误：
  进入 `FetchError::ParseFixture`
- 限流错误：
  Wave 1 fixture adapter 不涉及

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
  `cargo test -p trendradar-fetch`
- 错误边界：
  非法 fixture 解析失败时返回 `FetchError::ParseFixture`
- 空输入边界：
  合法空 RSS fixture 返回空集合
- 快照：
  当前不需要，结构由测试断言固定

## 待补充决策

- `Fetcher` trait 是否返回统一结果类型而不是裸集合
- RSS 与热榜 adapter 是否共享中间模型

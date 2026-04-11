# `fetch` 实施骨架

## 目标

先打通一个 RSS 源和一个热榜源到统一模型的最小抓取链路。

## 前置依赖

- `domain` 模型稳定
- `config` 数据源配置稳定

## 输入与输出

- 输入：数据源配置
- 输出：统一 `domain` 模型或明确错误分类

## 本轮范围

- 已实现 RSS fixture adapter
- 已实现热榜 fixture adapter
- 已实现最小归一化映射
- 已实现 HTTP RSS adapter（`HttpRssFetcher`，使用 `reqwest` + `rss` crate）
- 已实现 HTTP 热榜 adapter（`HttpHotlistFetcher`，使用 `reqwest` + JSON 解析）
- 已扩展 `FetchError` 含 `Network`、`Http`、`ParseResponse` 三个网络相关变体
- 已补 HTTP adapter 的 mockito 隔离测试（10 条）

## 暂不处理

- 全量来源接入
- 复杂重试编排
- HTTP 超时与限流配置
- 不同平台热榜的差异化响应格式

## 建议子任务

- `rss` 子模块
- `hotlist` 子模块
- 统一错误映射

## 完成定义

- 至少一个 RSS 源打通
- 至少一个热榜源打通
- 成功结果进入统一模型

## 当前进展

- 已通过 `FixtureRssFetcher` 打通一个 RSS 源到 `NewsItem`
- 已通过 `FixtureHotlistFetcher` 打通一个热榜源到 `NewsItem`
- 已补非法 RSS fixture 的解析失败断言
- 已补合法空 RSS fixture 的空集合断言
- 已实现 `HttpRssFetcher`：通过 `reqwest::blocking` 获取 RSS feed，通过 `rss::Channel` 解析 XML
- 已实现 `HttpHotlistFetcher`：通过 `reqwest::blocking` 获取 JSON API，复用 `HotlistFixtureItem` 结构体解析
- 已提取 `http_get_text` 共享函数统一处理网络错误和 HTTP 状态错误
- 已补 10 条 mockito 隔离测试：正常解析、空 channel/数组、HTTP 错误、XML/JSON 解析错误、网络不可达

## 验证命令

```bash
cargo test -p trendradar-fetch
```

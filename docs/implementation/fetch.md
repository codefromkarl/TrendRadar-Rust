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

## 暂不处理

- 全量来源接入
- 复杂重试编排

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
- 真实网络抓取、重试与限流仍留待后续阶段

## 验证命令

```bash
cargo test -p trendradar-fetch
```

# Wave 3 fetch error boundary

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：`fetch` 非法 fixture 解析边界
- 目标：固定 fixture adapter 在坏输入下的错误语义，避免系统链路退回成模糊失败

## 本次完成内容

- 新增 `fixtures/system/fetch/invalid-rss.json`
- 为 `FixtureRssFetcher` 新增解析失败测试
- 固定错误消息包含 `failed to parse fetch fixture` 与 fixture 路径
- 同步更新 `fetch` 契约、实施文档和验收矩阵

## 阶段结论

`fetch` 现在同时覆盖了正常 RSS、正常热榜和非法 RSS fixture 三条基础路径，Wave 3 的核心 crate 边界样例已经更完整。

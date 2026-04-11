# Wave 3 domain serialization

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：`domain` 序列化 fixture
- 目标：补齐 `NewsItem`、`RssItem`、`RunContext` 的最小序列化 / 反序列化验证，关闭验收矩阵里 `domain` 的遗留缺口

## 本次完成内容

- 新增 `fixtures/system/domain/news-item.json`
- 新增 `fixtures/system/domain/rss-item.json`
- 新增 `fixtures/system/domain/run-context.json`
- 为 `trendradar-domain` 增加 `serde_json` 测试依赖
- 补齐三个模型的 fixture roundtrip 测试
- 同步更新 `fixtures/README`、`domain` 契约、实施文档和验收矩阵

## 阶段结论

`domain` 不再只是“字段定义已落地”，而是已经有可复查的 JSON 形状证据，后续其他 crate 可以直接以这些 fixture 作为共享模型基线。

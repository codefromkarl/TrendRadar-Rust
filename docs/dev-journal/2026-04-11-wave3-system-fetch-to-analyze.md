# Wave 3 system fetch to analyze

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 组合链路
- 目标：把抓取结果直接接到分析层，验证 `fetch -> domain -> analyze` 的跨 crate 组合行为

## 本次完成内容

- 新增 `tests/system/fetch_to_analyze.rs`
- 复用热榜与 RSS fixture，组合出统一新闻条目集合
- 断言排序结果和来源聚合结果都符合预期
- 同步更新 `tests/README.md`

## 阶段结论

Wave 3 的根级系统测试现在开始覆盖“组合后的业务流”而不只是单层边界或单条链路。

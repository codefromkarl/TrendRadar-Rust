# Wave 3 system fetch to analyze mixed

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 混合空源路径
- 目标：验证部分来源为空时，`fetch -> domain -> analyze` 仍会对可用来源的数据稳定产出分析结果

## 本次完成内容

- 在 `tests/system/fetch_to_analyze.rs` 中新增混合空/非空来源系统测试
- 复用 `empty-hotlist.json` 与 `rss-rust-blog.json`
- 固定仅剩 RSS 数据时仍能产出排序和来源聚合结果
- 同步更新 `tests/README.md`

## 阶段结论

根级 `fetch -> analyze` 系统测试现在同时覆盖完整成功、全部为空、部分为空和错误路径。

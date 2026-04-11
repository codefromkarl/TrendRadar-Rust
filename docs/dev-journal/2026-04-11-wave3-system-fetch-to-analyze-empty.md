# Wave 3 system fetch to analyze empty

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 空组合路径
- 目标：补齐双来源都为空时 `fetch -> domain -> analyze` 的系统行为，验证排序和聚合仍稳定返回空结果

## 本次完成内容

- 在 `tests/system/fetch_to_analyze.rs` 中新增空组合系统测试
- 复用 `empty-hotlist.json` 与 `empty-rss.json`
- 固定排序结果和来源聚合结果都为空
- 同步更新 `tests/README.md`

## 阶段结论

根级 `fetch -> analyze` 系统测试现在同时覆盖正向组合和空组合两条路径。

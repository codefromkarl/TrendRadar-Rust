# Wave 3 system fetch analyze partial failure

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 的部分成功后整体中断
- 目标：补齐“前一个 source 已抓取成功、后一个 source 失败”时的链路语义，验证系统不会偷偷分析已成功的那一半数据

## 本次完成内容

- 在 `tests/system/fetch_to_analyze.rs` 中新增部分抓取成功后整体中断的系统测试
- 复用 `hotlist-weibo.json` 和 `invalid-rss.json`
- 固定后续 source 失败时整条 `fetch -> analyze` 链路直接返回错误
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`

## 阶段结论

`fetch -> analyze` 现在不仅覆盖“全部成功”与“单点失败”，也覆盖“部分已成功但随后失败”的整体语义。这样链路级错误处理就不再停留在单个 fetcher 的局部行为，而是提升到了跨 crate 组合层。

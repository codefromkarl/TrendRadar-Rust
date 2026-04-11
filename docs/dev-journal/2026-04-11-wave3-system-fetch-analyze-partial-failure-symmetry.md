# Wave 3 system fetch analyze partial failure symmetry

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 的部分成功后整体中断对称样例
- 目标：补齐“先 RSS 成功、后 hotlist 失败”这条对称路径，验证整体中断语义不依赖具体来源顺序

## 本次完成内容

- 在 `tests/system/fetch_to_analyze.rs` 中新增 RSS 先成功、hotlist 后失败的系统测试
- 复用 `rss-rust-blog.json` 和 `invalid-hotlist.json`
- 固定后续 source 失败时整条 `fetch -> analyze` 链路仍直接返回错误
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`

## 阶段结论

`fetch -> analyze` 现在对“部分抓取成功后整体中断”的语义已经具备对称样例：无论先成功的是 RSS 还是热榜，只要后续来源失败，整条链路都会停止。这让错误传播规则不再依赖来源顺序。

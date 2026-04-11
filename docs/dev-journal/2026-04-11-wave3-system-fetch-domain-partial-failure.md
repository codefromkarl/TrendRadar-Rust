# Wave 3 system fetch domain partial failure

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> domain` 的部分成功后整体中断
- 目标：补齐“先 RSS 成功、后 hotlist 失败”时的链路语义，验证 `fetch -> domain` 也不会偷偷保留已成功的一半数据

## 本次完成内容

- 在 `tests/system/fetch_to_domain.rs` 中新增部分抓取成功后整体中断的系统测试
- 复用 `rss-rust-blog.json` 和 `invalid-hotlist.json`
- 固定后续 source 失败时整条 `fetch -> domain` 链路直接返回错误
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`

## 阶段结论

现在 `fetch -> domain` 也具备了“部分成功后整体中断”的系统证据，而不是只在 `fetch -> analyze` 上存在这类语义。这样跨 crate 的错误传播规则在更上游的组合层也被锁住了。

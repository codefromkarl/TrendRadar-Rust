# Wave 3 system fetch analyze same-rank order

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 的同 rank 稳定排序
- 目标：补齐真实抓取输出进入排序后的同 rank 行为，验证 tie-break 不只在手工 fixture 上成立，也在实际抓取归一化结果上成立

## 本次完成内容

- 在 `tests/system/fetch_to_analyze.rs` 中新增真实抓取输出上的同 rank 稳定排序系统测试
- 复用 `hotlist-weibo.json` 和 `rss-rust-blog.json`
- 固定两个 rank=1 条目的标题顺序，验证抓取归一化后进入 `analyze` 仍保持稳定 tie-break
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`

## 阶段结论

`fetch -> analyze` 现在不仅证明了链路能跑通，也证明了真实抓取输出进入排序后仍具备稳定的同 rank 行为。这让非 `app` richer case 继续从“链路是否打通”推进到“排序语义是否可复查”。

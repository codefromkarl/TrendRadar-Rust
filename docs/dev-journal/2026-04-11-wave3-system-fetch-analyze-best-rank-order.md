# Wave 3 system fetch analyze best-rank order

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 的来源聚合 best-rank 排序
- 目标：补齐来源条目数打平时的聚合排序行为，验证 `group_news_by_source` 在真实抓取输出上会按 `best_rank` 继续决定先后

## 本次完成内容

- 新增 `fixtures/system/fetch/hotlist-low-ranks.json`
- 在 `tests/system/fetch_to_analyze.rs` 中新增来源聚合 best-rank 排序系统测试
- 固定两组来源条目数都为 2 时，`best_rank = 1` 的 RSS 分组排在 `best_rank = 3` 的热榜分组前面
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`

## 阶段结论

`fetch -> analyze` 现在不仅能证明条目排序稳定，也能证明来源聚合在真实抓取输出上具备稳定的二级排序语义。这让 Wave 3 的非 `app` richer case 继续向“聚合结果是否可预测”推进。

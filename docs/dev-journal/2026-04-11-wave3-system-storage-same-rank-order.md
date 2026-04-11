# Wave 3 system storage same-rank order

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `storage -> report` 的同 rank 稳定排序
- 目标：补齐 rank 打平时的系统排序行为，验证报告输出在相同 rank 下仍按 `source_id + title` 稳定排列

## 本次完成内容

- 在 `tests/system/storage_to_report.rs` 中新增同 rank 稳定排序系统测试
- 使用内存 SQLite 手工写入三条相同 rank、不同 `source_id/title` 的记录
- 固定 `report` 输出在同 rank 时按 `source_id`、再按 `title` 排序
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`

## 阶段结论

`storage -> report` 现在不仅证明了乱序写入后仍能按 rank 排序，也证明了 rank 打平时的二级排序是稳定且可复查的。这让非 `app` richer case 开始从“有无数据”推进到“输出顺序是否可预测”。

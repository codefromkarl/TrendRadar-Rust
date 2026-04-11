# Wave 3 system storage same-title same-rank separate

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `storage -> report` 的同标题同 rank 跨来源分离
- 目标：补齐“同标题、不同来源、相同 rank”时的系统行为，验证主键边界在完全相同优先级下仍然以 `source_id` 为分界

## 本次完成内容

- 在 `tests/system/storage_to_report.rs` 中新增同标题同 rank 但跨来源仍保持两条输出的系统测试
- 使用内存 SQLite 写入两条 title、rank 相同但 `source_id` 不同的记录
- 固定 `report` 输出保留两条 item，且按 `source_id` 稳定排序
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`

## 阶段结论

`storage -> report` 现在已经把“去重”和“分离”的边界都补到了相同 rank 场景：同源同标题会合并，不同来源同标题不会。这样来源主键语义在系统层就更完整了。

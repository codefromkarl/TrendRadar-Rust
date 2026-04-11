# Wave 3 system storage equal-rank dedup

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `storage -> report` 的相同 rank 去重
- 目标：补齐同源同标题且 rank 相等时的系统行为，验证 `storage` 在完全相同优先级下仍只保留一条记录

## 本次完成内容

- 在 `tests/system/storage_to_report.rs` 中新增相同 rank 重复写入仍只渲染一条的系统测试
- 使用内存 SQLite 写入两条完全相同的新闻记录
- 固定 `report` 输出只包含一条对应 item
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`

## 阶段结论

`storage -> report` 现在不仅证明了“更优 rank 会覆盖更差 rank”，也证明了“完全相同 rank 的重复写入不会产生重复输出”。这让去重语义在系统层更完整，也更接近真实运行时的稳定预期。

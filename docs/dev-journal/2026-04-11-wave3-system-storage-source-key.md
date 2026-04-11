# Wave 3 system storage source key

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `storage -> report` 的来源主键语义
- 目标：把 `(source_id, title)` 作为主键的语义提升到根级系统测试，验证同标题不同来源不会被错误去重

## 本次完成内容

- 在 `tests/system/storage_to_report.rs` 中新增同标题不同来源系统测试
- 固定 `report` 输出里保留两条不同来源记录
- 同步更新 `tests/README.md`

## 阶段结论

根级 `storage -> report` 系统测试现在已经明确覆盖“同标题同来源去重”和“同标题不同来源保留分离”两条关键主键语义。

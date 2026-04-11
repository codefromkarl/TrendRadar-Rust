# Snapshots

如果启用 `insta`，这里用于存放跨 crate 的快照结果。

## 使用约定

- 优先给根级 `tests/system/` 下结构稳定、适合审查的输出使用快照
- 快照名称应直接对应测试函数，避免使用临时人名或日期命名
- 更新快照时，应与对应 fixture / 契约 / 测试说明同轮出现，不能只改快照不解释原因
- `insta` 生成的 `.pending-snap` 或其他临时工件不属于正式产物，应在同轮验证后清理

## 当前适用场景

- `storage_to_report` 的结构化 JSON 输出
- `app_pipeline_modes` 的全链路 JSON 输出

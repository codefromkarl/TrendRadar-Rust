# Wave 3 app window integration

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：`app` 接入 `schedule.window`
- 目标：让 `app` 在完整 pipeline 中真正消费时间窗口配置，而不是只在 `schedule` crate 内部停留为局部能力

## 本次完成内容

- 在 `trendradar-app` 中引入基于 `started_at + timezone` 的本地小时计算
- 当配置存在 `schedule.window` 时，改为调用 `decision_from_config_at`
- 在根级 `tests/system/app_pipeline_modes.rs` 中新增窗口外阻断系统测试
- 同步更新实施文档、验收矩阵、并行迁移总方案和测试说明

## 阶段结论

Wave 3 现在不仅在 crate 层验证窗口逻辑，也已经把窗口判定真正接入 `app` 的完整 pipeline，系统层行为与调度契约开始对齐。

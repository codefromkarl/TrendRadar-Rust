# Wave 3 system app window allow error propagation

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的窗口放行错误传播
- 目标：补齐 `schedule.window` 允许执行时的损坏 source 行为，验证动态门控在放行路径上也不会吞掉上游解析错误

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增窗口放行时传播损坏 source 错误的系统测试
- 复用 `schedule/window-daytime.json` 和 `fetch/invalid-rss.json`
- 固定本地时间落在窗口内时，损坏 source 会立刻报错
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

现在动态窗口门控已经形成完整对照：

- 窗口阻断时，损坏 source 不会被解析
- 窗口放行时，损坏 source 会立刻上浮

这进一步证明 `app` 只是消费上游调度决策，并按决策决定是否触碰 source，而不是在内部重写一套时间规则。

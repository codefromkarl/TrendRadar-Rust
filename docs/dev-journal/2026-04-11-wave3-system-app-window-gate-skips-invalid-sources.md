# Wave 3 system app window gate skips invalid sources

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的动态窗口门控优先级
- 目标：补齐 `schedule.window` 计算出阻断决策时对损坏 source 的系统行为，验证动态门控同样先于 source 解析生效

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增窗口阻断时跳过损坏 source 的系统测试
- 复用 `schedule/window-daytime.json` 和 `invalid-hotlist.json`、`invalid-rss.json`
- 固定窗口外时间下 pipeline 直接返回全空状态，而不会触发上游解析错误
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

现在 `app` 的“先门控、后解析”证据已经同时覆盖静态阶段开关和动态窗口门控两种场景。后续就算继续扩充时间规则，也更难把 source 解析提前到应被阻断的路径里。

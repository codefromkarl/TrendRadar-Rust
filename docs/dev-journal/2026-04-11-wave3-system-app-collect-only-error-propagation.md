# Wave 3 system app collect-only error propagation

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的 collect-only 错误传播
- 目标：补齐 `collect=true` 但 `analyze=false`、`push=false` 时的损坏 source 行为，验证 source 解析是否发生只取决于抓取阶段是否启用

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增 `collect-only` 传播损坏 source 错误的系统测试
- 复用 `config/collect-only.json` 与 `fetch/invalid-rss.json`
- 固定即使后续阶段关闭，只要 `collect=true`，损坏 source 仍会立刻报错
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

现在 `app` 关于 source 解析顺序的系统证据已经形成对照组：

- `collect=false` 或窗口阻断时，损坏 source 不会被解析
- `collect=true` 时，即使 `analyze` / `push` 关闭，损坏 source 也会立刻暴露

这能更直接地证明 `app` 只按阶段门控做编排，没有把解析副作用和后续阶段状态混在一起。

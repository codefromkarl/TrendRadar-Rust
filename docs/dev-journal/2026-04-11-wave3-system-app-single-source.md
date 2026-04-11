# Wave 3 system app single source

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的单来源闭环
- 目标：补齐只提供一个 fixture source 时的系统行为，验证 `app` 不依赖“双来源固定组合”，而是按实际输入列表做薄编排

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增单来源全链路系统测试
- 固定 `minimal-valid.json` 配置下只传入一个 RSS source
- 断言 `collected_items`、`ranked_items`、`stored_items` 均为 2 条，`source_summaries` 为 1 组
- 通过快照固定部分报告输出，证明 `app` 会基于现有 source 列表产出稳定结果
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

根级 `app` 系统测试现在同时覆盖空来源、单来源和双来源三类输入形态。`app` 的职责依然只是编排现有模块，没有把“必须同时存在哪些 source”之类的业务规则吸入自身。

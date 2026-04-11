# Wave 3 system app collect only

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的 collect-only 模式
- 目标：把 `collect-only` 阶段组合提升到根级系统测试，验证完整链路只采集和落库，不会误跑分析或输出

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增 collect-only 系统测试
- 固定 `collect=true, analyze=false, push=false` 时只保留采集与落库结果
- 同步更新 `tests/README.md` 与 `tests/system/README.md`

## 阶段结论

根级 `app` 系统模式现在已经覆盖五条高信号阶段组合，系统层对薄编排行为的证据更完整。

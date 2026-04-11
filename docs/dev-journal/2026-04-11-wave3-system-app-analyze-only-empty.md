# Wave 3 system app analyze only empty

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的 analyze-only-empty 模式
- 目标：把 `collect=false, analyze=true, push=false` 提升到根级系统测试，验证完整链路不会在空输入上产生分析结果或报告

## 本次完成内容

- 新增 `fixtures/system/config/analyze-only-empty.json`
- 在 `tests/system/app_pipeline_modes.rs` 中新增 analyze-only-empty 系统测试
- 固定该模式下采集、分析结果、落库和报告都为空

## 阶段结论

根级 `app` 系统模式现在已经覆盖六类高信号阶段组合。

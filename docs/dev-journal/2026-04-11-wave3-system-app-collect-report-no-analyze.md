# Wave 3 system app collect report no analyze

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的“有报告无分析”模式
- 目标：把 `collect-and-report-no-analyze` 阶段组合提升到根级系统测试，验证完整链路会输出报告但不会生成分析结果

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增系统测试
- 固定 `collect=true, analyze=false, push=true` 时的报告快照与空分析结果
- 同步更新 `tests/README.md` 与 `tests/system/README.md`

## 阶段结论

根级 `app` 系统模式现在至少覆盖两条高信号组合：空报告模式，以及“有报告无分析”模式。

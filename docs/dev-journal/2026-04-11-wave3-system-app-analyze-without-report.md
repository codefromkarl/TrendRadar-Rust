# Wave 3 system app analyze without report

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的“有分析无报告”模式
- 目标：把 `analyze-without-report` 阶段组合提升到根级系统测试，验证完整链路会保留分析结果但不会渲染报告

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增系统测试
- 断言采集、排序、来源聚合和落库结果存在，但 `report_json` 为 `None`
- 同步更新 `tests/README.md` 与 `tests/system/README.md`

## 阶段结论

根级 `app` 系统模式现在已经覆盖空报告、“有报告无分析”、“有分析无报告”三类高信号阶段组合。

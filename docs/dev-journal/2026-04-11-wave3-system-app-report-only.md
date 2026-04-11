# Wave 3 system app report-only

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 空报告系统测试
- 目标：把 `report-only-empty` 阶段组合从 `crates/app/tests` 提升到根级系统测试，验证完整链路在工作区层也能稳定输出空报告

## 本次完成内容

- 新增 `tests/system/app_pipeline_modes.rs`
- 在根工作区测试中引入 `trendradar-app`
- 固定 `report-only-empty` 组合下的空报告 JSON 快照
- 同步更新 `tests/README.md` 与 `tests/system/README.md`

## 阶段结论

Wave 3 的根级系统测试现在不只覆盖 `fetch`、`analyze`、`storage -> report`，也开始直接覆盖 `app` 的完整编排模式。

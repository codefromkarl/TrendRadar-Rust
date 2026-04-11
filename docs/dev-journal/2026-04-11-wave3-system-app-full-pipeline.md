# Wave 3 system app full pipeline

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 正向最小闭环
- 目标：把 `minimal-valid` 的完整 fixture pipeline 提升到根级系统测试，验证工作区级别的结构化输出快照

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增正向最小闭环系统测试
- 固定 `config -> fetch -> analyze -> storage -> report` 的根级 JSON 快照
- 同步更新 `tests/README.md` 与 `tests/system/README.md`

## 阶段结论

根级 `app` 系统测试现在不只覆盖阶段组合，也覆盖了最小正向全链路本身。

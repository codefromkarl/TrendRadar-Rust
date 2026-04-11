# Wave 3 system app disabled all

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 全关闭模式
- 目标：把 `disabled-all` 阶段组合提升到根级系统测试，验证完整链路在系统层保持空状态而不会误跑任意阶段

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增全关闭系统测试
- 固定 `collect=false, analyze=false, push=false` 时所有结果集合为空，且没有报告输出
- 同步更新 `tests/README.md` 与 `tests/system/README.md`

## 阶段结论

根级 `app` 系统模式现在至少覆盖空报告、“有报告无分析”、“有分析无报告”和全关闭四条高信号阶段组合。

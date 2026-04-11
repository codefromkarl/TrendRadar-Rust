# Wave 3 system app default schedule

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的缺省 `schedule` 回退
- 目标：把 `schedule` 字段缺失时的默认值回退提升到完整系统链路，验证 `app` 不依赖显式 `schedule` 也能跑通最小闭环

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增缺省 `schedule` 全链路系统测试
- 复用 `fixtures/system/config/minimal-valid-no-schedule.json`
- 固定在缺省 `schedule` 下仍可完成 `config -> fetch -> analyze -> storage -> report`
- 同步更新 `tests/README.md`

## 阶段结论

根级 `app` 系统测试现在不仅验证显式阶段配置，也验证缺省 `schedule` 回退后的完整闭环。

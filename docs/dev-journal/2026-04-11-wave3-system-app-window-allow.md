# Wave 3 system app window allow

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的窗口内放行路径
- 目标：和“窗口外阻断”形成对称样例，验证 `schedule.window` 在 `app` 全链路里会在窗口内放行完整 pipeline

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增窗口内放行系统测试
- 复用 `fixtures/system/schedule/window-daytime.json`
- 固定窗口内时刻下采集、分析、落库和报告都可执行
- 同步更新 `tests/README.md`

## 阶段结论

根级 `app` 系统测试现在对窗口逻辑具备“窗口内放行 / 窗口外阻断”的成对证据。

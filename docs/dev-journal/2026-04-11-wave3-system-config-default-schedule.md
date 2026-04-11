# Wave 3 system config default schedule

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级缺省 `schedule` 成功路径
- 目标：把 `schedule` 缺失时的默认值回退提升到根级系统测试，补齐配置 / 调度的默认成功样例

## 本次完成内容

- 新增 `fixtures/system/config/minimal-valid-no-schedule.json`
- 在 `tests/system/config_schedule_errors.rs` 中新增默认值回退系统测试
- 固定缺失 `schedule` 时三个阶段开关都为 `true`
- 同步更新 `tests/README.md`

## 阶段结论

根级 `config + schedule` 系统测试现在同时覆盖默认值回退、窗口成功路径和配置错误路径。

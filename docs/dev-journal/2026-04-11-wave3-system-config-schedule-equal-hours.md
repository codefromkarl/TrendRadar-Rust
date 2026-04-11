# Wave 3 system config schedule equal hours

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `config + schedule` 相等小时错误路径
- 目标：把 `start_hour == end_hour` 的非法窗口错误提升到根级系统测试，补齐配置 / 调度边界的第二条主要失败路径

## 本次完成内容

- 在 `tests/system/config_schedule_errors.rs` 中新增相等小时系统测试
- 复用 `fixtures/system/schedule/invalid-window-equal-hours.json`
- 固定错误消息为 `invalid config: schedule window start_hour and end_hour must not be equal`
- 同步更新 `tests/README.md`

## 阶段结论

根级 `config + schedule` 系统测试现在同时覆盖越界小时和相等小时两类非法窗口配置。

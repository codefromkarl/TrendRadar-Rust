# Wave 3 system config schedule error

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `config + schedule` 错误路径
- 目标：把非法时间窗口配置的失败提升到根级系统测试，验证配置与调度边界在系统层的错误语义仍然稳定

## 本次完成内容

- 新增 `tests/system/config_schedule_errors.rs`
- 复用 `fixtures/system/schedule/invalid-window-out-of-range.json`
- 固定越界小时配置返回 `invalid config: schedule window hours must be between 0 and 23`
- 同步更新 `tests/README.md`

## 阶段结论

根级系统测试不再只覆盖成功路径和运行组合，也开始直接覆盖配置 / 调度边界错误。

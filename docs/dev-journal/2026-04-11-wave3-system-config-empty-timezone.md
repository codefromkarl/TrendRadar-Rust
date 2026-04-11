# Wave 3 system config empty timezone

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级空时区配置错误路径
- 目标：把 `invalid-empty-timezone.json` 的失败提升到根级系统测试，补齐配置基线错误在系统层的验证

## 本次完成内容

- 在 `tests/system/config_schedule_errors.rs` 中新增空时区系统测试
- 复用 `fixtures/system/config/invalid-empty-timezone.json`
- 固定错误消息为 `invalid config: timezone must not be empty`
- 同步更新 `tests/README.md`

## 阶段结论

根级系统错误测试现在已经覆盖基础配置错误和两类时间窗口错误。

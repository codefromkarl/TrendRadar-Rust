# Wave 3 config timezone validation

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：配置层时区校验下沉
- 目标：把未知时区错误从 `app` 运行时下沉到 `config` 校验阶段，保持错误定位更早、更稳定

## 本次完成内容

- 在 `trendradar-config` 中引入 IANA 时区合法性校验
- 为 `invalid-unknown-timezone-window.json` 增加 crate 级与根级系统测试
- 固定错误消息为 `timezone must be a valid IANA timezone`
- 同步更新配置契约和测试说明

## 阶段结论

`timezone` 现在不再只是非空校验，而是具备了合法时区字符串校验，窗口调度在系统层的错误语义更一致。

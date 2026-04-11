# Wave 3 system schedule window daytime block

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级白天窗口外禁止路径
- 目标：补齐 `window-daytime.json` 的窗口外禁止判定，让根级系统测试对白天窗口也同时具备正向和反向样例

## 本次完成内容

- 在 `tests/system/config_schedule_errors.rs` 中新增白天窗口外禁止测试
- 复用 `fixtures/system/schedule/window-daytime.json`
- 固定 `local_hour = 20` 时三个阶段都为 `false`
- 同步记录到开发日志

## 阶段结论

根级 `config + schedule` 系统测试现在对白天窗口与跨午夜窗口都具备“窗口内允许 / 窗口外禁止”的对称样例。

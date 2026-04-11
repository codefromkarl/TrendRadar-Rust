# Wave 3 system schedule window success

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `config + schedule` 窗口成功路径
- 目标：把白天窗口与跨午夜窗口的成功判定提升到根级系统测试，补齐配置 / 调度在系统层的正向样例

## 本次完成内容

- 在 `tests/system/config_schedule_errors.rs` 中新增白天窗口和跨午夜窗口系统测试
- 复用 `fixtures/system/schedule/window-daytime.json` 与 `fixtures/system/schedule/window-overnight.json`
- 固定显式上下文下的窗口内 / 窗口外决策结果
- 同步更新 `tests/README.md`

## 阶段结论

根级 `config + schedule` 系统测试现在同时覆盖配置错误路径和窗口成功路径。

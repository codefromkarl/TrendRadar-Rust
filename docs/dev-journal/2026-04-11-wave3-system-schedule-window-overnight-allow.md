# Wave 3 system schedule window overnight allow

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级跨午夜窗口内允许路径
- 目标：补齐 `window-overnight.json` 的窗口内成功判定，避免根级系统测试只覆盖“跨午夜窗口外禁止”而缺少正向样例

## 本次完成内容

- 在 `tests/system/config_schedule_errors.rs` 中新增跨午夜窗口内允许测试
- 复用 `fixtures/system/schedule/window-overnight.json`
- 固定 `local_hour = 23` 时 `collect=true, analyze=false, push=true`
- 同步更新 `tests/README.md`

## 阶段结论

根级 `config + schedule` 系统测试现在同时覆盖白天窗口成功、跨午夜窗口成功、跨午夜窗口失败以及两类非法窗口错误。

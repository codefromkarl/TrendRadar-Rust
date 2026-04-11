# Wave 3 module map window gate sync

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：把动态窗口门控顺序同步回模块映射文档
- 目标：让 `docs/module-map.md` 明确记录 `app` 在 `schedule.window` 阻断场景下也遵守“先门控、后解析”的边界

## 本次完成内容

- 更新 `docs/module-map.md` 中的 `app` 边界证据
- 明确这条顺序约束同时覆盖显式 `collect=false` 和动态 `schedule.window` 阻断
- 保持 `module-map` 与当前根级系统测试证据一致

## 阶段结论

`module-map` 现在已经能完整表达 `app` 的执行顺序边界，而不只是静态阶段开关的行为。这样后续继续扩时间规则时，更不容易把 source 解析提前到本应被阻断的路径。

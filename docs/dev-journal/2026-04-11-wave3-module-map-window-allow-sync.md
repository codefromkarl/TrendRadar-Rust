# Wave 3 module map window allow sync

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：把窗口放行错误传播同步回模块映射文档
- 目标：让 `docs/module-map.md` 明确记录 `schedule.window` 在允许执行和阻断执行两侧的完整行为对照

## 本次完成内容

- 在 `docs/module-map.md` 中新增动态窗口放行 / 阻断的对照边界说明
- 明确窗口放行时上游解析错误会直接上浮，窗口阻断时则不会触碰 source
- 保持 `module-map` 与当前 51 条根级 system richer cases 对齐

## 阶段结论

`module-map` 现在已经能完整表达 `app` 在动态时间门控下的双侧行为，不再只描述阻断路径。这样后续扩时间规则或整理编排入口时，更容易守住“由上游决策决定是否解析 source”的原则。

# Wave 3 module map collect error sync

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：把 `collect=true` 的错误传播边界同步回模块映射文档
- 目标：让 `docs/module-map.md` 明确记录 source 解析是否发生只由抓取阶段门控决定，而不受后续阶段状态影响

## 本次完成内容

- 在 `docs/module-map.md` 中新增 `collect=true` 时立即暴露损坏 source 的边界说明
- 明确即使 `analyze` / `push` 关闭，只要抓取阶段开启，source 解析错误仍然会直接上浮
- 保持 `module-map` 与当前 50 条根级 system richer cases 对齐

## 阶段结论

`module-map` 现在已经同时表达了 source 解析的“跳过条件”和“触发条件”。这样后续继续扩阶段组合或重构 `app` 时，更不容易把解析时机和下游阶段状态混淆。

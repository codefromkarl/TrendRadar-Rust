# Wave 3 module map overnight window symmetry sync

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：把跨午夜窗口的对称编排证据同步回模块映射文档
- 目标：让 `docs/module-map.md` 明确记录 overnight window 已经在 `app` 层形成“窗口内允许 / 窗口外禁止”的完整系统证据

## 本次完成内容

- 更新 `docs/module-map.md` 中的 overnight window 边界说明
- 明确 `started_at + timezone` 在跨午夜窗口下已经具备 allow/block 双侧编排证据
- 保持 `module-map` 与当前 53 条根级 system richer cases 对齐

## 阶段结论

`module-map` 现在不只说明 `app` 支持时间窗口，还明确记录了跨午夜窗口在完整编排层的对称行为。这样后续扩时间规则时，更不容易把 overnight 处理退回成只剩 crate 级判断的状态。

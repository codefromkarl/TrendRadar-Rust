# Wave 3 module map overnight window sync

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：把跨午夜窗口的编排证据同步回模块映射文档
- 目标：让 `docs/module-map.md` 明确记录 overnight window 已经被 `app` 全链路系统测试证明，而不只是 `schedule` crate 的局部行为

## 本次完成内容

- 在 `docs/module-map.md` 中新增跨午夜窗口编排证据
- 明确 `started_at + timezone` 的跨午夜放行已经在根级系统测试中完成验证
- 保持 `module-map` 与当前 52 条根级 system richer cases 对齐

## 阶段结论

`module-map` 现在已经能表达 `app` 对时间窗口的处理不仅有规则边界，也有真实编排证据。这样后续扩时间规则或整理运行入口时，更不容易把 overnight 行为只当成 `schedule` crate 的局部细节。

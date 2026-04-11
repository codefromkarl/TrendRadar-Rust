# Wave 3 module map source shape sync

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：把 `app` 的来源形态覆盖同步回模块映射文档
- 目标：让 `docs/module-map.md` 直接反映当前根级系统测试已经证明的 source shape 边界，而不是只保留模糊的“单来源 / 双来源”概念

## 本次完成内容

- 更新 `docs/module-map.md` 中的 `app` 边界证据
- 明确根级系统测试已覆盖空来源、单来源、双来源，以及 RSS-only / hotlist-only 形态
- 保持 `module-map` 与当前 47 条根级 system richer cases 同步

## 阶段结论

`module-map` 现在可以更准确地描述 `app` 只依赖显式 source 列表，而不是对具体来源组合做假设。这样后续继续扩 fixture 或迁移运行入口时，更不容易把来源组合规则重新塞回 `app`。

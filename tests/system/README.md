# System Tests

这个目录用于放置跨 crate 或完整链路的系统性测试。

## 目标

- 用稳定输入覆盖跨模块行为
- 让迁移阶段的回归风险尽量前置暴露
- 为后续 parity 测试、snapshot 和覆盖率统计提供固定挂载点

## 推荐结构

- 一个测试文件对应一条链路或一类能力
- 对应 fixture 放在 `fixtures/system/` 下
- 如果输出适合做快照，统一放入 `tests/snapshots/`

## 命名建议

- `config_to_bootstrap.rs`
- `fetch_to_domain.rs`
- `analyze_pipeline.rs`
- `storage_roundtrip.rs`

## 新增测试时至少要回答的问题

- 这个测试保护的是哪条链路
- 输入样例从哪里来
- 期望输出如何比较
- 失败时如何快速定位是配置、抓取、分析还是输出层的问题

## 参考模板

- [系统性测试模板](../../docs/system-test-template.md)

# Tests

工作区级测试说明：

- crate 内部的单元测试放在各自 `src/` 中
- crate 级集成测试放在对应 crate 的 `tests/` 目录
- 根目录 `tests/` 主要用于跨 crate 集成场景

## 当前约束

- `tests/system/` 预留给系统性测试
- 根目录测试优先覆盖跨 crate 行为，而不是重复 crate 内部单元测试
- 每个系统性测试应尽量对应稳定 fixture 或明确的内联样例
- 根目录通过 `tests/system.rs` 挂载系统性测试模块
- 共享 fixture loader 与通用 helper 统一放在 `tests/common/mod.rs`

## 推荐新增顺序

1. `config -> app::bootstrap`
2. `domain + analyze`
3. `fetch -> domain`
4. `storage -> report`

## 当前已落地样例

- `crates/app/tests/config_to_bootstrap.rs`
  使用 `fixtures/system/config/` 下的最小配置样例，验证 `config -> app::bootstrap` 基础链路
- `tests/system/fetch_to_domain.rs`
  使用 `fixtures/system/fetch/` 和最小配置样例，验证 `config -> fetch -> domain`
- `tests/system/analyze_pipeline.rs`
  使用 `fixtures/system/analyze/` 和配置样例，验证 `config -> schedule -> analyze`
- `tests/system/storage_to_report.rs`
  使用 `fixtures/system/storage/`，验证 `storage -> report` 并接入 JSON snapshot

## 默认新增流程

1. 先补 fixture
2. 再补 crate 内测试或系统性测试
3. 确认测试先失败
4. 再进入实现

不要把测试留到功能完成后再集中补。

## 参考模板

- [system-test-template.md](../docs/system-test-template.md)
- [tests/system/README.md](./system/README.md)

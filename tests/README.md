# Tests

工作区级测试说明：

- crate 内部的单元测试放在各自 `src/` 中
- crate 级集成测试放在对应 crate 的 `tests/` 目录
- 根目录 `tests/` 主要用于跨 crate 集成场景

## 当前约束

- `tests/system/` 预留给系统性测试
- 根目录测试优先覆盖跨 crate 行为，而不是重复 crate 内部单元测试
- 每个系统性测试应尽量对应稳定 fixture 或明确的内联样例

## 推荐新增顺序

1. `config -> app::bootstrap`
2. `domain + analyze`
3. `fetch -> domain`
4. `storage -> report`

## 当前已落地样例

- `crates/app/tests/config_to_bootstrap.rs`
  使用 `fixtures/system/config/` 下的最小配置样例，验证 `config -> app::bootstrap` 基础链路

## 参考模板

- [system-test-template.md](../docs/system-test-template.md)
- [tests/system/README.md](./system/README.md)

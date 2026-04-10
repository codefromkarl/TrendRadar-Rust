# 实现验收矩阵

## 目标

这份文档用于把模块契约、实施文档、fixture、测试、验证命令和当前状态收口到一张表里，方便进入实现阶段后做并行跟踪。

## 使用规则

- 每个模块开始实现前，先补全契约文档和实施文档
- 每个模块完成后，更新状态、测试入口和阻塞项
- 如果某模块被拆成多个并行子任务，可在“备注”列继续展开

## 矩阵

| 模块 | 契约文档 | 实施文档 | fixture / 测试入口 | 最低验证命令 | 当前状态 | 阻塞项 / 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| `domain` | [contracts/domain.md](./contracts/domain.md) | [implementation/domain.md](./implementation/domain.md) | `cargo test -p trendradar-domain`、后续补序列化测试 | `cargo test -p trendradar-domain` | 契约已落地 | Wave 0 后续补序列化测试样例 |
| `config` | [contracts/config.md](./contracts/config.md) | [implementation/config.md](./implementation/config.md) | `fixtures/system/config/`、`crates/app/tests/config_to_bootstrap.rs` | `cargo test -p trendradar-config` | 契约已落地 | 调度 / 输出字段仍属后续扩展 |
| `schedule` | [contracts/schedule.md](./contracts/schedule.md) | [implementation/schedule.md](./implementation/schedule.md) | 下一阶段补 `schedule` fixture 与 crate 级测试 | `cargo test -p trendradar-schedule` | 未开始实现 | 前置是 `config` 中调度字段进入契约 |
| `analyze` | [contracts/analyze.md](./contracts/analyze.md) | [implementation/analyze.md](./implementation/analyze.md) | 下一阶段补分析 fixture、结构断言或快照 | `cargo test -p trendradar-analyze` | 未开始实现 | 前置是统一输入模型与分析样例固定 |
| `fetch` | [contracts/fetch.md](./contracts/fetch.md) | [implementation/fetch.md](./implementation/fetch.md) | 下一阶段补 RSS / 热榜来源样例 | `cargo test -p trendradar-fetch` | 未开始实现 | 前置是来源范围与错误分类冻结 |
| `storage` | [contracts/storage.md](./contracts/storage.md) | [implementation/storage.md](./implementation/storage.md) | 下一阶段补读写 roundtrip fixture | `cargo test -p trendradar-storage` | 未开始实现 | 前置是主键与去重策略固定 |
| `report` | [contracts/report.md](./contracts/report.md) | [implementation/report.md](./implementation/report.md) | 下一阶段补 JSON 结构断言或快照 | `cargo test -p trendradar-report` | 未开始实现 | 前置是输出顶层结构固定 |
| `app` | 暂无独立契约，依赖上游模块 | [implementation/app.md](./implementation/app.md) | `crates/app/tests/`、`tests/system/` | `cargo test -p trendradar-app` | 已有最小 bootstrap 链路 | 前置是 Wave 1 模块形成最小可编排接口 |

## 阶段门槛

进入 Wave 1 之前，至少应满足：

- `domain` 和 `config` 的契约文档已从骨架补成可执行版本
- 至少一组 fixture / 测试入口已写入矩阵
- 负责模块的人可以只靠矩阵和模块文档开始实现

## 当前 Wave 0 证据

- 真实 fixture：
  - `fixtures/system/config/minimal-valid.json`
  - `fixtures/system/config/invalid-empty-timezone.json`
- 真实测试：
  - `crates/app/tests/config_to_bootstrap.rs`
- 当前验证命令：
  - `cargo fmt --all --check`
  - `cargo test --workspace`

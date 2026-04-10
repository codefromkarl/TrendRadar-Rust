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
| `schedule` | [contracts/schedule.md](./contracts/schedule.md) | [implementation/schedule.md](./implementation/schedule.md) | 待补 | `cargo test -p trendradar-schedule` | 骨架已建 | 等待配置契约 |
| `analyze` | [contracts/analyze.md](./contracts/analyze.md) | [implementation/analyze.md](./implementation/analyze.md) | 待补 | `cargo test -p trendradar-analyze` | 骨架已建 | 等待分析 fixture |
| `fetch` | [contracts/fetch.md](./contracts/fetch.md) | [implementation/fetch.md](./implementation/fetch.md) | 待补 | `cargo test -p trendradar-fetch` | 骨架已建 | 等待来源范围冻结 |
| `storage` | [contracts/storage.md](./contracts/storage.md) | [implementation/storage.md](./implementation/storage.md) | 待补 | `cargo test -p trendradar-storage` | 骨架已建 | 等待主键 / 去重策略 |
| `report` | [contracts/report.md](./contracts/report.md) | [implementation/report.md](./implementation/report.md) | 待补 | `cargo test -p trendradar-report` | 骨架已建 | 等待 JSON 顶层结构 |
| `app` | 暂无独立契约，依赖上游模块 | [implementation/app.md](./implementation/app.md) | `crates/app/tests/`、`tests/system/` | `cargo test -p trendradar-app` | 骨架已建 | 等待 Wave 1 模块最小闭环 |

## 阶段门槛

进入 Wave 1 之前，至少应满足：

- `domain` 和 `config` 的契约文档已从骨架补成可执行版本
- 至少一组 fixture / 测试入口已写入矩阵
- 负责模块的人可以只靠矩阵和模块文档开始实现

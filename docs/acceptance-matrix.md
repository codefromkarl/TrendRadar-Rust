# 实现验收矩阵

## 目标

这份文档用于把模块契约、实施文档、fixture、测试、验证命令和当前状态收口到一张表里，方便进入实现阶段后做并行跟踪。

## 使用规则

- 每个模块开始实现前，先补全契约文档和实施文档
- 每个模块开始实现前，必须先填 `fixture / 测试入口` 与 `最低验证命令`
- 每个模块开始实现前，至少要确定一个 crate 内测试或系统性测试入口
- 每个模块完成后，更新状态、测试入口和阻塞项
- 如果某模块被拆成多个并行子任务，可在“备注”列继续展开
- 没有 fixture、测试入口或验证命令的模块，不应进入“已完成”状态
- 如果输出适合快照，需在“fixture / 测试入口”或“阻塞项 / 备注”中写明 snapshot 挂载点

## 推荐工作流

按下面顺序使用这张矩阵：

1. 先写契约文档和实施文档
2. 在矩阵中登记 fixture、测试入口、验证命令
3. 先补 fixture 和测试，再进入实现
4. 实现完成后，再回填状态和剩余阻塞项

这张矩阵不是收尾清单，而是实现前的测试驱动入口。

## 矩阵

| 模块 | 契约文档 | 实施文档 | fixture / 测试入口 | 最低验证命令 | 当前状态 | 阻塞项 / 备注 |
| --- | --- | --- | --- | --- | --- | --- |
| `domain` | [contracts/domain.md](./contracts/domain.md) | [implementation/domain.md](./implementation/domain.md) | `cargo test -p trendradar-domain`、后续补序列化测试 | `cargo test -p trendradar-domain` | 契约已落地 | Wave 0 后续补序列化测试样例 |
| `config` | [contracts/config.md](./contracts/config.md) | [implementation/config.md](./implementation/config.md) | `fixtures/system/config/`、`crates/app/tests/config_to_bootstrap.rs` | `cargo test -p trendradar-config` | 契约已落地，含 `schedule` 字段 | 输出字段仍属后续扩展 |
| `schedule` | [contracts/schedule.md](./contracts/schedule.md) | [implementation/schedule.md](./implementation/schedule.md) | 复用 `fixtures/system/config/`、`cargo test -p trendradar-schedule` | `cargo test -p trendradar-schedule` | 已完成配置到决策映射与 fixture 驱动测试 | 后续如进入时间窗口表达，再补独立 `schedule` fixture |
| `analyze` | [contracts/analyze.md](./contracts/analyze.md) | [implementation/analyze.md](./implementation/analyze.md) | `fixtures/system/analyze/news-ranking-input.json`、`fixtures/system/analyze/source-groups-input.json`、`cargo test -p trendradar-analyze` | `cargo test -p trendradar-analyze` | 已完成基础评分、排序与来源聚合测试 | 更高阶过滤与综合排序留待后续阶段 |
| `fetch` | [contracts/fetch.md](./contracts/fetch.md) | [implementation/fetch.md](./implementation/fetch.md) | `fixtures/system/fetch/rss-rust-blog.json`、`fixtures/system/fetch/hotlist-weibo.json`、`cargo test -p trendradar-fetch` | `cargo test -p trendradar-fetch` | 已完成一个 RSS 源和一个热榜源的 fixture 打通 | 真实网络抓取与更细错误分类留待后续阶段 |
| `storage` | [contracts/storage.md](./contracts/storage.md) | [implementation/storage.md](./implementation/storage.md) | `fixtures/system/storage/news-roundtrip-input.json`、`cargo test -p trendradar-storage` | `cargo test -p trendradar-storage` | 已完成 SQLite 最小实现与去重测试 | 文件数据库路径与迁移框架留待后续阶段 |
| `report` | [contracts/report.md](./contracts/report.md) | [implementation/report.md](./implementation/report.md) | `fixtures/system/report/news-report-input.json`、`cargo test -p trendradar-report` | `cargo test -p trendradar-report` | 已完成带运行元数据的 JSON 顶层结构测试 | HTML 报告与错误渲染留待后续阶段 |
| `app` | 暂无独立契约，依赖上游模块 | [implementation/app.md](./implementation/app.md) | `crates/app/tests/`、`tests/system/` | `cargo test -p trendradar-app` | 已有最小 bootstrap 链路 | 前置是 Wave 1 模块形成最小可编排接口 |

## 阶段门槛

进入 Wave 1 之前，至少应满足：

- `domain` 和 `config` 的契约文档已从骨架补成可执行版本
- 至少一组 fixture / 测试入口已写入矩阵
- 负责模块的人可以只靠矩阵和模块文档开始实现

## 当前 Wave 0 证据

真实 fixture：

- `fixtures/system/config/minimal-valid.json`
- `fixtures/system/config/invalid-empty-timezone.json`

真实测试：

- `crates/app/tests/config_to_bootstrap.rs`

当前验证命令：

- `cargo fmt --all --check`
- `cargo test --workspace`

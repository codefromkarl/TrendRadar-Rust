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
| `domain` | [contracts/domain.md](./contracts/domain.md) | [implementation/domain.md](./implementation/domain.md) | `fixtures/system/domain/`、`cargo test -p trendradar-domain` | `cargo test -p trendradar-domain` | 契约已落地，含序列化 fixture roundtrip 测试 | 后续如进入统一内容模型，再补跨模型迁移样例 |
| `config` | [contracts/config.md](./contracts/config.md) | [implementation/config.md](./implementation/config.md) | `fixtures/system/config/`、`crates/app/tests/config_to_bootstrap.rs` | `cargo test -p trendradar-config` | 契约已落地，含 `schedule` 字段 | 输出字段仍属后续扩展 |
| `schedule` | [contracts/schedule.md](./contracts/schedule.md) | [implementation/schedule.md](./implementation/schedule.md) | `fixtures/system/config/`、`fixtures/system/schedule/`、`cargo test -p trendradar-schedule`、`cargo test -p trendradar-config` | `cargo test -p trendradar-schedule` | 已完成配置到决策映射、最小时间窗口表达与 fixture 驱动测试 | 更细粒度规则如工作日 / 冷却周期留待后续阶段 |
| `analyze` | [contracts/analyze.md](./contracts/analyze.md) | [implementation/analyze.md](./implementation/analyze.md) | `fixtures/system/analyze/news-ranking-input.json`、`fixtures/system/analyze/source-groups-input.json`、`fixtures/system/analyze/zero-rank-input.json`、`fixtures/system/analyze/same-rank-input.json`、`cargo test -p trendradar-analyze` | `cargo test -p trendradar-analyze` | 已完成基础评分、排序、来源聚合、零排名边界与同排名 tie-break 测试 | 更高阶过滤与综合排序留待后续阶段 |
| `fetch` | [contracts/fetch.md](./contracts/fetch.md) | [implementation/fetch.md](./implementation/fetch.md) | `fixtures/system/fetch/rss-rust-blog.json`、`fixtures/system/fetch/hotlist-weibo.json`、`fixtures/system/fetch/invalid-rss.json`、`fixtures/system/fetch/empty-rss.json`、`cargo test -p trendradar-fetch`（含 mockito 隔离的 HTTP adapter 测试） | `cargo test -p trendradar-fetch` | 已完成 fixture adapter 与 HTTP adapter，含 `Network`/`Http`/`ParseResponse` 错误分类，14 条测试覆盖 | HTTP 超时配置、限流、不同平台热榜差异化格式留待后续阶段 |
| `storage` | [contracts/storage.md](./contracts/storage.md) | [implementation/storage.md](./implementation/storage.md) | `fixtures/system/storage/news-roundtrip-input.json`、空仓库读取断言、`cargo test -p trendradar-storage` | `cargo test -p trendradar-storage` | 已完成 SQLite 最小实现、去重与空仓库边界测试 | 文件数据库路径与迁移框架留待后续阶段 |
| `report` | [contracts/report.md](./contracts/report.md) | [implementation/report.md](./implementation/report.md) | `fixtures/system/report/news-report-input.json`、空输入 JSON 断言、`cargo test -p trendradar-report` | `cargo test -p trendradar-report` | 已完成带运行元数据的 JSON 顶层结构与空输入边界测试 | HTML 报告与错误渲染留待后续阶段 |
| `app` | 暂无独立契约，依赖上游模块 | [implementation/app.md](./implementation/app.md) | `crates/app/tests/config_to_bootstrap.rs`、`crates/app/tests/wave2_pipeline.rs`、`crates/app/tests/wave3_schedule_gate.rs`、`tests/system/` | `cargo test -p trendradar-app` | 已具备最小 fixture pipeline 与系统测试入口 | `W2-parity-review` 已完成；Wave 3 已补系统级阶段开关、空链路与空报告证明，`app` 仍为薄编排 |

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

## 当前 Wave 2 证据

真实 fixture：

- `fixtures/system/config/minimal-valid.json`
- `fixtures/system/fetch/hotlist-weibo.json`
- `fixtures/system/fetch/rss-rust-blog.json`

真实测试：

- `crates/app/tests/config_to_bootstrap.rs`
- `crates/app/tests/wave2_pipeline.rs`
- `crates/app/tests/wave3_schedule_gate.rs`

当前结论：

- 已存在从 `config` 到结构化输出的最小 fixture pipeline
- `app` 仍只负责编排和系统测试挂载，业务规则未被吸入
- Wave 3 可以在当前基线之上继续推进

## 当前 Wave 3 证据

真实 fixture / 样例：

- `fixtures/system/domain/`
- `fixtures/system/schedule/`
- `fixtures/system/analyze/zero-rank-input.json`
- `fixtures/system/analyze/same-rank-input.json`
- `fixtures/system/fetch/invalid-rss.json`
- `fixtures/system/fetch/invalid-hotlist.json`
- `fixtures/system/fetch/empty-rss.json`
- `fixtures/system/fetch/empty-hotlist.json`
- `fixtures/system/config/collect-only.json`
- `fixtures/system/config/disabled-all.json`
- `fixtures/system/config/report-only-empty.json`
- `fixtures/system/config/analyze-without-report.json`
- `fixtures/system/config/collect-and-report-no-analyze.json`
- `fixtures/system/config/analyze-disabled.json`
- `fixtures/system/config/analyze-only-empty.json`
- `fixtures/system/config/push-only-empty.json`
- `fixtures/system/config/minimal-valid-rss-only.json`

当前覆盖：

- `domain`：共享模型 JSON roundtrip
- `schedule`：白天窗口、跨午夜窗口、相等小时非法、越界小时非法
- `analyze`：零排名边界、同排名 tie-break、`analyze=false` 门控，以及真实抓取输出上的同 rank 排序稳定性、来源聚合 best-rank / item-count 优先级，以及部分抓取成功后的整体中断语义
- `fetch`：RSS / 热榜的正常、空输入、非法 fixture 路径，以及部分抓取成功后的整体中断语义；HTTP adapter 已补 `HttpRssFetcher` 和 `HttpHotlistFetcher`，含 mockito 隔离的正常解析、空 channel/数组、HTTP 错误、XML/JSON 解析错误、网络不可达共 10 条测试
- `storage`：空仓库初始读取、去重后进入报告、相同 rank 重复写入仍去重、同标题不同来源在相同 rank 下仍保留分离、乱序写入后稳定排序、同 rank 时按 `source_id + title` 稳定排序
- `report`：空输入 JSON 结构
- `app`：根级系统层已覆盖最小正向全链路、空来源全链路、单来源全链路、RSS-only 全链路、hotlist-only 全链路、跨午夜窗口内放行 / 窗口外阻断全链路，以及 `collect=false` 时跳过损坏 source、窗口阻断时跳过损坏 source、`collect-only` 时仍传播损坏 source 错误、窗口放行时仍传播损坏 source 错误的路径和 8 个阶段布尔组合、窗口内放行 / 窗口外阻断路径
- 根级 `tests/system/`：当前共有 62 条系统测试，已覆盖 `config_schedule_errors`、`fetch_to_domain`、`fetch_to_analyze`、`analyze_pipeline`、`storage_to_report`、`app_pipeline_modes`
- 根级 `tests/system/app_pipeline_modes.rs`：已覆盖最小正向全链路、空来源全链路、单来源全链路、RSS-only 全链路、hotlist-only 全链路、跨午夜窗口内放行 / 窗口外阻断全链路、`collect=false` 时跳过损坏 source、窗口阻断时跳过损坏 source、`collect-only` 时仍传播损坏 source 错误、窗口放行时仍传播损坏 source 错误的路径、8 个 `collect/analyze/push` 布尔组合和窗口内放行 / 窗口外阻断

当前结论：

- Wave 3 已形成一批 crate 级与根级系统级边界样例
- 当前下一步更适合把 richer cases 继续推进到非 `app` 链路，而不是回退到重新划分 `app` 边界

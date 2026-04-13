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
| `config` | [contracts/config.md](./contracts/config.md) | [implementation/config.md](./implementation/config.md) | `fixtures/system/config/`、`crates/app/tests/config_to_bootstrap.rs` | `cargo test -p trendradar-config` | 契约已落地，含 `schedule` 字段和 `rss_feeds`/`hotlist_apis` 源 URL 配置；Wave 5 新增 `http_timeout_secs`（默认 30s）和 `keywords`（`Vec<String>`）字段；Wave 6 新增 `notification`（`NotificationConfig`）字段 | 输出字段仍属后续扩展 |
| `schedule` | [contracts/schedule.md](./contracts/schedule.md) | [implementation/schedule.md](./implementation/schedule.md) | `fixtures/system/config/`、`fixtures/system/schedule/`、`cargo test -p trendradar-schedule`、`cargo test -p trendradar-config` | `cargo test -p trendradar-schedule` | 已完成配置到决策映射、最小时间窗口表达与 fixture 驱动测试 | 更细粒度规则如工作日 / 冷却周期留待后续阶段 |
| `analyze` | [contracts/analyze.md](./contracts/analyze.md) | [implementation/analyze.md](./implementation/analyze.md) | `fixtures/system/analyze/news-ranking-input.json`、`fixtures/system/analyze/source-groups-input.json`、`fixtures/system/analyze/zero-rank-input.json`、`fixtures/system/analyze/same-rank-input.json`、`cargo test -p trendradar-analyze` | `cargo test -p trendradar-analyze` | 已完成基础评分、排序、来源聚合、零排名边界与同排名 tie-break 测试；Wave 5 补齐 `filter_by_keywords()` 不区分大小写关键词过滤 | 更高阶过滤与综合排序留待后续阶段 |
| `fetch` | [contracts/fetch.md](./contracts/fetch.md) | [implementation/fetch.md](./implementation/fetch.md) | `fixtures/system/fetch/rss-rust-blog.json`、`fixtures/system/fetch/hotlist-weibo.json`、`fixtures/system/fetch/invalid-rss.json`、`fixtures/system/fetch/empty-rss.json`、`cargo test -p trendradar-fetch`（含 mockito 隔离的 HTTP adapter 测试） | `cargo test -p trendradar-fetch` | 已完成 fixture adapter 与 HTTP adapter，含 `Network`/`Http`/`ParseResponse` 错误分类；Wave 5 补齐 `with_timeout()` 可配置超时构造器；Wave 7 新增 `HotlistParser` trait + 8 实现（Generic/Weibo/Zhihu/Bilibili/Toutiao/Baidu/Pengpai/Cls）+ 工厂函数 `hotlist_parser_for()` + `HttpHotlistFetcher` parser 注入 | 新平台解析器可通过实现 trait 扩展 |
| `storage` | [contracts/storage.md](./contracts/storage.md) | [implementation/storage.md](./implementation/storage.md) | `fixtures/system/storage/news-roundtrip-input.json`、空仓库读取断言、`cargo test -p trendradar-storage` | `cargo test -p trendradar-storage` | 已完成 SQLite 最小实现、去重与空仓库边界测试；Wave 5 补齐文件持久化 `SqliteNewsRepository::open(path)` 及父目录自动创建 | 远程存储与 schema 迁移留待后续阶段 |
| `report` | [contracts/report.md](./contracts/report.md) | [implementation/report.md](./implementation/report.md) | `fixtures/system/report/news-report-input.json`、空输入 JSON 断言、`cargo test -p trendradar-report` | `cargo test -p trendradar-report` | 已完成 JSON 输出与 HTML 报告；Wave 6 补齐 `render_news_html()` 自包含 HTML5 输出；Wave 7 新增 `render_news_table()` 终端彩色表格（comfy-table）和 `render_news_markdown()` GFM 表格输出 | 更多输出格式留待后续阶段 |
| `notification` | 暂无独立契约 | 暂无独立实施文档 | `crates/notification/src/lib.rs`（tests module）、`cargo test -p trendradar-notification` | `cargo test -p trendradar-notification` | 已完成 `Notifier` trait、`WebhookNotifier`（HTTP POST JSON）、`ConsoleNotifier`（stdout）、`build_notifiers()` 工厂函数；7 条测试覆盖 | 更多通知渠道（飞书/钉钉/Telegram 等）留待后续阶段 |
| `app` | 暂无独立契约，依赖上游模块 | [implementation/app.md](./implementation/app.md) | `crates/app/tests/config_to_bootstrap.rs`、`crates/app/tests/wave2_pipeline.rs`、`crates/app/tests/wave3_schedule_gate.rs`、`crates/app/tests/wave4_http_pipeline.rs`、`crates/app/tests/binary_smoke.rs`、`tests/system/` | `cargo test -p trendradar-app` | 已具备 fixture pipeline、HTTP config pipeline 和 CLI binary 入口；Wave 5 补齐 clap CLI、tracing 日志、配置自动发现、文件 SQLite、关键词过滤、resilient 双模式；Wave 6 补齐 `--output json/html/both`、HTML 报告、通知集成；Wave 7 补齐 `--output table/markdown` CLI 路由、多平台热榜 parser 注入、CI/CD release pipeline | `app` 仍为薄编排 |

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

## 当前 Wave 5 证据

Wave 5 完成生产就绪化，补齐 7 项缺口：

新增依赖：
- `tracing` + `tracing-subscriber`（日志框架）
- `clap`（CLI 参数解析）

新增 API：
- `SqliteNewsRepository::open(path)` — 文件 SQLite 持久化，自动创建父目录
- `HttpRssFetcher::with_timeout()` / `HttpHotlistFetcher::with_timeout()` — 可配置 HTTP 超时
- `filter_by_keywords(items, keywords)` — 不区分大小写关键词过滤
- `discover_config_path()` — 三级配置文件搜索（`./config.json` → `~/.config/trendradar/config.json` → `/etc/trendradar/config.json`）
- `resolve_config_path()` / `default_db_path()` — CLI 集成辅助

新增 AppConfig 字段：
- `http_timeout_secs: u64`（默认 30）
- `keywords: Vec<String>`

Pipeline 行为变更：
- `run_pipeline_with_fetchers` 新增 `resilient: bool` 参数
- `run_fixture_pipeline` 使用 `resilient=false`（错误传播）
- `run_config_pipeline` 使用 `resilient=true`（warn+skip 容错）

新增测试：
- `crates/storage` — 文件持久化 + 父目录创建（2 tests）
- `crates/fetch` — 自定义超时构造器（1 test）
- `crates/analyze` — 关键词过滤（4 tests）
- `crates/app` — binary smoke（3 tests：`--help`、`--version`、`--dry-run`）

当前全量验证：127 tests passed (26 suites), 0 clippy issues

## 当前 Wave 6 证据

Wave 6 完成首版闭合，补齐 migration-strategy.md §7 产品边界最后缺口：

新增 crate：
- `crates/notification` — 通知适配器（`Notifier` trait + `WebhookNotifier` + `ConsoleNotifier` + `build_notifiers`）

新增 API：
- `render_news_html(items, context)` — 自包含 HTML5 报告渲染（内联 CSS、XSS 转义、响应式表格、空状态）
- `build_notifiers(enabled, webhook_url)` — 从配置参数构建通知器列表
- `WebhookNotifier::new(url)` — HTTP POST JSON payload 通知
- `ConsoleNotifier` — stdout 输出通知（调试/默认回退）

新增 AppConfig 字段：
- `notification: NotificationConfig`（`{ enabled: bool, webhook_url: Option<String> }`）

新增 CLI 参数：
- `--output <format>` — 输出格式选择（`json` | `html` | `both`，默认 `json`）

新增测试：
- `crates/report` — HTML 报告（5 tests：HTML5 结构、数据嵌入、XSS 转义、元数据、空状态）
- `crates/notification` — 通知渠道（7 tests：console 发送、webhook mock、HTTP 错误、build_notifiers 各分支）

## 当前 Wave 8 证据

Wave 8 完成通知渠道扩展，补齐 v1.2 第一项功能缺口：

新增 API：
- `FeishuNotifier::new(url)` — 飞书机器人文本通知
- `DingTalkNotifier::new(url)` — 钉钉机器人文本通知
- `WeComNotifier::new(url)` — 企业微信机器人文本通知
- `build_notifiers(enabled, webhook_url, feishu_webhook_url, dingtalk_webhook_url, wecom_webhook_url)` — 多渠道通知工厂

新增 AppConfig 字段：
- `notification.feishu_webhook_url: Option<String>`
- `notification.dingtalk_webhook_url: Option<String>`
- `notification.wecom_webhook_url: Option<String>`

新增测试：
- `crates/notification` — 3 条新增 mock payload 测试 + 1 条多渠道工厂测试
- `crates/config` — 通知字段加载与默认值测试

当前全量验证：全 workspace tests passed，0 clippy issues（除仓库既有 MSRV 提示）

新增文档：
- `docs/migration-guide.md` — Python → Rust 配置迁移指南

Release 验证：
- `cargo build --release` → 9.6MB binary
- `--help` 输出完整（7 个选项）
- 全部 migration-strategy.md §7 首版产品边界已闭合

当前全量验证：161 tests passed, 0 clippy issues

## 当前 Wave 7 证据

Wave 7 完成 v1.1.0 首版增强，补齐 4 项高价值功能：

新增 API：
- `HotlistParser` trait + 8 实现（`GenericHotlistParser` / `WeiboHotlistParser` / `ZhihuHotlistParser` / `BilibiliHotlistParser` / `ToutiaoHotlistParser` / `BaiduHotlistParser` / `PengpaiHotlistParser` / `ClsHotlistParser`）
- `hotlist_parser_for(source_type)` — 工厂函数，根据配置选择解析器
- `HttpHotlistFetcher::with_parser()` — 支持注入自定义 parser
- `render_news_table(items, context)` — 终端彩色表格输出（comfy-table 7.1）
- `render_news_markdown(items, context)` — GFM Markdown 表格输出

新增 AppConfig 字段：
- `hotlist_apis[].source_type: Option<String>` — 数据源类型（`"generic"` / `"weibo"` / `"zhihu"` / `"bilibili"` / `"toutiao"` / `"baidu"` / `"pengpai"` / `"cls"`，默认 `"generic"`）

新增 CLI 参数：
- `--output table` — 终端彩色表格（排名前3红色高亮）
- `--output markdown` — GFM Markdown 表格

新增 CI/CD：
- `.github/workflows/ci.yml` — 补齐 `cargo build --release --locked` 步骤
- `.github/workflows/release.yml` — tag 触发三平台 release（Linux/macOS/Windows）

新增测试：
- `crates/fetch` — 4 个 parser 各 3 条测试 + 工厂函数 1 条 + parser 注入 1 条（共 14 条新增）
- `crates/report` — table 输出 3 条 + markdown 输出 4 条（共 7 条新增）

Pipeline 变更：
- `PipelineResult` 新增 `report_table` 和 `report_markdown` 字段
- `run_pipeline_with_fetchers` 同时生成 4 种格式报告
- pipeline 根据 `api.source_type` 注入对应 parser

依赖变更：
- `comfy-table` 降级到 7.1.1（7.2.2 使用 let chains 特性，需 Rust 1.87+）

当前全量验证：161 tests passed, 0 clippy issues

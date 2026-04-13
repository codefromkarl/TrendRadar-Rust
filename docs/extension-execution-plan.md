# 后续拓展执行文档

## 文档目标

这份文档用于把 TrendRadar Rust 在首版闭合后的后续拓展工作，整理成一份可连续执行、可审查、可验证的实施清单。

它不替代下面这些基线文档，而是建立在它们之上：

- [路线图](./roadmap.md)
- [模块映射基线](./module-map.md)
- [迁移策略](./migration-strategy.md)
- [实现验收矩阵](./acceptance-matrix.md)
- [系统性测试模板](./system-test-template.md)

## 适用范围

本文件覆盖首版内核稳定后的三类工作：

- v1.2 范围内的性能优化与能力补齐
- v1.x 范围内的工程交付增强
- v2.0 之后的生态扩展预研与落地顺序

本文件默认不要求一次性完成所有任务，而是要求后续执行时按“单任务闭环”推进。

## 执行原则

1. 每次只执行一个任务编号，避免把多个功能混成一个大改动。
2. 执行前先确认受影响 crate、测试入口和文档同步点。
3. 执行中优先保持 `app` 为薄编排层，不把业务规则塞回 `app`。
4. 执行完成前至少运行一条相关验证命令，不能只写代码不验证。
5. 若任务改变 crate 边界、CLI 参数、验证命令或配置契约，必须同步更新 README 和对应文档。
6. 若任务仍处于设计分歧阶段，只允许补文档、fixture、测试骨架，不直接扩实现。

## 执行顺序总览

建议按下面顺序推进：

1. A1 性能基线与 benchmark
2. A2 报告按需渲染
3. A3 SQLite 批量写入
4. A4 多源并发抓取
5. B1 通知渠道扩展
6. B2 调度增强
7. B3 热榜平台扩展
8. B4 错误码规范
9. B5 安装与分发入口
10. C1 远程对象存储
11. C2 AI 分析
12. C3 MCP Server

## Phase A：性能与运行时效率

### A1. 性能基线与 Benchmark

- 状态：`done`
- 目标：建立 Rust 版当前性能基线，替代“体感更快”的模糊判断。
- 主要范围：`crates/app`、`crates/fetch`、`crates/analyze`、`crates/storage`、`crates/report`
- 关键输出：
- 新增 benchmark 入口，至少覆盖 fixture pipeline 和 HTTP smoke pipeline
- 记录各阶段耗时：fetch、filter、analyze、store、report
- 在文档中留下可复现的运行命令和基线结果
- 建议步骤：
- 引入 `criterion` 或等价 benchmark 方案
- 先对 fixture pipeline 建稳定 benchmark，避免网络噪声
- 再补单次 HTTP smoke benchmark，并明确其仅作趋势参考
- 在 `docs/roadmap.md` 或本文件追加基线结果
- 验证命令：
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo bench --workspace`
- 完成标准：
- 仓库内存在稳定 benchmark 入口
- 至少有一份基线结果被写入文档
- 后续优化任务都能引用同一基线
- 当前实现：
- benchmark 入口：`cargo bench --package trendradar-app --bench pipeline_bench`
- 基线覆盖：fixture pipeline total、HTTP smoke pipeline total、fetch/analyze/storage/report 四个阶段
- 当前备注：本轮先建立 Rust 内部基线；Python 对比值留待后续单独补充
- 首次基线记录：
- `pipeline_total/fixture_pipeline_minimal`: `194.20 µs ~ 207.51 µs`
- `pipeline_total/http_pipeline_smoke`: `1.8386 ms ~ 1.9952 ms`
- `pipeline_stage/fetch_fixture_sources`: `9.0537 µs ~ 9.4556 µs`
- `pipeline_stage/analyze_filter_rank_group`: `1.1192 µs ~ 1.1954 µs`
- `pipeline_stage/storage_in_memory_roundtrip`: `99.968 µs ~ 108.28 µs`
- `pipeline_stage/report_render_all_formats`: `30.268 µs ~ 31.643 µs`

### A2. 报告按需渲染

- 状态：`done`
- 目标：避免 pipeline 在 push 阶段无条件生成 `json/html/table/markdown` 四份报告。
- 主要范围：`crates/app`、`crates/report`
- 关键输出：
- pipeline 只渲染 CLI 或调用方实际请求的输出格式
- `both` 仍保持 JSON + HTML 双输出语义
- 建议步骤：
- 在 `app` 层引入显式输出目标枚举，而不是用原始字符串在 `main.rs` 分支
- 将输出目标传入 pipeline
- 将 `PipelineResult` 调整为按需携带输出，而不是固定四字段全填
- 补齐 `json/html/both/table/markdown` 行为测试
- 验证命令：
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 完成标准：
- 单一输出模式下只生成对应报告
- 现有 CLI 行为不回退
- 相关系统测试和二进制测试仍通过
- 当前实现：
- `app` 已新增 `OutputMode`，CLI 输出模式通过显式枚举传入 pipeline
- 旧 `run_fixture_pipeline()` / `run_config_pipeline()` 保持兼容，默认走 `OutputMode::All`
- 新增 `crates/app/tests/output_mode.rs`，覆盖 `json/html/both/table/markdown` 五种输出模式

### A3. SQLite 批量写入

- 状态：`done`
- 目标：把逐条插入改为事务批量写入，减少 I/O 和 SQLite 提交开销。
- 主要范围：`crates/storage`、`crates/app`
- 关键输出：
- `NewsRepository` 或 SQLite 实现支持批量写入接口
- 保持去重与 `rank = MIN(existing, incoming)` 语义不变
- 建议步骤：
- 在 `storage` crate 增加批量写入 API
- 使用事务包裹整批写入
- 保留现有单条写入接口，避免不必要破坏
- 增加“大批量输入 + 重复标题 + 多来源”测试
- 验证命令：
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 完成标准：
- 批量写入路径已接入 pipeline
- 去重和排序行为与当前一致
- benchmark 能显示存储阶段耗时下降
- 当前实现：
- `NewsRepository` 已增加默认 `save_news_batch()`，`SqliteNewsRepository` 使用单事务批量写入
- `app` pipeline 已切到批量写入路径，不再逐条执行 SQL
- 优化结果：
- `pipeline_stage/storage_in_memory_roundtrip`: `75.258 µs ~ 84.820 µs`
- 对 A1 初始基线提升约 `15.6% ~ 25.3%`
- `pipeline_total/fixture_pipeline_minimal`: `147.17 µs ~ 166.30 µs`

### A4. 多源并发抓取

- 状态：`todo`
- 目标：缩短多源抓取总耗时，避免串行抓取按最慢总和累积。
- 主要范围：`crates/app`、`crates/fetch`
- 关键输出：
- 多个 fetcher 可并发执行
- `resilient=true/false` 的错误语义保持不变
- 建议步骤：
- 先采用最小侵入方案并发包装现有 blocking fetcher
- 保持 fixture pipeline 和 HTTP pipeline 的公共行为一致
- 为并发模式补错误传播、空输入、部分成功样例
- 若后续决定全链路 async，此任务只先做过渡实现，不提前重写全仓
- 验证命令：
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 完成标准：
- 多源抓取可并发
- 错误与日志语义不回退
- benchmark 能显示 fetch 阶段总耗时下降

## Phase B：功能补齐与工程交付

### B1. 通知渠道扩展

- 状态：`todo`
- 目标：在现有 `Notifier` trait 基础上补齐飞书、钉钉、企业微信通知。
- 主要范围：`crates/notification`、`crates/config`、`crates/app`
- 关键输出：
- 新增各平台 notifier 实现
- 配置结构支持多通知渠道
- 默认 `ConsoleNotifier` 回退语义不变
- 建议步骤：
- 先统一通知配置模型，再分别实现各平台 payload 适配
- 每个平台都用 mock server 做成功与失败测试
- `app` 只负责构建 notifiers 和发送，不负责编码各平台字段
- 同步更新 `docs/module-map.md` 和通知相关实现文档
- 验证命令：
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 完成标准：
- 至少支持飞书、钉钉、企业微信三种通知
- 通知失败仍为旁路警告，不中断主 pipeline
- 至少一份配置样例文档可复用

### B2. 调度增强

- 状态：`todo`
- 目标：支持工作日/周末区分与冷却周期，不破坏现有时间窗口语义。
- 主要范围：`crates/schedule`、`crates/config`、`crates/app`
- 关键输出：
- 调度配置支持 weekday/weekend 和 cooldown 规则
- 时区与跨午夜场景下的决策保持可验证
- 建议步骤：
- 先补契约与 fixture，再补实现
- 新规则全部落在 `schedule` crate，`app` 只消费决策结果
- 补齐错误路径：非法天数、非法冷却值、窗口叠加冲突
- 同步更新 `docs/contracts/schedule.md`
- 验证命令：
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 完成标准：
- 新调度规则通过 crate 级和系统级样例验证
- `app` 未吸收额外调度业务规则

### B3. 热榜平台扩展

- 状态：`todo`
- 目标：在现有 `HotlistParser` 扩展点上继续补齐头条、百度、澎湃、财联社等平台。
- 主要范围：`crates/fetch`、`crates/config`、fixtures、系统测试
- 关键输出：
- 新 parser 实现
- 新 fixture 与配置样例
- 平台选择逻辑继续通过工厂函数路由
- 建议步骤：
- 每次只新增 1 到 2 个平台
- 每个平台都先补 fixture，再补 parser，再接配置测试
- 不修改 `HttpHotlistFetcher` 主体职责，保持开闭原则
- 验证命令：
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 完成标准：
- 新平台可以独立解析并通过系统测试
- 不影响已有 `generic/weibo/zhihu/bilibili` 行为

### B4. 错误码规范

- 状态：`todo`
- 目标：统一 CLI 退出码，便于脚本调用和运维诊断。
- 主要范围：`crates/app`
- 关键输出：
- 明确配置错误、网络错误、存储错误、未知错误的退出码
- README 与文档中写清退出码语义
- 建议步骤：
- 先列出当前错误来源与归类边界
- 在 `main.rs` 做统一映射，不把退出码分散在各 crate
- 为二进制 smoke test 增加失败码断言
- 验证命令：
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 完成标准：
- CLI 退出码稳定可预测
- 文档与测试同步

### B5. 安装与分发入口

- 状态：`todo`
- 目标：补齐 `install.sh`、`cargo install` 说明和后续 Homebrew 入口。
- 主要范围：仓库根目录、README、CI/release 文档
- 关键输出：
- 最小安装脚本
- README 安装章节
- release 产物下载说明
- 建议步骤：
- 先交付 `install.sh` 与 README 使用说明
- 再视需要补 Homebrew formula
- 明确平台差异和校验方式
- 验证命令：
- `./scripts/check_environment.sh`
- `cargo test --workspace`
- 完成标准：
- 新用户按 README 能完成安装和首次运行

## Phase C：生态扩展

### C1. 远程对象存储

- 状态：`todo`
- 目标：在不破坏本地 SQLite 默认路径的前提下，补齐 S3/OSS adapter。
- 前置条件：A3 已完成，`NewsRepository` 边界稳定。
- 主要范围：`crates/storage`、`crates/config`
- 关键输出：
- 远程存储实现
- 配置开关与失败回退语义
- 远程存储测试策略
- 完成标准：
- 本地与远程后端可切换
- 本地路径仍是默认实现

### C2. AI 分析

- 状态：`todo`
- 目标：基于内核输出增加摘要、主题分析、标签抽取等能力。
- 前置条件：A1 已完成，性能基线明确；B1-B3 至少完成一项，避免输入模型频繁变化。
- 主要范围：新增独立 crate 或独立模块，不直接污染核心 pipeline
- 关键输出：
- 明确 AI 输入输出契约
- prompt / provider / timeout / retry 策略
- 与报告层的集成点
- 完成标准：
- AI 能力为可选旁路，不阻塞核心抓取与存储链路

### C3. MCP Server

- 状态：`todo`
- 目标：将稳定的 Rust 内核能力以工具接口形式暴露。
- 前置条件：C2 可选；核心内核 API 稳定。
- 主要范围：独立服务入口、工具协议层、文档
- 关键输出：
- 工具列表与权限边界
- 查询类工具优先，写操作延后
- 与 CLI 路径分离的服务入口
- 完成标准：
- MCP 服务不复用 CLI 输出作为协议层
- 工具契约可单独测试

## 单任务执行模板

后续每次执行某个任务时，建议按下面顺序推进：

1. 选定任务编号，例如 `A2`。
2. 列出本次只改哪些 crate、哪些文件类型、哪些文档。
3. 先补或确认 fixture / contract / 测试样例。
4. 再补实现。
5. 跑最小必要验证命令。
6. 更新本文件中对应任务的状态与结果备注。
7. 如有必要，补一篇 `docs/dev-journal/` 日志记录关键决策。

## 状态记录约定

任务状态统一使用下面四种文本值：

- `todo`：尚未开始
- `in-progress`：已开始但未形成可验证闭环
- `blocked`：存在外部依赖或设计阻塞
- `done`：代码、测试、文档和验证已闭环

## 首批建议执行项

如果后续要从这份文档直接开工，建议第一批按下面顺序执行：

1. A1 性能基线与 Benchmark
2. A2 报告按需渲染
3. A3 SQLite 批量写入
4. B1 通知渠道扩展

原因：

- A1 先提供量化依据，后续优化不会失焦。
- A2 和 A3 都是低风险、直接释放 Rust 运行优势的任务。
- B1 是最容易快速补齐用户侧价值差距的能力。

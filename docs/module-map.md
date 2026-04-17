# 模块映射基线

## 文档目标

这份文档用于记录旧 TrendRadar 系统到 Rust 工作区的模块映射关系。

目标不是做“目录翻译表”，而是把迁移任务切分成 AI 和人工都能稳定执行的单位：

- 旧系统里哪一类能力归到哪个 Rust crate
- 哪些能力首版必须进入 Rust
- 哪些能力暂缓、删除或等待重设计
- 每个 crate 的迁移完成标准是什么

## 使用原则

- 这里记录的是“模块级映射”，不是函数级细节
- 一个旧模块如果会拆到多个 crate，必须写清主归属和协作边界
- 如果某项能力决定不迁移，应明确写成“删除”或“延后”，不要留模糊表述
- 映射表要和 `docs/migration-strategy.md` 保持一致

## 工作区目标模块

当前 Rust 工作区按下面的职责收敛：

- `domain`：统一领域模型、共享错误、运行元数据
- `config`：配置模型、默认值、加载入口、后续校验规则
- `schedule`：时间线与执行决策解析
- `analyze`：过滤、聚合、排序、评分等纯计算逻辑
- `storage`：本地持久化抽象与 SQLite 实现
- `fetch`：热点源、RSS 和后续抓取适配
- `report`：结构化结果输出与后续 HTML 输出
- `app`：编排、CLI、运行入口和阶段性集成测试挂载点，不承载抓取、排序、存储或输出规则，也不要求未激活 source 的上游配置成为隐式前提

## 首版模块映射表

| 旧系统能力 | Rust crate | 迁移优先级 | 迁移方式 | 完成定义 |
| --- | --- | --- | --- | --- |
| 配置读取与默认值 | `config` | 高 | 直接迁移并重设计结构体 | 能加载最小配置、校验关键字段、补齐默认值 |
| 统一新闻 / RSS 数据模型 | `domain` | 高 | 重建核心数据模型 | 结构体、错误和运行元数据稳定，序列化策略明确 |
| 调度规则与执行窗口 | `schedule` | 高 | 优先迁纯逻辑 | 能根据固定样例给出稳定调度结果 |
| 关键词过滤、聚合、排序 | `analyze` | 高 | 优先迁纯逻辑 | 对固定输入样例输出稳定排序与统计结果 |
| 热榜与 RSS 抓取 | `fetch` | 高 | 分源适配 | 先打通最小抓取链路，再扩更多源 |
| 本地存储 | `storage` | 高 | 先抽象，再补 SQLite | 数据写入与读取行为在固定 fixture 下稳定 |
| JSON / 结构化输出 | `report` | 中 | 直接实现最小版本 | 对固定内部模型输出稳定结果 |
| CLI 与运行编排 | `app` | 高 | 薄编排 | 能串起配置、抓取、分析、存储和输出的最小闭环，且业务规则仍留在上游 crate |
| HTML 报告 | `report` | 中 | Wave 6 已实现精简 HTML | 自包含 HTML5，内联 CSS，XSS 转义 |
| 通知渠道 | `notification` | 中 | Wave 8 / P5 与第 1 轮已实现核心渠道与 sink 模型 | `Notifier` trait + Webhook / Console / Feishu / DingTalk / WeCom / Slack / Discord / ntfy |
| 最小 AI 分析旁路 | `ai` | 低 | 最小 provider + app 旁路集成 | `mock` provider、最小 `openai-compatible` provider、配置字段与 Markdown 分析输出已落地 |
| 最小 MCP 查询工具 | `mcp` | 低 | 查询型工具服务入口 | `tools/list` 与查询类 `tools/call` 已落地 |
| AI 翻译 | 暂不进入核心 crate | 低 | 延后 | 当前不阻塞迁移收尾 |
| 版本检查 / 自动开浏览器 / 复杂环境分支 | 不迁移 | 无 | 删除 | 不进入 Rust 首版 |

## 推荐任务切分

迁移时建议按下面的最小单元推进：

1. 明确一个 crate 的输入、输出和依赖方向。
2. 为这个 crate 补 fixture、系统测试样例或 parity 样例。
3. 实现该 crate 的最小闭环。
4. 跑固定验证命令并记录结果。
5. 更新本映射文档和开发日志。

## 当前边界证据

Wave 3 当前已经补出几条和 `app` 边界直接相关的系统证据：

- `app` 只消费显式传入的 source 列表；根级系统测试已经覆盖空来源、单来源、双来源，以及 RSS-only / hotlist-only 几类核心输入形态
- `app` 不要求未启用 source 的配置字段成为隐式前提；例如 RSS-only 链路在 `platforms = []` 时仍可稳定完成
- `app` 会先消费调度决策再决定是否触碰 source；无论是显式 `collect=false`，还是 `schedule.window` 动态阻断，传入损坏 fixture 都不会提前触发解析错误
- `app` 是否触碰 source 只由抓取阶段是否启用决定；只要 `collect=true`，即使后续 `analyze` / `push` 关闭，损坏 source 也会立刻暴露
- `app` 对动态时间门控也保持同一原则：`schedule.window` 放行时会立刻传播上游解析错误，阻断时则完全跳过 source 解析
- `app` 已通过根级系统测试证明 `started_at + timezone` 在跨午夜窗口下同样具备“窗口内允许 / 窗口外禁止”的完整编排行为，而不是只在 `schedule` crate 内部成立
- `schedule` / `config` 负责时间窗口和时区合法性，`app` 只负责把 `started_at + timezone` 折算成本地上下文再交给上游决策

这些证据的目标是防止后续迁移把"必须有哪些 source / 平台 / 时间规则"重新塞回 `app`。

## Wave 5 边界证据

Wave 5 补齐生产就绪化，新增以下边界约束：

- `app` 对 fetch 错误的处理由 `resilient` 参数控制：fixture pipeline（`resilient=false`）传播错误，HTTP pipeline（`resilient=true`）仅记录警告并跳过；系统级测试已覆盖两种行为
- `app` 的配置文件来源由 `resolve_config_path` 控制，支持三级自动发现；不要求用户显式指定 `--config`
- `app` 的数据库路径由 `--db` 或 `default_db_path` 自动推导，不硬编码在配置文件中
- `app` 的日志输出使用 `tracing` 框架，不使用 `println!` 或 `eprintln!`（main.rs 的 JSON 输出除外）
- `app` 的关键词过滤由 `AppConfig.keywords` 驱动，空列表时不过滤，不引入额外过滤 crate
- `app` 的 HTTP 超时由 `AppConfig.http_timeout_secs` 统一控制，不分散在各个 fetcher 构造中

## Wave 6 边界证据

Wave 6 完成首版闭合，新增以下边界约束：

- `report` 新增 `render_news_html()` 生成自包含 HTML5，不引入外部模板引擎；HTML 输出通过 `html_escape()` 防 XSS
- `notification` 作为独立 crate，不依赖 `config` 或 `app`；通过 `build_notifiers(...)` 工厂函数解耦配置
- `app` 的输出格式由 CLI `--output` 参数控制（`json` / `html` / `both`），不硬编码在配置文件中
- `app` 的通知在 pipeline push 阶段触发，通知失败仅记录警告不中断 pipeline（通知是旁路，不是主路径）
- `notification` 默认回退到 `ConsoleNotifier`（不配置 webhook 时），确保通知不会静默丢失
- `AppConfig.notification` 使用 `#[serde(default)]`，现有配置文件无需修改即可升级

## Wave 8 边界证据

Wave 8 完成多通知渠道扩展，新增以下边界约束：

- `notification` crate 新增 `FeishuNotifier`、`DingTalkNotifier`、`WeComNotifier`，渠道 payload 细节仍封装在 crate 内部
- `build_notifiers(...)` 支持同时构建 webhook、飞书、钉钉、企业微信多个通知器
- `config` 只暴露 URL 级配置字段，不把各渠道协议逻辑泄漏到 `app`
- `app` 仍只在 push 阶段调用 notifier 列表，通知失败保持 warn+skip 旁路语义

## Wave 7 边界证据

Wave 7 完成 v1.1.0 首版增强，新增以下边界约束：

- `fetch` 新增 `HotlistParser` trait 作为解析策略抽象，新增平台只需实现 trait + 注册到工厂函数，不修改 `HttpHotlistFetcher` 主体（开闭原则）
- `fetch` 的 `HttpHotlistFetcher` 保持向后兼容：`new()` 和 `with_timeout()` 仍使用 `GenericHotlistParser`，新构造器 `with_parser()` 接受注入
- `config` 的 `HotlistApiConfig.source_type` 为可选字段，缺失时默认 `"generic"`，零破坏性升级
- `app` 的 pipeline 根据 `api.source_type` 选择 parser，parser 选择逻辑封装在 `hotlist_parser_for()` 工厂函数中
- `GenericHotlistParser` 当前除 fixture 数组格式外，也兼容 `newsnow` 的 `{"items":[...]}` 包装响应；第 1 轮已把 `douyin` / `wallstreetcn-hot` / `ifeng` / `tieba` 纳入这一兼容路径
- `app` 的 CLI `--output` 参数扩展为 `json / html / both / table / markdown`，路由逻辑在 main.rs
- `report` 新增 `render_news_table()` 和 `render_news_markdown()`，遵循现有 render 函数签名模式
- `PipelineResult` 扩展 `report_table` 和 `report_markdown` 字段，与 `report_json`/`report_html` 同级
- CI/CD release pipeline 通过 `.github/workflows/release.yml` 实现，tag 触发三平台构建，不修改应用代码

## 当前空白与后续补充点

当前映射表还是“能力级”基线，还没有细化到旧系统文件或模块名。

后续应按真实迁移进度补充：

- 旧 Python 模块名或目录路径
- 对应 Rust crate / 子模块
- 已完成、进行中、待开始状态
- 关键依赖和阻塞项

## 维护规则

- 新增 crate 或调整 crate 边界时，同轮更新本文件
- 决定删除或延后某类能力时，同轮更新本文件
- 如果旧系统某能力被拆成多个 Rust 模块，必须补一条说明，避免 AI 误判为重复实现

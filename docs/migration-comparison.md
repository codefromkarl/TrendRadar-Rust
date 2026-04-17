# 迁移前后项目对比：Python TrendRadar vs TrendRadar Rust

## 目标

这份文档用于把“迁移前是什么”和“迁移后现在是什么”放在同一张桌子上比较。

它不重复讲迁移计划，而是回答下面这些问题：

- 迁移前的原项目版本是什么
- 两边项目体量差多少
- 两边分别怎么启动
- 哪一边更偏完整生态，哪一边更偏轻量内核
- 迁移后到底保留了什么、删掉了什么、延后了什么

## 对比基线

本次对比采用下面两个快照：

- **迁移前基线**：原始 Python 项目 `TrendRadar v6.6.1`
- **迁移后快照**：当前 Rust 工作区（`v1.x` 迁移收口完成，已进入 `v2.x` 增量演进 / 生态扩展阶段）

说明：

- Python 版本号取自上游 `pyproject.toml` 的 `trendradar` 包版本。
- Rust 侧指标取自当前仓库工作树。
- 体积对比默认排除 `.git/` 与 Rust 构建产物 `target/`，避免把缓存算进项目本体。

## 一页结论

- **Python 原版更像“完整产品”**：功能面更广，部署路径更多，通知、AI、MCP 和 Docker / Actions 生态更成熟。
- **Rust 迁移版更像“收敛后的内核”**：主链路更小、更线性，CLI、配置、验证和部署边界更清楚，更适合做可验证、可维护的趋势监控内核。
- **如果你追求开箱即用的完整生态**，原版 Python 仍然更强。
- **如果你追求单机可控、结构清楚、验证统一、运维简单**，当前 Rust 版更顺手。

## 项目大小对比

### 仓库总体体积

| 指标 | Python 原版 | Rust 迁移版 |
| --- | --- | --- |
| 仓库体积（排除 `.git` / `target`） | `15M` | `2.3M` |
| 核心代码目录体积 | `1.4M`（`trendradar/` + `mcp_server/`） | `476K`（`crates/` + `src/`） |
| 核心源码文件数 | `64` 个 `.py` | `34` 个 `.rs` |
| 核心源码总行数 | `33,723` | `11,926` |
| Markdown 文档数 | `5` | `155` |

### 如何解读这些数字

- Python 原版体量更大，说明它承载了更多“完整产品层”的能力。
- Rust 版核心代码更小，说明迁移不是照搬，而是主动缩小了内核边界。
- Rust 文档明显更多，这不是偶然，而是迁移过程中把边界、验收、契约和验证口径都写进了仓库。

换句话说：

- **Python 更像功能先行的应用仓库**
- **Rust 更像文档、测试和边界先行的迁移工程仓库**

## 如何启动

### Python 原版

Python 原版当前有两条主路径：

1. **Docker 部署**：推荐长期运行，适合服务器 / NAS。
2. **GitHub Actions 部署**：无服务器运行，但需要远程云存储和 Secrets 配置。

典型启动路径更像这样：

```bash
git clone https://github.com/sansan0/TrendRadar.git
cd TrendRadar
# 准备 config/config.yaml、frequency_words.txt、timeline.yaml、docker/.env 等配置
docker compose pull
docker compose up -d
```

如果走包入口，它同时暴露：

- `trendradar`
- `trendradar-mcp`

这说明 Python 原版从一开始就把“推送服务 + MCP 服务 + 配置矩阵”一起交给用户。

### Rust 迁移版

Rust 版当前更强调“单次执行一轮 pipeline”的 CLI 模型。

官方入口更线性：

```bash
cargo install --path crates/app --locked
trendradar --config /path/to/config.json --db /path/to/trendradar.db
```

或：

```bash
curl -fsSL https://raw.githubusercontent.com/codefromkarl/TrendRadar-Rust/main/install.sh | bash
trendradar --help
```

容器路径也更偏 one-shot：

```bash
cp deploy/examples/config.rss.json deploy/runtime/config.json
mkdir -p deploy/runtime/data
docker compose -f deploy/docker-compose.yml run --rm trendradar --dry-run
```

### 启动复杂度判断

如果把“流畅度”拆成启动路径长度和前置认知负担，当前结论是：

- **Python 原版**：部署模式更多，但首次选择成本更高，配置面也更宽。
- **Rust 迁移版**：模式更少，但路径更直，单机 CLI / 定时任务场景更顺滑。

## 配置与使用流畅度

我把“使用流畅度”拆成 4 个维度看：

### 1. 首次上手

- **Python 原版**：更像“部署一个完整系统”，常见路径是 Docker / Actions / 多配置文件 / Secrets。
- **Rust 迁移版**：更像“安装一个 CLI 工具”，一份 `config.json` 加一个数据库路径即可起步。

### 2. 配置认知负担

- **Python 原版**：配置拆得更细，`config.yaml`、关键词文件、时间线文件、AI 提示词、自定义目录、`.env` 都可能参与运行。
- **Rust 迁移版**：当前把主链路尽量压到结构化 `config.json` 和显式 CLI 参数里，认知路径更短。

### 3. 运行模型一致性

- **Python 原版**：同时覆盖本地、Docker、GitHub Actions、MCP 服务、网页展示等多条路径，功能更完整，但语境切换也更多。
- **Rust 迁移版**：把运行模型固定为“一次执行一轮 pipeline”，更容易放进 `cron`、`systemd timer`、CI 或容器任务。

### 4. 本地验证与调试

- **Python 原版**：更偏应用使用与部署导向。
- **Rust 迁移版**：更偏工程验证导向，`just env-check`、`just verify-basic`、`just verify` 形成了清楚的本地门禁。

### 流畅度总结

- **面向最终使用者的功能丰富度**：Python 更流畅
- **面向开发者 / 维护者的可验证性与可控性**：Rust 更流畅

## 功能面差异

### Rust 当前已经保留下来的核心能力

- 多平台热榜抓取
- RSS 抓取
- 统一领域模型
- 关键词过滤、排序、来源聚合
- 时间窗口、工作日 / 周末、冷却周期
- SQLite 本地存储
- `json` / `html` / `table` / `markdown` 输出
- Webhook / 飞书 / 钉钉 / 企业微信 / Slack / Discord / ntfy / Console 通知
- 最小 AI 分析旁路
- 查询型 MCP 工具服务入口
- 最小远程对象存储闭环

### Python 原版仍然更完整的部分

- AI 翻译
- 更完整的 AI provider 矩阵
- 更完整的 MCP 协议与工具集
- 更大的通知渠道矩阵
- 更成熟的 Docker / GitHub Actions / 网页展示一体化路径
- 更细粒度的显示区域、配置模板和环境分支处理

### Rust 迁移版主动缩掉的部分

Rust 版不是“还没抄完”，而是明确做了取舍：

- 不追求把原版历史兼容层全部带过来
- 不把外部生态接入默认纳入核心完成标准
- 优先保留配置、抓取、分析、存储、输出这条主链路
- 把 AI、MCP、通知、远程存储放到更小、更可控的边界里

## 迁移前后差异的本质

如果只看表面，迁移像是：

- Python → Rust

但如果看结构，本质变化其实是：

- **完整生态应用** → **可验证的趋势监控内核**

因此 Rust 版的价值不只是“更快”，还包括：

- 更明确的 crate 边界
- 更统一的本地验证命令
- 更清晰的输入 / 输出契约
- 更容易做审查、回归和增量维护

而 Python 原版的价值仍然在于：

- 功能覆盖广
- 社区使用面大
- 部署教程成熟
- 生态整合更完整

## 适合谁

### 更适合继续使用 Python 原版的人

- 需要完整的现成功能矩阵
- 需要 GitHub Actions / Docker 双路径成熟部署
- 需要 AI 翻译、更多通知渠道和更完整 MCP
- 更重视“功能广度”而不是“内核收敛”

### 更适合使用 Rust 迁移版的人

- 想要一个更小、更稳、更可验证的趋势监控核心
- 想以 CLI、定时任务、容器作业的方式运行
- 想把部署和维护复杂度压低
- 更在意边界、验证和后续可维护性

## 相关文档

- [README.md](../README.md)
- [对外能力总览](./public-capability-overview.md)
- [配置迁移指南](./migration-guide.md)
- [Python 对比基线](./benchmark-python-baseline.md)
- [迁移策略](./migration-strategy.md)

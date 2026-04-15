# 对外能力总览

## 目标

这份文档用于给仓库外部读者提供一页式产品现状说明。

它不解释迁移过程，只回答下面这些外部问题：

- 现在支持什么
- 和 Python 版相比差异在哪里
- 性能提升有多少
- 部署难度如何
- 当前不支持什么

## 当前定位

TrendRadar Rust 当前已经完成内核闭环，正处于第 1 轮“对外可用化”阶段。

当前更适合这样理解：

- 它已经是一个可运行、可测试、可部署的趋势监控 CLI
- 它还不是 Python 原版的 100% 生态复刻

## 当前支持

### 热榜与 RSS

当前支持：

- 热榜抓取
- RSS 抓取
- 关键词过滤
- 排序与来源聚合
- 调度窗口与冷却周期
- SQLite 本地存储

### 热榜平台

当前明确支持的主流热榜平台 ID：

- `weibo`
- `zhihu`
- `bilibili`
- `toutiao`
- `baidu`
- `pengpai`
- `cls`
- `douyin`
- `wallstreetcn-hot`
- `ifeng`
- `tieba`

其中前 7 个带专门解析器，后 4 个按 `newsnow` 通用包装响应走 `generic` 兼容路径。

### 输出格式

CLI 当前支持：

- `json`
- `html`
- `both`
- `table`
- `markdown`

### 通知渠道

当前支持：

- Webhook
- 飞书
- 钉钉
- 企业微信
- Slack
- Discord
- ntfy
- Console 回退输出

## 性能数据

当前以 `fixture_pipeline_minimal` 作为主对比入口。

同机对比结果：

| 项目 | Rust | Python |
| --- | --- | --- |
| 主闭环中位耗时 | `163.96 µs` | `81.22 ms` |

粗略估算，Rust 当前约快 `495x`。

这个数字只对当前本机、当前 fixture 输入和当前 benchmark 入口成立，不外推到所有输入。

## 部署方式

当前官方支持 3 种部署方式：

1. Release 安装脚本
2. 源码 `cargo install`
3. Docker one-shot 运行

同时仓库提供 `systemd service/timer` 样例，适合服务器定时执行。

## 运行稳定性

当前稳定性证据来自两部分：

1. 工作区测试
2. 系统层重复执行与恢复测试

当前工作区测试通过口径：

- `235 tests passed`

系统层已覆盖：

- 慢源与失败源恢复
- 多轮重复执行输出一致性
- 大输入稳定性
- 多格式输出一致性

详见 [运行稳定性说明](./runtime-stability.md)。

## 与 Python 版相比

### Rust 当前更强的点

- 性能显著更高
- 单二进制 CLI 更易交付
- 测试与验证口径更统一
- 配置、抓取、分析、存储、输出边界更清晰

### Python 当前仍更完整的点

- 通知渠道生态更大
- 更完整的远程生态接入
- 更成熟的现成部署矩阵
- AI 翻译与更完整 MCP 兼容层

## 当前边界

下面这些能力当前仍未纳入 Rust 第 1 轮完成口径：

- AI 翻译
- 真实远程 LLM provider
- 完整 MCP 协议兼容层
- 更完整的远程对象存储 provider
- 更大范围分发入口


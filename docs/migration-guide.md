# Config 迁移指南：Python → Rust

本文档帮助从 Python TrendRadar 迁移到 Rust 版本。

## 快速对照

| 维度 | Python | Rust |
|------|--------|------|
| 配置格式 | YAML (`config.yaml`) | JSON (`config.json`) |
| 运行方式 | `python main.py` | `trendradar` |
| 数据存储 | SQLite (文件) | SQLite (文件) |
| 输出格式 | HTML / JSON | JSON / HTML / Table / Markdown |
| 通知渠道 | 9 种 | 5 种（webhook / console / 飞书 / 钉钉 / 企业微信） |

## 配置字段映射

### 基础配置

| Python 字段 | Rust 字段 | 说明 |
|-------------|-----------|------|
| `timezone` | `timezone` | 相同，IANA 时区名 |
| `schedule.enabled` | `schedule.collect` / `analyze` / `push` | 拆分为三个独立布尔开关 |
| `schedule.window.start_hour` | `schedule.window.startHour` | 相同，0-23 |
| `schedule.window.end_hour` | `schedule.window.endHour` | 相同，0-23 |

### 数据源配置

| Python 字段 | Rust 字段 | 说明 |
|-------------|-----------|------|
| `hot_boards[].name` | `hotlist_apis[].platform_id` | 平台标识 |
| `hot_boards[].url` | `hotlist_apis[].url` | API URL |
| — | `hotlist_apis[].source_type` | 数据源类型（`"generic"`/`"weibo"`/`"zhihu"`/`"bilibili"`/`"toutiao"`/`"baidu"`/`"pengpai"`/`"cls"`，可选，默认 `"generic"`） |
| `rss_feeds[].name` | `rss_feeds[].source_id` | 订阅源标识 |
| `rss_feeds[].url` | `rss_feeds[].url` | Feed URL |
| `hot_boards[].platforms` | `platforms` | fixture 模式平台列表 |

### 新增字段（Rust 独有）

| Rust 字段 | 类型 | 默认值 | 说明 |
|-----------|------|--------|------|
| `http_timeout_secs` | `u64` | `30` | HTTP 请求超时秒数 |
| `keywords` | `string[]` | `[]` | 关键词过滤列表（空=不过滤） |
| `notification.enabled` | `bool` | `false` | 是否启用通知 |
| `notification.sinks` | `NotificationSinkConfig[]` | `[]` | 可扩展通知 sink 列表 |
| `notification.webhook_url` | `string?` | `null` | Webhook URL |
| `notification.feishu_webhook_url` | `string?` | `null` | 飞书机器人 Webhook URL |
| `notification.dingtalk_webhook_url` | `string?` | `null` | 钉钉机器人 Webhook URL |
| `notification.wecom_webhook_url` | `string?` | `null` | 企业微信机器人 Webhook URL |
| `storage.backend` | `string` | `"sqlite"` | 存储后端类型（当前支持 `sqlite` / `s3`） |
| `storage.remote` | `object?` | `null` | 远程对象存储配置；`provider` 支持 `s3` / `aws-s3` / `oss` / `aliyun-oss` / `mock-s3` |
| `ai_analysis.enabled` | `bool` | `false` | 是否启用 AI 分析旁路 |
| `ai_analysis.provider` | `string` | `"mock"` | AI provider 名称 |
| `ai_analysis.timeout_secs` | `u64` | `15` | AI 分析超时秒数 |
| `ai_analysis.retry_attempts` | `u8` | `0` | AI 分析重试次数 |
| `ai_analysis.max_items` | `usize` | `5` | AI 分析最大纳入条目数 |
| `ai_analysis.prompt` | `string?` | `null` | 可选 AI prompt 提示 |
| `ai_analysis.model` | `string?` | `null` | 真实 provider 使用的模型名 |
| `ai_analysis.base_url` | `string?` | `null` | 真实 provider 的 API URL |
| `ai_analysis.api_key` | `string?` | `null` | 直接提供的 API key |
| `ai_analysis.api_key_env` | `string?` | `null` | API key 对应环境变量名 |
| `hotlist_apis[].source_type` | `string?` | `"generic"` | 热榜数据源类型 |

## 最小配置示例

```json
{
  "timezone": "Asia/Shanghai",
  "rss_feeds": [
    { "source_id": "rust-blog", "url": "https://blog.rust-lang.org/feed.xml" }
  ],
  "hotlist_apis": [
    { "platform_id": "weibo", "url": "https://example.com/api/hotlist", "source_type": "weibo" }
  ]
}
```

带通知和关键词过滤：

```json
{
  "timezone": "Asia/Shanghai",
  "keywords": ["rust", "ai", "open source"],
  "http_timeout_secs": 15,
  "notification": {
    "enabled": true,
    "sinks": [
      {
        "kind": "slack",
        "url": "https://hooks.slack.com/services/T000/B000/XXX"
      }
    ],
    "webhook_url": "https://hooks.example.com/trendradar",
    "feishu_webhook_url": "https://open.feishu.cn/open-apis/bot/v2/hook/xxx",
    "dingtalk_webhook_url": "https://oapi.dingtalk.com/robot/send?access_token=xxx",
    "wecom_webhook_url": "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=xxx"
  },
  "rss_feeds": [
    { "source_id": "rust-blog", "url": "https://blog.rust-lang.org/feed.xml" }
  ]
}
```

说明：

- 推荐新配置优先使用 `notification.sinks`
- 旧平铺字段当前仍兼容，可与 `sinks` 并存
- 如果同类型同 URL 同时出现在 `sinks` 和旧字段中，Rust 侧会去重，避免重复发送

## CLI 参数对照

| Python | Rust | 说明 |
|--------|------|------|
| `python main.py --config config.yaml` | `trendradar --config config.json` | 指定配置文件 |
| — | `trendradar --db data/trendradar.db` | 指定数据库路径 |
| — | `trendradar --output html` | 输出格式（json/html/both/table/markdown） |
| — | `trendradar --verbose` | 详细日志 |
| — | `trendradar --dry-run` | 仅打印调度决策 |
| — | `trendradar --version` | 版本信息 |
| — | `trendradar --help` | 帮助信息 |

## 配置文件自动发现

Rust 版本按以下顺序搜索配置文件（无需手动指定 `--config`）：

1. `./config.json` — 当前目录
2. `~/.config/trendradar/config.json` — 用户配置
3. `/etc/trendradar/config.json` — 系统配置

## 不再支持的功能

以下 Python 功能在 Rust 当前版本中仍未迁移：

- ❌ 真实远程 LLM provider 与 AI 翻译
- ❌ 完整 MCP 协议兼容层
- ❌ 更大范围的通知渠道（Telegram/Email/ntfy/Bark/Slack）
- ❌ 自动打开浏览器
- ❌ 版本在线检查
- ❌ Docker 集成

这些功能可能在后续版本中按需重新实现。

## 数据存储

- 数据库位置：默认在配置文件同目录下的 `trendradar.db`
- 可通过 `--db` 参数指定自定义路径
- 自动创建父目录
- Schema 与 Python 版本兼容（news_items 表）

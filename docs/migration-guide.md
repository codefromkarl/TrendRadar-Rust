# Config 迁移指南：Python → Rust

本文档帮助从 Python TrendRadar 迁移到 Rust 版本。

## 快速对照

| 维度 | Python | Rust |
|------|--------|------|
| 配置格式 | YAML (`config.yaml`) | JSON (`config.json`) |
| 运行方式 | `python main.py` | `trendradar` |
| 数据存储 | SQLite (文件) | SQLite (文件) |
| 输出格式 | HTML / JSON | JSON / HTML |
| 通知渠道 | 9 种 | 1 种 (webhook) + console |

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
| — | `hotlist_apis[].source_type` | 数据源类型（`"generic"`/`"weibo"`/`"zhihu"`/`"bilibili"`/`"toutiao"`/`"baidu"`，可选，默认 `"generic"`） |
| `rss_feeds[].name` | `rss_feeds[].source_id` | 订阅源标识 |
| `rss_feeds[].url` | `rss_feeds[].url` | Feed URL |
| `hot_boards[].platforms` | `platforms` | fixture 模式平台列表 |

### 新增字段（Rust 独有）

| Rust 字段 | 类型 | 默认值 | 说明 |
|-----------|------|--------|------|
| `http_timeout_secs` | `u64` | `30` | HTTP 请求超时秒数 |
| `keywords` | `string[]` | `[]` | 关键词过滤列表（空=不过滤） |
| `notification.enabled` | `bool` | `false` | 是否启用通知 |
| `notification.webhook_url` | `string?` | `null` | Webhook URL |
| `notification.feishu_webhook_url` | `string?` | `null` | 飞书机器人 Webhook URL |
| `notification.dingtalk_webhook_url` | `string?` | `null` | 钉钉机器人 Webhook URL |
| `notification.wecom_webhook_url` | `string?` | `null` | 企业微信机器人 Webhook URL |
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

以下 Python 功能在 Rust 首版中明确不迁移：

- ❌ AI 分析 / AI 翻译
- ❌ MCP Server
- ❌ 多通知渠道（飞书/钉钉/企业微信/Telegram/Email/ntfy/Bark/Slack）
- ❌ 自动打开浏览器
- ❌ 版本在线检查
- ❌ Docker 集成
- ❌ 多通知渠道（飞书/钉钉/企业微信/Telegram/Email/ntfy/Bark/Slack）
- ❌ 自动打开浏览器
- ❌ 版本在线检查
- ❌ Docker 集成

这些功能可能在后续版本中按需重新实现。

## 数据存储

- 数据库位置：默认在配置文件同目录下的 `trendradar.db`
- 可通过 `--db` 参数指定自定义路径
- 自动创建父目录
- Schema 与 Python 版本兼容（news_items 表）

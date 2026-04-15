//! 配置模型与加载入口。

use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::path::Path;
use trendradar_domain::{Result, TrendRadarError};

/// 调度配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleWindowConfig {
    /// 本地时区窗口起始小时，范围 0-23。
    pub start_hour: u8,
    /// 本地时区窗口结束小时，范围 0-23。
    pub end_hour: u8,
}

/// 工作日或周末调度覆盖配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScheduleOverrideConfig {
    /// 可选覆盖抓取阶段开关。
    #[serde(default)]
    pub collect: Option<bool>,
    /// 可选覆盖分析阶段开关。
    #[serde(default)]
    pub analyze: Option<bool>,
    /// 可选覆盖推送阶段开关。
    #[serde(default)]
    pub push: Option<bool>,
    /// 可选覆盖时间窗口。
    #[serde(default)]
    pub window: Option<ScheduleWindowConfig>,
}

/// 调度配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// 是否执行抓取阶段。
    pub collect: bool,
    /// 是否执行分析阶段。
    pub analyze: bool,
    /// 是否执行推送阶段。
    pub push: bool,
    /// 可选的本地时区小时窗口。
    #[serde(default)]
    pub window: Option<ScheduleWindowConfig>,
    /// 冷却周期，单位分钟。
    #[serde(default)]
    pub cooldown_minutes: Option<u64>,
    /// 工作日覆盖规则。
    #[serde(default)]
    pub weekday: Option<ScheduleOverrideConfig>,
    /// 周末覆盖规则。
    #[serde(default)]
    pub weekend: Option<ScheduleOverrideConfig>,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            collect: true,
            analyze: true,
            push: true,
            window: None,
            cooldown_minutes: None,
            weekday: None,
            weekend: None,
        }
    }
}

/// RSS 订阅源配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RssFeedConfig {
    /// 订阅源标识。
    pub source_id: String,
    /// Feed URL。
    pub url: String,
    /// 可选最大条目数。
    #[serde(default)]
    pub max_items: Option<usize>,
}

/// 热榜 API 配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotlistApiConfig {
    /// 平台标识。
    pub platform_id: String,
    /// API URL。
    pub url: String,
    /// 热榜数据源类型，用于选择解析策略。
    #[serde(default)]
    pub source_type: Option<String>,
}

/// 存储后端类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackend {
    /// 本地 SQLite。
    #[default]
    Sqlite,
    /// 远程对象存储（预留）。
    S3,
}

/// 远程对象存储配置（预留）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RemoteStorageConfig {
    /// 提供方标识，例如 `s3` / `oss`。
    #[serde(default)]
    pub provider: Option<String>,
    /// 远程 bucket 名称。
    #[serde(default)]
    pub bucket: Option<String>,
    /// 可选 endpoint。
    #[serde(default)]
    pub endpoint: Option<String>,
    /// 可选 region。
    #[serde(default)]
    pub region: Option<String>,
    /// 可选对象前缀。
    #[serde(default)]
    pub prefix: Option<String>,
}

/// 存储配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StorageConfig {
    /// 存储后端类型。
    #[serde(default)]
    pub backend: StorageBackend,
    /// 远程对象存储配置（预留）。
    #[serde(default)]
    pub remote: Option<RemoteStorageConfig>,
}

/// AI 分析配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiAnalysisConfig {
    /// 是否启用 AI 分析。
    #[serde(default)]
    pub enabled: bool,
    /// provider 名称。
    #[serde(default = "default_ai_provider")]
    pub provider: String,
    /// 超时秒数。
    #[serde(default = "default_ai_timeout_secs")]
    pub timeout_secs: u64,
    /// 重试次数。
    #[serde(default)]
    pub retry_attempts: u8,
    /// 分析时最多纳入的条目数。
    #[serde(default = "default_ai_max_items")]
    pub max_items: usize,
    /// 可选 prompt 提示。
    #[serde(default)]
    pub prompt: Option<String>,
    /// 真实 provider 使用的模型名称。
    #[serde(default)]
    pub model: Option<String>,
    /// 真实 provider 的 API base URL。
    #[serde(default)]
    pub base_url: Option<String>,
    /// 直接提供 API key，适合受控测试或局部运行配置。
    #[serde(default)]
    pub api_key: Option<String>,
    /// 读取 API key 的环境变量名。
    #[serde(default)]
    pub api_key_env: Option<String>,
}

impl Default for AiAnalysisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_ai_provider(),
            timeout_secs: default_ai_timeout_secs(),
            retry_attempts: 0,
            max_items: default_ai_max_items(),
            prompt: None,
            model: None,
            base_url: None,
            api_key: None,
            api_key_env: None,
        }
    }
}

/// 应用配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    /// 时区名称。
    pub timezone: String,
    /// 热榜平台列表（fixture 模式）。
    #[serde(default)]
    pub platforms: Vec<String>,
    /// 调度配置。
    #[serde(default)]
    pub schedule: ScheduleConfig,
    /// RSS 订阅源列表（HTTP 模式）。
    #[serde(default)]
    pub rss_feeds: Vec<RssFeedConfig>,
    /// 热榜 API 列表（HTTP 模式）。
    #[serde(default)]
    pub hotlist_apis: Vec<HotlistApiConfig>,
    /// HTTP 请求超时秒数。
    #[serde(default = "default_http_timeout_secs")]
    pub http_timeout_secs: u64,
    /// 存储配置。
    #[serde(default)]
    pub storage: StorageConfig,
    /// AI 分析配置。
    #[serde(default)]
    pub ai_analysis: AiAnalysisConfig,
    /// 关键词过滤列表（不区分大小写）。
    #[serde(default)]
    pub keywords: Vec<String>,
    /// 通知配置。
    #[serde(default)]
    pub notification: NotificationConfig,
}

/// 默认 HTTP 超时 30 秒。
const fn default_http_timeout_secs() -> u64 {
    30
}

fn default_ai_provider() -> String {
    "mock".to_owned()
}

const fn default_ai_timeout_secs() -> u64 {
    15
}

const fn default_ai_max_items() -> usize {
    5
}

/// 通知配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NotificationConfig {
    /// 是否启用通知。
    #[serde(default)]
    pub enabled: bool,
    /// 可扩展 sink 列表。
    #[serde(default)]
    pub sinks: Vec<NotificationSinkConfig>,
    /// Webhook URL。
    #[serde(default)]
    pub webhook_url: Option<String>,
    /// 飞书 Webhook URL。
    #[serde(default)]
    pub feishu_webhook_url: Option<String>,
    /// 钉钉 Webhook URL。
    #[serde(default)]
    pub dingtalk_webhook_url: Option<String>,
    /// 企业微信 Webhook URL。
    #[serde(default)]
    pub wecom_webhook_url: Option<String>,
    /// Discord Webhook URL。
    #[serde(default)]
    pub discord_webhook_url: Option<String>,
    /// ntfy topic URL。
    #[serde(default)]
    pub ntfy_topic_url: Option<String>,
}

/// 通知 sink 类型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationSinkKind {
    /// 通用 webhook。
    Webhook,
    /// 飞书 webhook。
    Feishu,
    /// 钉钉 webhook。
    Dingtalk,
    /// 企业微信 webhook。
    Wecom,
    /// Slack webhook。
    Slack,
    /// Discord webhook。
    Discord,
    /// ntfy topic。
    Ntfy,
}

/// 单个通知 sink 配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationSinkConfig {
    /// sink 类型。
    pub kind: NotificationSinkKind,
    /// webhook URL。
    pub url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: Vec::new(),
            schedule: ScheduleConfig::default(),
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
            http_timeout_secs: default_http_timeout_secs(),
            storage: StorageConfig::default(),
            ai_analysis: AiAnalysisConfig::default(),
            keywords: Vec::new(),
            notification: NotificationConfig::default(),
        }
    }
}

/// 校验应用配置。
pub fn validate_config(config: AppConfig) -> Result<AppConfig> {
    if config.timezone.is_empty() {
        return Err(TrendRadarError::InvalidConfig {
            message: "timezone must not be empty".to_owned(),
        });
    }

    config
        .timezone
        .parse::<Tz>()
        .map_err(|_| TrendRadarError::InvalidConfig {
            message: "timezone must be a valid IANA timezone".to_owned(),
        })?;

    validate_schedule_window(config.schedule.window.as_ref())?;
    if let Some(weekday) = &config.schedule.weekday {
        validate_schedule_window(weekday.window.as_ref())?;
    }
    if let Some(weekend) = &config.schedule.weekend {
        validate_schedule_window(weekend.window.as_ref())?;
    }

    Ok(config)
}

fn validate_schedule_window(window: Option<&ScheduleWindowConfig>) -> Result<()> {
    if let Some(window) = window {
        if window.start_hour > 23 || window.end_hour > 23 {
            return Err(TrendRadarError::InvalidConfig {
                message: "schedule window hours must be between 0 and 23".to_owned(),
            });
        }

        if window.start_hour == window.end_hour {
            return Err(TrendRadarError::InvalidConfig {
                message: "schedule window start_hour and end_hour must not be equal".to_owned(),
            });
        }
    }

    Ok(())
}

/// 从 JSON 字符串加载配置。
pub fn load_config_from_json_str(input: &str) -> Result<AppConfig> {
    let config = from_str::<AppConfig>(input).map_err(|error| TrendRadarError::InvalidConfig {
        message: format!("failed to parse config json: {error}"),
    })?;

    validate_config(config)
}

/// 加载默认配置。
pub fn load_default_config() -> Result<AppConfig> {
    validate_config(AppConfig::default())
}

/// 从 JSON 文件加载配置。
pub fn load_config_from_file(path: &Path) -> Result<AppConfig> {
    let contents =
        std::fs::read_to_string(path).map_err(|error| TrendRadarError::InvalidConfig {
            message: format!("failed to read config file {}: {error}", path.display()),
        })?;
    load_config_from_json_str(&contents)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        AiAnalysisConfig, AppConfig, NotificationConfig, NotificationSinkKind, RemoteStorageConfig,
        ScheduleConfig, ScheduleWindowConfig, StorageBackend, StorageConfig,
        load_config_from_json_str,
    };
    use std::error::Error;
    use std::fs::read_to_string;

    fn schedule_fixture_path(name: &str) -> String {
        format!(
            "{}/../../fixtures/system/schedule/{name}",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    fn config_fixture_path(name: &str) -> String {
        format!(
            "{}/../../fixtures/system/config/{name}",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    #[test]
    fn missing_schedule_uses_default_values() -> Result<(), Box<dyn Error>> {
        let input = r#"{"timezone":"Asia/Shanghai","platforms":["weibo"]}"#;
        let config = load_config_from_json_str(input)?;

        assert_eq!(config.schedule, ScheduleConfig::default());
        Ok(())
    }

    #[test]
    fn explicit_schedule_is_loaded_from_json() -> Result<(), Box<dyn Error>> {
        let input = r#"{
            "timezone":"Asia/Shanghai",
            "platforms":["weibo"],
            "schedule":{"collect":true,"analyze":true,"push":false}
        }"#;
        let config = load_config_from_json_str(input)?;

        assert_eq!(
            config,
            AppConfig {
                timezone: "Asia/Shanghai".to_owned(),
                platforms: vec!["weibo".to_owned()],
                schedule: ScheduleConfig {
                    collect: true,
                    analyze: true,
                    push: false,
                    window: None,
                    cooldown_minutes: None,
                    weekday: None,
                    weekend: None,
                },
                rss_feeds: Vec::new(),
                hotlist_apis: Vec::new(),
                http_timeout_secs: 30,
                storage: StorageConfig::default(),
                ai_analysis: AiAnalysisConfig::default(),
                keywords: Vec::new(),
                notification: NotificationConfig::default(),
            }
        );
        Ok(())
    }

    #[test]
    fn schedule_window_is_loaded_from_fixture() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(schedule_fixture_path("window-daytime.json"))?;
        let config = load_config_from_json_str(&fixture)?;

        assert_eq!(
            config.schedule.window,
            Some(ScheduleWindowConfig {
                start_hour: 9,
                end_hour: 18,
            })
        );
        Ok(())
    }

    #[test]
    fn schedule_window_with_equal_hours_is_rejected() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(schedule_fixture_path("invalid-window-equal-hours.json"))?;
        let error = load_config_from_json_str(&fixture).expect_err("fixture should be rejected");

        assert_eq!(
            error.to_string(),
            "invalid config: schedule window start_hour and end_hour must not be equal"
        );
        Ok(())
    }

    #[test]
    fn schedule_window_with_out_of_range_hour_is_rejected() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(schedule_fixture_path("invalid-window-out-of-range.json"))?;
        let error = load_config_from_json_str(&fixture).expect_err("fixture should be rejected");

        assert_eq!(
            error.to_string(),
            "invalid config: schedule window hours must be between 0 and 23"
        );
        Ok(())
    }

    #[test]
    fn unknown_timezone_is_rejected() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(config_fixture_path("invalid-unknown-timezone-window.json"))?;
        let error = load_config_from_json_str(&fixture).expect_err("fixture should be rejected");

        assert_eq!(
            error.to_string(),
            "invalid config: timezone must be a valid IANA timezone"
        );
        Ok(())
    }

    #[test]
    fn rss_feeds_and_hotlist_apis_load_from_json() -> Result<(), Box<dyn Error>> {
        let input = r#"{
            "timezone":"Asia/Shanghai",
            "rss_feeds":[{"source_id":"rust-blog","url":"https://blog.rust-lang.org/feed.xml","max_items":20}],
            "hotlist_apis":[{"platform_id":"weibo","url":"https://example.com/api/hotlist"}]
        }"#;
        let config = load_config_from_json_str(input)?;

        assert_eq!(config.rss_feeds.len(), 1);
        assert_eq!(config.rss_feeds[0].source_id, "rust-blog");
        assert_eq!(
            config.rss_feeds[0].url,
            "https://blog.rust-lang.org/feed.xml"
        );
        assert_eq!(config.rss_feeds[0].max_items, Some(20));
        assert_eq!(config.hotlist_apis.len(), 1);
        assert_eq!(config.hotlist_apis[0].platform_id, "weibo");
        Ok(())
    }

    #[test]
    fn missing_sources_default_to_empty() -> Result<(), Box<dyn Error>> {
        let input = r#"{"timezone":"UTC"}"#;
        let config = load_config_from_json_str(input)?;

        assert!(config.platforms.is_empty());
        assert!(config.rss_feeds.is_empty());
        assert!(config.hotlist_apis.is_empty());
        assert_eq!(config.storage, StorageConfig::default());
        assert_eq!(config.ai_analysis, AiAnalysisConfig::default());
        Ok(())
    }

    #[test]
    fn remote_storage_config_loads_from_json() -> Result<(), Box<dyn Error>> {
        let input = r#"{
            "timezone":"Asia/Shanghai",
            "storage":{
                "backend":"s3",
                "remote":{
                    "provider":"s3",
                    "bucket":"trendradar-artifacts",
                    "endpoint":"https://storage.example.com",
                    "region":"cn-hangzhou",
                    "prefix":"daily/"
                }
            }
        }"#;
        let config = load_config_from_json_str(input)?;

        assert_eq!(config.storage.backend, StorageBackend::S3);
        assert_eq!(
            config.storage.remote,
            Some(RemoteStorageConfig {
                provider: Some("s3".to_owned()),
                bucket: Some("trendradar-artifacts".to_owned()),
                endpoint: Some("https://storage.example.com".to_owned()),
                region: Some("cn-hangzhou".to_owned()),
                prefix: Some("daily/".to_owned()),
            })
        );
        Ok(())
    }

    #[test]
    fn ai_analysis_config_loads_from_json() -> Result<(), Box<dyn Error>> {
        let input = r#"{
            "timezone":"Asia/Shanghai",
            "ai_analysis":{
                "enabled":true,
                "provider":"mock",
                "timeout_secs":20,
                "retry_attempts":2,
                "max_items":3,
                "prompt":"focus on ai",
                "model":"gpt-4.1-mini",
                "base_url":"https://example.com/v1/responses",
                "api_key_env":"OPENAI_API_KEY"
            }
        }"#;
        let config = load_config_from_json_str(input)?;

        assert_eq!(
            config.ai_analysis,
            AiAnalysisConfig {
                enabled: true,
                provider: "mock".to_owned(),
                timeout_secs: 20,
                retry_attempts: 2,
                max_items: 3,
                prompt: Some("focus on ai".to_owned()),
                model: Some("gpt-4.1-mini".to_owned()),
                base_url: Some("https://example.com/v1/responses".to_owned()),
                api_key: None,
                api_key_env: Some("OPENAI_API_KEY".to_owned()),
            }
        );
        Ok(())
    }

    #[test]
    fn notification_channels_load_from_json() -> Result<(), Box<dyn Error>> {
        let input = r#"{
            "timezone":"Asia/Shanghai",
            "notification":{
                "enabled":true,
                "webhook_url":"https://hooks.example.com/webhook",
                "feishu_webhook_url":"https://open.feishu.cn/webhook",
                "dingtalk_webhook_url":"https://oapi.dingtalk.com/robot/send",
                "wecom_webhook_url":"https://qyapi.weixin.qq.com/cgi-bin/webhook/send"
            }
        }"#;
        let config = load_config_from_json_str(input)?;

        assert!(config.notification.enabled);
        assert_eq!(
            config.notification.webhook_url.as_deref(),
            Some("https://hooks.example.com/webhook")
        );
        assert_eq!(
            config.notification.feishu_webhook_url.as_deref(),
            Some("https://open.feishu.cn/webhook")
        );
        assert_eq!(
            config.notification.dingtalk_webhook_url.as_deref(),
            Some("https://oapi.dingtalk.com/robot/send")
        );
        assert_eq!(
            config.notification.wecom_webhook_url.as_deref(),
            Some("https://qyapi.weixin.qq.com/cgi-bin/webhook/send")
        );
        Ok(())
    }

    #[test]
    fn notification_sinks_load_from_json() -> Result<(), Box<dyn Error>> {
        let input = r#"{
            "timezone":"Asia/Shanghai",
            "notification":{
                "enabled":true,
                "sinks":[
                    {"kind":"webhook","url":"https://hooks.example.com/webhook"},
                    {"kind":"slack","url":"https://hooks.slack.com/services/test"},
                    {"kind":"discord","url":"https://discord.com/api/webhooks/test"},
                    {"kind":"ntfy","url":"https://ntfy.sh/trendradar"}
                ]
            }
        }"#;
        let config = load_config_from_json_str(input)?;

        assert!(config.notification.enabled);
        assert_eq!(config.notification.sinks.len(), 4);
        assert_eq!(
            config.notification.sinks[0].kind,
            NotificationSinkKind::Webhook
        );
        assert_eq!(
            config.notification.sinks[0].url,
            "https://hooks.example.com/webhook"
        );
        assert_eq!(
            config.notification.sinks[1].kind,
            NotificationSinkKind::Slack
        );
        assert_eq!(
            config.notification.sinks[1].url,
            "https://hooks.slack.com/services/test"
        );
        assert_eq!(
            config.notification.sinks[2].kind,
            NotificationSinkKind::Discord
        );
        assert_eq!(
            config.notification.sinks[2].url,
            "https://discord.com/api/webhooks/test"
        );
        assert_eq!(
            config.notification.sinks[3].kind,
            NotificationSinkKind::Ntfy
        );
        assert_eq!(
            config.notification.sinks[3].url,
            "https://ntfy.sh/trendradar"
        );
        Ok(())
    }

    #[test]
    fn legacy_notification_channels_load_from_json() -> Result<(), Box<dyn Error>> {
        let input = r#"{
            "timezone":"Asia/Shanghai",
            "notification":{
                "enabled":true,
                "discord_webhook_url":"https://discord.com/api/webhooks/legacy",
                "ntfy_topic_url":"https://ntfy.sh/trendradar"
            }
        }"#;
        let config = load_config_from_json_str(input)?;

        assert_eq!(
            config.notification.discord_webhook_url.as_deref(),
            Some("https://discord.com/api/webhooks/legacy")
        );
        assert_eq!(
            config.notification.ntfy_topic_url.as_deref(),
            Some("https://ntfy.sh/trendradar")
        );
        Ok(())
    }

    #[test]
    fn missing_notification_channels_default_to_none() -> Result<(), Box<dyn Error>> {
        let config = load_config_from_json_str(r#"{"timezone":"Asia/Shanghai"}"#)?;

        assert_eq!(config.notification, NotificationConfig::default());
        Ok(())
    }

    #[test]
    fn cooldown_minutes_load_from_json() -> Result<(), Box<dyn Error>> {
        let input = r#"{
            "timezone":"Asia/Shanghai",
            "schedule":{
                "collect":true,
                "analyze":true,
                "push":true,
                "cooldown_minutes":45
            }
        }"#;
        let config = load_config_from_json_str(input)?;

        assert_eq!(config.schedule.cooldown_minutes, Some(45));
        Ok(())
    }

    #[test]
    fn load_config_from_file_reads_valid_fixture() -> Result<(), Box<dyn Error>> {
        let path = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/config/minimal-valid.json"
        ));
        let config = super::load_config_from_file(path)?;

        assert_eq!(config.timezone, "Asia/Shanghai");
        Ok(())
    }

    #[test]
    fn load_config_from_file_reports_missing_file() -> Result<(), Box<dyn Error>> {
        let path = std::path::Path::new("/nonexistent/config.json");
        let error = super::load_config_from_file(path).expect_err("should fail on missing file");

        assert!(error.to_string().contains("failed to read config file"));
        Ok(())
    }
}

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
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            collect: true,
            analyze: true,
            push: true,
            window: None,
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
}

/// 热榜 API 配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotlistApiConfig {
    /// 平台标识。
    pub platform_id: String,
    /// API URL。
    pub url: String,
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: Vec::new(),
            schedule: ScheduleConfig::default(),
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
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

    if let Some(window) = &config.schedule.window {
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

    Ok(config)
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
mod tests {
    use super::{AppConfig, ScheduleConfig, ScheduleWindowConfig, load_config_from_json_str};
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
                },
                rss_feeds: Vec::new(),
                hotlist_apis: Vec::new(),
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
            "rss_feeds":[{"source_id":"rust-blog","url":"https://blog.rust-lang.org/feed.xml"}],
            "hotlist_apis":[{"platform_id":"weibo","url":"https://example.com/api/hotlist"}]
        }"#;
        let config = load_config_from_json_str(input)?;

        assert_eq!(config.rss_feeds.len(), 1);
        assert_eq!(config.rss_feeds[0].source_id, "rust-blog");
        assert_eq!(
            config.rss_feeds[0].url,
            "https://blog.rust-lang.org/feed.xml"
        );
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

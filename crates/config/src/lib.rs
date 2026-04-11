//! 配置模型与加载入口。

use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
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

/// 应用配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    /// 时区名称。
    pub timezone: String,
    /// 热榜平台列表。
    pub platforms: Vec<String>,
    /// 调度配置。
    #[serde(default)]
    pub schedule: ScheduleConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: Vec::new(),
            schedule: ScheduleConfig::default(),
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
}

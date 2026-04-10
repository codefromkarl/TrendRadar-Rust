//! 配置模型与加载入口。

use serde::{Deserialize, Serialize};
use serde_json::from_str;
use trendradar_domain::{Result, TrendRadarError};

/// 调度配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// 是否执行抓取阶段。
    pub collect: bool,
    /// 是否执行分析阶段。
    pub analyze: bool,
    /// 是否执行推送阶段。
    pub push: bool,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            collect: true,
            analyze: true,
            push: true,
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
    use super::{AppConfig, ScheduleConfig, load_config_from_json_str};
    use std::error::Error;

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
                },
            }
        );
        Ok(())
    }
}

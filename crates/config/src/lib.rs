//! 配置模型与加载入口。

use serde::{Deserialize, Serialize};
use serde_json::from_str;
use trendradar_domain::{Result, TrendRadarError};

/// 应用配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    /// 时区名称。
    pub timezone: String,
    /// 热榜平台列表。
    pub platforms: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: Vec::new(),
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

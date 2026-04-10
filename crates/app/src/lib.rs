//! 应用编排层骨架。

use trendradar_config::{AppConfig, load_default_config, validate_config};

/// 返回应用标识。
#[must_use]
pub fn app_name() -> &'static str {
    "trendradar-rust"
}

/// 验证基础编排依赖是否可用。
pub fn bootstrap() -> anyhow::Result<()> {
    let config = load_default_config()?;
    bootstrap_with_config(&config)?;
    Ok(())
}

/// 验证给定配置能否通过基础编排校验。
pub fn bootstrap_with_config(config: &AppConfig) -> anyhow::Result<()> {
    let _ = validate_config(config.clone())?;
    Ok(())
}

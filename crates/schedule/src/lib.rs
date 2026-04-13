//! 调度解析骨架。

use serde::{Deserialize, Serialize};
use trendradar_config::{AppConfig, ScheduleConfig, ScheduleWindowConfig};

/// 调度决策。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleDecision {
    /// 是否抓取。
    pub collect: bool,
    /// 是否分析。
    pub analyze: bool,
    /// 是否推送。
    pub push: bool,
}

impl Default for ScheduleDecision {
    fn default() -> Self {
        Self {
            collect: true,
            analyze: true,
            push: true,
        }
    }
}

/// 调度上下文。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleContext {
    /// 已按配置时区折算后的本地小时，范围 0-23。
    pub local_hour: u8,
}

/// 从调度配置生成决策。
#[must_use]
pub fn decision_from_schedule(schedule: &ScheduleConfig) -> ScheduleDecision {
    ScheduleDecision {
        collect: schedule.collect,
        analyze: schedule.analyze,
        push: schedule.push,
    }
}

fn window_matches_hour(window: &ScheduleWindowConfig, local_hour: u8) -> bool {
    if local_hour > 23 {
        return false;
    }

    if window.start_hour < window.end_hour {
        (window.start_hour..window.end_hour).contains(&local_hour)
    } else {
        local_hour >= window.start_hour || local_hour < window.end_hour
    }
}

/// 从调度配置和显式上下文生成决策。
#[must_use]
pub fn decision_from_schedule_at(
    schedule: &ScheduleConfig,
    context: ScheduleContext,
) -> ScheduleDecision {
    let base = decision_from_schedule(schedule);

    match &schedule.window {
        Some(window) if !window_matches_hour(window, context.local_hour) => ScheduleDecision {
            collect: false,
            analyze: false,
            push: false,
        },
        _ => base,
    }
}

/// 从应用配置生成调度决策。
#[must_use]
pub fn decision_from_config(config: &AppConfig) -> ScheduleDecision {
    decision_from_schedule(&config.schedule)
}

/// 从应用配置和显式上下文生成调度决策。
#[must_use]
pub fn decision_from_config_at(config: &AppConfig, context: ScheduleContext) -> ScheduleDecision {
    decision_from_schedule_at(&config.schedule, context)
}

#[cfg(test)]
mod tests {
    use super::{ScheduleContext, ScheduleDecision, decision_from_config, decision_from_config_at};
    use std::error::Error;
    use std::fs::read_to_string;
    use trendradar_config::{AppConfig, ScheduleConfig, load_config_from_json_str};

    fn schedule_fixture_path(name: &str) -> String {
        format!(
            "{}/../../fixtures/system/schedule/{name}",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    #[test]
    fn decision_follows_explicit_schedule_flags() {
        let config = AppConfig {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: vec!["weibo".to_owned()],
            schedule: ScheduleConfig {
                collect: false,
                analyze: true,
                push: false,
                window: None,
            },
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
            http_timeout_secs: 30,
            keywords: Vec::new(),
            notification: trendradar_config::NotificationConfig::default(),
        };

        assert_eq!(
            decision_from_config(&config),
            ScheduleDecision {
                collect: false,
                analyze: true,
                push: false,
            }
        );
    }

    #[test]
    fn decision_uses_default_schedule_from_fixture() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/config/minimal-valid.json"
        );
        let fixture = read_to_string(fixture_path)?;
        let config = load_config_from_json_str(&fixture)?;

        assert_eq!(decision_from_config(&config), ScheduleDecision::default());
        Ok(())
    }

    #[test]
    fn decision_respects_daytime_window() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(schedule_fixture_path("window-daytime.json"))?;
        let config = load_config_from_json_str(&fixture)?;

        assert_eq!(
            decision_from_config_at(&config, ScheduleContext { local_hour: 10 }),
            ScheduleDecision {
                collect: true,
                analyze: true,
                push: true,
            }
        );
        assert_eq!(
            decision_from_config_at(&config, ScheduleContext { local_hour: 20 }),
            ScheduleDecision {
                collect: false,
                analyze: false,
                push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn decision_supports_overnight_window() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(schedule_fixture_path("window-overnight.json"))?;
        let config = load_config_from_json_str(&fixture)?;

        assert_eq!(
            decision_from_config_at(&config, ScheduleContext { local_hour: 23 }),
            ScheduleDecision {
                collect: true,
                analyze: false,
                push: true,
            }
        );
        assert_eq!(
            decision_from_config_at(&config, ScheduleContext { local_hour: 12 }),
            ScheduleDecision {
                collect: false,
                analyze: false,
                push: false,
            }
        );
        Ok(())
    }
}

//! 调度解析骨架。

use serde::{Deserialize, Serialize};
use trendradar_config::{AppConfig, ScheduleConfig};

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

/// 从调度配置生成决策。
#[must_use]
pub fn decision_from_schedule(schedule: &ScheduleConfig) -> ScheduleDecision {
    ScheduleDecision {
        collect: schedule.collect,
        analyze: schedule.analyze,
        push: schedule.push,
    }
}

/// 从应用配置生成调度决策。
#[must_use]
pub fn decision_from_config(config: &AppConfig) -> ScheduleDecision {
    decision_from_schedule(&config.schedule)
}

#[cfg(test)]
mod tests {
    use super::{ScheduleDecision, decision_from_config};
    use std::error::Error;
    use std::fs::read_to_string;
    use trendradar_config::{AppConfig, ScheduleConfig, load_config_from_json_str};

    #[test]
    fn decision_follows_explicit_schedule_flags() {
        let config = AppConfig {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: vec!["weibo".to_owned()],
            schedule: ScheduleConfig {
                collect: false,
                analyze: true,
                push: false,
            },
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
}

//! 调度解析骨架。

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use trendradar_config::{AppConfig, ScheduleConfig, ScheduleOverrideConfig, ScheduleWindowConfig};

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
    /// 当前是否为周末。
    pub is_weekend: bool,
    /// 当前运行时间。
    pub current_time: DateTime<Utc>,
    /// 上次成功运行时间。
    pub last_success_at: Option<DateTime<Utc>>,
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

fn active_override(
    schedule: &ScheduleConfig,
    context: ScheduleContext,
) -> Option<&ScheduleOverrideConfig> {
    if context.is_weekend {
        schedule.weekend.as_ref()
    } else {
        schedule.weekday.as_ref()
    }
}

fn cooldown_active(schedule: &ScheduleConfig, context: ScheduleContext) -> bool {
    let Some(cooldown_minutes) = schedule.cooldown_minutes else {
        return false;
    };
    let Some(last_success_at) = context.last_success_at else {
        return false;
    };

    context.current_time < last_success_at + Duration::minutes(cooldown_minutes as i64)
}

/// 从调度配置和显式上下文生成决策。
#[must_use]
pub fn decision_from_schedule_at(
    schedule: &ScheduleConfig,
    context: ScheduleContext,
) -> ScheduleDecision {
    let override_config = active_override(schedule, context);
    let base = ScheduleDecision {
        collect: override_config
            .and_then(|config| config.collect)
            .unwrap_or(schedule.collect),
        analyze: override_config
            .and_then(|config| config.analyze)
            .unwrap_or(schedule.analyze),
        push: override_config
            .and_then(|config| config.push)
            .unwrap_or(schedule.push),
    };
    let effective_window = override_config
        .and_then(|config| config.window.as_ref())
        .or(schedule.window.as_ref());

    match effective_window {
        Some(window) if !window_matches_hour(window, context.local_hour) => ScheduleDecision {
            collect: false,
            analyze: false,
            push: false,
        },
        _ if cooldown_active(schedule, context) => ScheduleDecision {
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
    use chrono::{TimeZone, Utc};
    use std::error::Error;
    use std::fs::read_to_string;
    use trendradar_config::{
        AppConfig, ScheduleConfig, ScheduleOverrideConfig, ScheduleWindowConfig,
        load_config_from_json_str,
    };

    fn schedule_fixture_path(name: &str) -> String {
        format!(
            "{}/../../fixtures/system/schedule/{name}",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    fn context(local_hour: u8) -> ScheduleContext {
        ScheduleContext {
            local_hour,
            is_weekend: false,
            current_time: Utc::now(),
            last_success_at: None,
        }
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
                cooldown_minutes: None,
                weekday: None,
                weekend: None,
            },
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
            http_timeout_secs: 30,
            storage: Default::default(),
            ai_analysis: Default::default(),
            keywords: Vec::new(),
            notification: trendradar_config::NotificationConfig::default(),
            selection: Default::default(),
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
            decision_from_config_at(&config, context(10)),
            ScheduleDecision {
                collect: true,
                analyze: true,
                push: true,
            }
        );
        assert_eq!(
            decision_from_config_at(&config, context(20)),
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
            decision_from_config_at(&config, context(23)),
            ScheduleDecision {
                collect: true,
                analyze: false,
                push: true,
            }
        );
        assert_eq!(
            decision_from_config_at(&config, context(12)),
            ScheduleDecision {
                collect: false,
                analyze: false,
                push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn weekend_override_disables_pipeline_on_weekend() {
        let config = AppConfig {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: vec!["weibo".to_owned()],
            schedule: ScheduleConfig {
                collect: true,
                analyze: true,
                push: true,
                window: None,
                cooldown_minutes: None,
                weekday: None,
                weekend: Some(ScheduleOverrideConfig {
                    collect: Some(false),
                    analyze: Some(false),
                    push: Some(false),
                    window: None,
                }),
            },
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
            http_timeout_secs: 30,
            storage: Default::default(),
            ai_analysis: Default::default(),
            keywords: Vec::new(),
            notification: trendradar_config::NotificationConfig::default(),
            selection: Default::default(),
        };

        assert_eq!(
            decision_from_config_at(
                &config,
                ScheduleContext {
                    local_hour: 10,
                    is_weekend: true,
                    current_time: Utc::now(),
                    last_success_at: None,
                },
            ),
            ScheduleDecision {
                collect: false,
                analyze: false,
                push: false,
            }
        );
    }

    #[test]
    fn weekday_override_applies_custom_window() {
        let config = AppConfig {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: vec!["weibo".to_owned()],
            schedule: ScheduleConfig {
                collect: true,
                analyze: true,
                push: true,
                window: None,
                cooldown_minutes: None,
                weekday: Some(ScheduleOverrideConfig {
                    collect: None,
                    analyze: None,
                    push: None,
                    window: Some(ScheduleWindowConfig {
                        start_hour: 9,
                        end_hour: 18,
                    }),
                }),
                weekend: None,
            },
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
            http_timeout_secs: 30,
            storage: Default::default(),
            ai_analysis: Default::default(),
            keywords: Vec::new(),
            notification: trendradar_config::NotificationConfig::default(),
            selection: Default::default(),
        };

        assert_eq!(
            decision_from_config_at(&config, context(20)),
            ScheduleDecision {
                collect: false,
                analyze: false,
                push: false,
            }
        );
    }

    #[test]
    fn cooldown_blocks_run_within_window() -> Result<(), Box<dyn Error>> {
        let current_time = Utc
            .with_ymd_and_hms(2026, 4, 14, 10, 0, 0)
            .single()
            .ok_or("fixed current time must be valid")?;
        let last_success_at = Utc
            .with_ymd_and_hms(2026, 4, 14, 9, 30, 0)
            .single()
            .ok_or("fixed success time must be valid")?;

        let config = AppConfig {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: vec!["weibo".to_owned()],
            schedule: ScheduleConfig {
                collect: true,
                analyze: true,
                push: true,
                window: None,
                cooldown_minutes: Some(45),
                weekday: None,
                weekend: None,
            },
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
            http_timeout_secs: 30,
            storage: Default::default(),
            ai_analysis: Default::default(),
            keywords: Vec::new(),
            notification: trendradar_config::NotificationConfig::default(),
            selection: Default::default(),
        };

        assert_eq!(
            decision_from_config_at(
                &config,
                ScheduleContext {
                    local_hour: 10,
                    is_weekend: false,
                    current_time,
                    last_success_at: Some(last_success_at),
                },
            ),
            ScheduleDecision {
                collect: false,
                analyze: false,
                push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn cooldown_allows_run_after_window_expires() -> Result<(), Box<dyn Error>> {
        let current_time = Utc
            .with_ymd_and_hms(2026, 4, 14, 10, 30, 0)
            .single()
            .ok_or("fixed current time must be valid")?;
        let last_success_at = Utc
            .with_ymd_and_hms(2026, 4, 14, 9, 30, 0)
            .single()
            .ok_or("fixed success time must be valid")?;

        let config = AppConfig {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: vec!["weibo".to_owned()],
            schedule: ScheduleConfig {
                collect: true,
                analyze: false,
                push: false,
                window: None,
                cooldown_minutes: Some(45),
                weekday: None,
                weekend: None,
            },
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
            http_timeout_secs: 30,
            storage: Default::default(),
            ai_analysis: Default::default(),
            keywords: Vec::new(),
            notification: trendradar_config::NotificationConfig::default(),
            selection: Default::default(),
        };

        assert_eq!(
            decision_from_config_at(
                &config,
                ScheduleContext {
                    local_hour: 10,
                    is_weekend: false,
                    current_time,
                    last_success_at: Some(last_success_at),
                },
            ),
            ScheduleDecision {
                collect: true,
                analyze: false,
                push: false,
            }
        );
        Ok(())
    }
}

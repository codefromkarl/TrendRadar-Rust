use anyhow::Result;
use trendradar_config::load_config_from_json_str;
use trendradar_schedule::{ScheduleContext, decision_from_config_at};

use crate::common::read_system_fixture;

#[test]
fn invalid_schedule_window_is_rejected_in_system_layer() -> Result<()> {
    let fixture = read_system_fixture("schedule/invalid-window-out-of-range.json")?;
    let error = load_config_from_json_str(&fixture).expect_err("fixture should be rejected");

    assert_eq!(
        error.to_string(),
        "invalid config: schedule window hours must be between 0 and 23"
    );
    Ok(())
}

#[test]
fn equal_hour_schedule_window_is_rejected_in_system_layer() -> Result<()> {
    let fixture = read_system_fixture("schedule/invalid-window-equal-hours.json")?;
    let error = load_config_from_json_str(&fixture).expect_err("fixture should be rejected");

    assert_eq!(
        error.to_string(),
        "invalid config: schedule window start_hour and end_hour must not be equal"
    );
    Ok(())
}

#[test]
fn empty_timezone_is_rejected_in_system_layer() -> Result<()> {
    let fixture = read_system_fixture("config/invalid-empty-timezone.json")?;
    let error = load_config_from_json_str(&fixture).expect_err("fixture should be rejected");

    assert_eq!(
        error.to_string(),
        "invalid config: timezone must not be empty"
    );
    Ok(())
}

#[test]
fn daytime_schedule_window_allows_in_window_hour() -> Result<()> {
    let fixture = read_system_fixture("schedule/window-daytime.json")?;
    let config = load_config_from_json_str(&fixture)?;

    let decision = decision_from_config_at(
        &config,
        ScheduleContext {
            local_hour: 10,
            is_weekend: false,
        },
    );

    assert!(decision.collect);
    assert!(decision.analyze);
    assert!(decision.push);
    Ok(())
}

#[test]
fn overnight_schedule_window_blocks_out_of_window_hour() -> Result<()> {
    let fixture = read_system_fixture("schedule/window-overnight.json")?;
    let config = load_config_from_json_str(&fixture)?;

    let decision = decision_from_config_at(
        &config,
        ScheduleContext {
            local_hour: 12,
            is_weekend: false,
        },
    );

    assert!(!decision.collect);
    assert!(!decision.analyze);
    assert!(!decision.push);
    Ok(())
}

#[test]
fn overnight_schedule_window_allows_in_window_hour() -> Result<()> {
    let fixture = read_system_fixture("schedule/window-overnight.json")?;
    let config = load_config_from_json_str(&fixture)?;

    let decision = decision_from_config_at(
        &config,
        ScheduleContext {
            local_hour: 23,
            is_weekend: false,
        },
    );

    assert!(decision.collect);
    assert!(!decision.analyze);
    assert!(decision.push);
    Ok(())
}

#[test]
fn missing_schedule_uses_default_gate_in_system_layer() -> Result<()> {
    let fixture = read_system_fixture("config/minimal-valid-no-schedule.json")?;
    let config = load_config_from_json_str(&fixture)?;

    let decision = decision_from_config_at(
        &config,
        ScheduleContext {
            local_hour: 10,
            is_weekend: false,
        },
    );

    assert!(decision.collect);
    assert!(decision.analyze);
    assert!(decision.push);
    Ok(())
}

#[test]
fn unknown_timezone_with_window_is_rejected_in_system_layer() -> Result<()> {
    let fixture = read_system_fixture("config/invalid-unknown-timezone-window.json")?;
    let error = load_config_from_json_str(&fixture).expect_err("fixture should be rejected");

    assert_eq!(
        error.to_string(),
        "invalid config: timezone must be a valid IANA timezone"
    );
    Ok(())
}

#[test]
fn daytime_schedule_window_blocks_out_of_window_hour() -> Result<()> {
    let fixture = read_system_fixture("schedule/window-daytime.json")?;
    let config = load_config_from_json_str(&fixture)?;

    let decision = decision_from_config_at(
        &config,
        ScheduleContext {
            local_hour: 20,
            is_weekend: false,
        },
    );

    assert!(!decision.collect);
    assert!(!decision.analyze);
    assert!(!decision.push);
    Ok(())
}

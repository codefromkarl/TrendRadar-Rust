//! 调度阶段开关的系统性测试。

use std::{fs, path::PathBuf};

use chrono::TimeZone;
use trendradar_app::{FixtureSource, run_fixture_pipeline};
use trendradar_config::load_config_from_json_str;

fn system_fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/system")
        .join(relative)
}

#[test]
fn collect_only_schedule_skips_analyze_and_report() -> Result<(), Box<dyn std::error::Error>> {
    let config_fixture = fs::read_to_string(system_fixture_path("config/collect-only.json"))?;
    let config = load_config_from_json_str(&config_fixture)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 11, 0, 0)
        .single()
        .ok_or("invalid fixed timestamp")?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss("rust-blog", system_fixture_path("fetch/rss-rust-blog.json")),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;

    assert!(result.decision.collect);
    assert!(!result.decision.analyze);
    assert!(!result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert_eq!(result.stored_items.len(), 4);
    assert!(result.report_json.is_none());
    Ok(())
}

#[test]
fn disabled_schedule_produces_empty_pipeline_state() -> Result<(), Box<dyn std::error::Error>> {
    let config_fixture = fs::read_to_string(system_fixture_path("config/disabled-all.json"))?;
    let config = load_config_from_json_str(&config_fixture)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 11, 30, 0)
        .single()
        .ok_or("invalid fixed timestamp")?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss("rust-blog", system_fixture_path("fetch/rss-rust-blog.json")),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;

    assert!(!result.decision.collect);
    assert!(!result.decision.analyze);
    assert!(!result.decision.push);
    assert!(result.collected_items.is_empty());
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert!(result.stored_items.is_empty());
    assert!(result.report_json.is_none());
    Ok(())
}

#[test]
fn report_only_schedule_renders_empty_report_without_collecting()
-> Result<(), Box<dyn std::error::Error>> {
    let config_fixture = fs::read_to_string(system_fixture_path("config/report-only-empty.json"))?;
    let config = load_config_from_json_str(&config_fixture)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
        .single()
        .ok_or("invalid fixed timestamp")?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss("rust-blog", system_fixture_path("fetch/rss-rust-blog.json")),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;

    assert!(!result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert!(result.collected_items.is_empty());
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert!(result.stored_items.is_empty());

    let report = result.report_json.ok_or("report should be rendered")?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;
    assert_eq!(report_value["meta"]["item_count"], 0);
    assert_eq!(report_value["items"], serde_json::json!([]));
    Ok(())
}

#[test]
fn analyze_without_report_keeps_analysis_results() -> Result<(), Box<dyn std::error::Error>> {
    let config_fixture =
        fs::read_to_string(system_fixture_path("config/analyze-without-report.json"))?;
    let config = load_config_from_json_str(&config_fixture)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 12, 30, 0)
        .single()
        .ok_or("invalid fixed timestamp")?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss("rust-blog", system_fixture_path("fetch/rss-rust-blog.json")),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;

    assert!(result.decision.collect);
    assert!(result.decision.analyze);
    assert!(!result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert_eq!(result.ranked_items.len(), 4);
    assert_eq!(result.source_summaries.len(), 2);
    assert_eq!(result.stored_items.len(), 4);
    assert!(result.report_json.is_none());
    Ok(())
}

#[test]
fn collect_and_report_without_analyze_skips_analysis_results()
-> Result<(), Box<dyn std::error::Error>> {
    let config_fixture = fs::read_to_string(system_fixture_path(
        "config/collect-and-report-no-analyze.json",
    ))?;
    let config = load_config_from_json_str(&config_fixture)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 13, 0, 0)
        .single()
        .ok_or("invalid fixed timestamp")?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss("rust-blog", system_fixture_path("fetch/rss-rust-blog.json")),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;

    assert!(result.decision.collect);
    assert!(!result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert_eq!(result.stored_items.len(), 4);

    let report = result.report_json.ok_or("report should be rendered")?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;
    assert_eq!(report_value["meta"]["item_count"], 4);
    assert_eq!(
        report_value["items"][0]["title"],
        "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
    );
    Ok(())
}

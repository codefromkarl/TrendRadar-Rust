//! Wave 2 系统闭环测试。

use std::fs;
use std::path::PathBuf;

use chrono::TimeZone;
use trendradar_app::{FixtureSource, run_fixture_pipeline};
use trendradar_config::load_config_from_json_str;

fn system_fixture_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/system")
        .join(relative_path)
}

#[test]
fn wave2_minimal_pipeline_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    let config_fixture = fs::read_to_string(system_fixture_path("config/minimal-valid.json"))?;
    let config = load_config_from_json_str(&config_fixture)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
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
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert_eq!(result.ranked_items.len(), 4);
    assert_eq!(
        result.ranked_items[0].item.title,
        "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
    );
    assert_eq!(result.ranked_items[0].score, 100);
    assert_eq!(result.source_summaries.len(), 2);
    assert_eq!(result.source_summaries[0].source_id, "rust-blog");
    assert_eq!(result.source_summaries[0].item_count, 2);
    assert_eq!(result.source_summaries[1].source_id, "weibo");
    assert_eq!(result.stored_items.len(), 4);

    let report = result.report_json.ok_or("report should be rendered")?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;
    assert_eq!(report_value["meta"]["timezone"], "Asia/Shanghai");
    assert_eq!(report_value["meta"]["item_count"], 4);
    assert_eq!(
        report_value["items"][0]["title"],
        "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
    );
    assert_eq!(report_value["items"][1]["title"], "Rust 1.85.0 released");
    Ok(())
}

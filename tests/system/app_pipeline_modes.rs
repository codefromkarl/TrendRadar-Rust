use anyhow::Result;
use chrono::TimeZone;
use insta::assert_json_snapshot;
use trendradar_app::{FixtureSource, run_fixture_pipeline};
use trendradar_config::load_config_from_json_str;

use crate::common::read_system_fixture;

#[test]
fn report_only_schedule_renders_empty_report_in_system_layer() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/report-only-empty.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(!result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert!(result.collected_items.is_empty());
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert!(result.stored_items.is_empty());

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [],
          "meta": {
            "item_count": 0,
            "started_at": "2026-04-11T12:00:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn report_only_schedule_skips_invalid_sources_when_collect_is_disabled() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/report-only-empty.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/invalid-hotlist.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/invalid-rss.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(!result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert!(result.collected_items.is_empty());
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert!(result.stored_items.is_empty());

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [],
          "meta": {
            "item_count": 0,
            "started_at": "2026-04-11T12:00:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn collect_and_report_without_analyze_in_system_layer() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture(
        "config/collect-and-report-no-analyze.json",
    )?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 13, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(result.decision.collect);
    assert!(!result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert_eq!(result.stored_items.len(), 4);

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [
            {
              "rank": 1,
              "source_id": "rust-blog",
              "title": "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
            },
            {
              "rank": 1,
              "source_id": "weibo",
              "title": "Rust 1.85.0 released"
            },
            {
              "rank": 2,
              "source_id": "rust-blog",
              "title": "Cargo Tips Weekly (https://blog.rust-lang.org/cargo-tips-weekly)"
            },
            {
              "rank": 12,
              "source_id": "weibo",
              "title": "TrendRadar migration plan updated"
            }
          ],
          "meta": {
            "item_count": 4,
            "started_at": "2026-04-11T13:00:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn analyze_without_report_in_system_layer() -> Result<()> {
    let config =
        load_config_from_json_str(&read_system_fixture("config/analyze-without-report.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 12, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
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
fn disabled_all_schedule_keeps_system_pipeline_empty() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/disabled-all.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 11, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
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
fn minimal_valid_pipeline_renders_full_report_in_system_layer() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert_eq!(result.ranked_items.len(), 4);
    assert_eq!(result.source_summaries.len(), 2);
    assert_eq!(result.stored_items.len(), 4);

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [
            {
              "rank": 1,
              "source_id": "rust-blog",
              "title": "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
            },
            {
              "rank": 1,
              "source_id": "weibo",
              "title": "Rust 1.85.0 released"
            },
            {
              "rank": 2,
              "source_id": "rust-blog",
              "title": "Cargo Tips Weekly (https://blog.rust-lang.org/cargo-tips-weekly)"
            },
            {
              "rank": 12,
              "source_id": "weibo",
              "title": "TrendRadar migration plan updated"
            }
          ],
          "meta": {
            "item_count": 4,
            "started_at": "2026-04-11T09:30:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn minimal_valid_without_schedule_still_renders_full_report() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture(
        "config/minimal-valid-no-schedule.json",
    )?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert_eq!(result.ranked_items.len(), 4);
    assert_eq!(result.source_summaries.len(), 2);
    assert_eq!(result.stored_items.len(), 4);

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [
            {
              "rank": 1,
              "source_id": "rust-blog",
              "title": "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
            },
            {
              "rank": 1,
              "source_id": "weibo",
              "title": "Rust 1.85.0 released"
            },
            {
              "rank": 2,
              "source_id": "rust-blog",
              "title": "Cargo Tips Weekly (https://blog.rust-lang.org/cargo-tips-weekly)"
            },
            {
              "rank": 12,
              "source_id": "weibo",
              "title": "TrendRadar migration plan updated"
            }
          ],
          "meta": {
            "item_count": 4,
            "started_at": "2026-04-11T09:30:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn minimal_valid_with_empty_sources_keeps_pipeline_stable() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert!(result.collected_items.is_empty());
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert!(result.stored_items.is_empty());

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [],
          "meta": {
            "item_count": 0,
            "started_at": "2026-04-11T09:30:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn minimal_valid_with_single_rss_source_renders_partial_report() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![FixtureSource::rss(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    )];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 2);
    assert_eq!(result.ranked_items.len(), 2);
    assert_eq!(result.source_summaries.len(), 1);
    assert_eq!(result.stored_items.len(), 2);

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [
            {
              "rank": 1,
              "source_id": "rust-blog",
              "title": "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
            },
            {
              "rank": 2,
              "source_id": "rust-blog",
              "title": "Cargo Tips Weekly (https://blog.rust-lang.org/cargo-tips-weekly)"
            }
          ],
          "meta": {
            "item_count": 2,
            "started_at": "2026-04-11T09:30:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn rss_only_pipeline_does_not_require_hotlist_platforms() -> Result<()> {
    let config =
        load_config_from_json_str(&read_system_fixture("config/minimal-valid-rss-only.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![FixtureSource::rss(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    )];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(config.platforms.is_empty());
    assert!(result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 2);
    assert_eq!(result.ranked_items.len(), 2);
    assert_eq!(result.source_summaries.len(), 1);
    assert_eq!(result.source_summaries[0].source_id, "rust-blog");
    assert_eq!(result.stored_items.len(), 2);
    assert_eq!(report_value["meta"]["item_count"], 2);
    assert_eq!(report_value["items"][0]["source_id"], "rust-blog");
    Ok(())
}

#[test]
fn minimal_valid_with_single_hotlist_source_renders_partial_report() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![FixtureSource::hotlist(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
    )];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 2);
    assert_eq!(result.ranked_items.len(), 2);
    assert_eq!(result.source_summaries.len(), 1);
    assert_eq!(result.stored_items.len(), 2);

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [
            {
              "rank": 1,
              "source_id": "weibo",
              "title": "Rust 1.85.0 released"
            },
            {
              "rank": 12,
              "source_id": "weibo",
              "title": "TrendRadar migration plan updated"
            }
          ],
          "meta": {
            "item_count": 2,
            "started_at": "2026-04-11T09:30:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn collect_only_schedule_keeps_storage_without_analysis_or_report() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/collect-only.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 11, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
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
fn collect_only_schedule_still_surfaces_invalid_sources() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/collect-only.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 10, 15, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![FixtureSource::rss(
        "rust-blog",
        crate::common::system_fixture_path("fetch/invalid-rss.json"),
    )];

    let error = run_fixture_pipeline(&config, started_at, &sources)
        .expect_err("invalid fixture should still fail when collect=true");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-rss.json"));
    Ok(())
}

#[test]
fn analyze_only_schedule_keeps_empty_analysis_outputs() -> Result<()> {
    let config =
        load_config_from_json_str(&read_system_fixture("config/analyze-only-empty.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 11, 15, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;

    assert!(!result.decision.collect);
    assert!(result.decision.analyze);
    assert!(!result.decision.push);
    assert!(result.collected_items.is_empty());
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert!(result.stored_items.is_empty());
    assert!(result.report_json.is_none());
    Ok(())
}

#[test]
fn push_only_schedule_renders_empty_report_without_collect_or_analyze() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/push-only-empty.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 11, 45, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(!result.decision.collect);
    assert!(!result.decision.analyze);
    assert!(result.decision.push);
    assert!(result.collected_items.is_empty());
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert!(result.stored_items.is_empty());

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [],
          "meta": {
            "item_count": 0,
            "started_at": "2026-04-11T11:45:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn pipeline_surfaces_fetch_fixture_parse_errors() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 14, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![FixtureSource::rss(
        "broken-rss",
        crate::common::system_fixture_path("fetch/invalid-rss.json"),
    )];

    let error = run_fixture_pipeline(&config, started_at, &sources)
        .expect_err("pipeline should surface fixture parse failure");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-rss.json"));
    Ok(())
}

#[test]
fn pipeline_surfaces_hotlist_fixture_parse_errors() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 14, 5, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![FixtureSource::hotlist(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/invalid-hotlist.json"),
    )];

    let error = run_fixture_pipeline(&config, started_at, &sources)
        .expect_err("pipeline should surface fixture parse failure");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-hotlist.json"));
    Ok(())
}

#[test]
fn windowed_schedule_blocks_pipeline_outside_local_hour() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("schedule/window-daytime.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 13, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
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
fn windowed_schedule_skips_invalid_sources_when_blocked() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("schedule/window-daytime.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/invalid-hotlist.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/invalid-rss.json"),
        ),
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
fn windowed_schedule_allows_pipeline_inside_local_hour() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("schedule/window-daytime.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 2, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;

    assert!(result.decision.collect);
    assert!(result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert_eq!(result.ranked_items.len(), 4);
    assert_eq!(result.source_summaries.len(), 2);
    assert_eq!(result.stored_items.len(), 4);
    assert!(result.report_json.is_some());
    Ok(())
}

#[test]
fn windowed_schedule_allows_invalid_sources_to_surface() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("schedule/window-daytime.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 2, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![FixtureSource::rss(
        "rust-blog",
        crate::common::system_fixture_path("fetch/invalid-rss.json"),
    )];

    let error = run_fixture_pipeline(&config, started_at, &sources)
        .expect_err("invalid fixture should fail when window allows collect");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-rss.json"));
    Ok(())
}

#[test]
fn overnight_window_allows_pipeline_inside_local_hour() -> Result<()> {
    let config =
        load_config_from_json_str(&read_system_fixture("schedule/window-overnight.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 15, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
    ];

    let result = run_fixture_pipeline(&config, started_at, &sources)?;
    let report = result
        .report_json
        .ok_or_else(|| anyhow::anyhow!("report should be rendered"))?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert!(result.decision.collect);
    assert!(!result.decision.analyze);
    assert!(result.decision.push);
    assert_eq!(result.collected_items.len(), 4);
    assert!(result.ranked_items.is_empty());
    assert!(result.source_summaries.is_empty());
    assert_eq!(result.stored_items.len(), 4);

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [
            {
              "rank": 1,
              "source_id": "rust-blog",
              "title": "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
            },
            {
              "rank": 1,
              "source_id": "weibo",
              "title": "Rust 1.85.0 released"
            },
            {
              "rank": 2,
              "source_id": "rust-blog",
              "title": "Cargo Tips Weekly (https://blog.rust-lang.org/cargo-tips-weekly)"
            },
            {
              "rank": 12,
              "source_id": "weibo",
              "title": "TrendRadar migration plan updated"
            }
          ],
          "meta": {
            "item_count": 4,
            "started_at": "2026-04-11T15:00:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    Ok(())
}

#[test]
fn overnight_window_blocks_pipeline_outside_local_hour() -> Result<()> {
    let config =
        load_config_from_json_str(&read_system_fixture("schedule/window-overnight.json")?)?;
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 4, 0, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
    let sources = vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss(
            "rust-blog",
            crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
        ),
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

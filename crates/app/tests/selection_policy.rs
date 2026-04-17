#![allow(clippy::expect_used, missing_docs)]

use std::fs;
use std::path::PathBuf;

use chrono::TimeZone;
use trendradar_app::{FixtureSource, OutputMode, run_fixture_pipeline_with_output};
use trendradar_config::load_config_from_json_str;

fn system_fixture_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/system")
        .join(relative_path)
}

fn fixture_config() -> trendradar_config::AppConfig {
    let config_fixture = fs::read_to_string(system_fixture_path("config/minimal-valid.json"))
        .expect("fixture config must exist");
    load_config_from_json_str(&config_fixture).expect("fixture config must parse")
}

fn fixture_sources(config: &trendradar_config::AppConfig) -> Vec<FixtureSource> {
    vec![
        FixtureSource::hotlist(
            config.platforms[0].clone(),
            system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss("rust-blog", system_fixture_path("fetch/rss-rust-blog.json")),
    ]
}

fn fixed_started_at() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc
        .with_ymd_and_hms(2026, 4, 17, 9, 30, 0)
        .single()
        .expect("fixed timestamp must be valid")
}

#[test]
fn high_rank_fallback_keeps_top_items_without_keyword_match() {
    let mut config = fixture_config();
    config.keywords = vec!["openai".to_owned()];
    config.selection.high_rank_fallback_max_rank = Some(1);
    let result = run_fixture_pipeline_with_output(
        &config,
        fixed_started_at(),
        &fixture_sources(&config),
        OutputMode::Json,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.filtered_items.len(), 2);
    assert!(
        result
            .filtered_items
            .iter()
            .any(|item| item.source_id == "weibo" && item.rank == 1)
    );
    assert!(
        result
            .filtered_items
            .iter()
            .any(|item| item.source_id == "rust-blog" && item.rank == 1)
    );
}

#[test]
fn min_items_per_source_keeps_source_diversity() {
    let mut config = fixture_config();
    config.keywords = vec!["openai".to_owned()];
    config.selection.min_items_per_source = Some(1);
    let result = run_fixture_pipeline_with_output(
        &config,
        fixed_started_at(),
        &fixture_sources(&config),
        OutputMode::Json,
    )
    .expect("pipeline should succeed");

    assert_eq!(result.filtered_items.len(), 2);
    assert!(
        result
            .filtered_items
            .iter()
            .any(|item| item.source_id == "weibo")
    );
    assert!(
        result
            .filtered_items
            .iter()
            .any(|item| item.source_id == "rust-blog")
    );
}

#[test]
fn min_items_per_domain_keeps_multiple_domains() {
    let mut config = fixture_config();
    config.keywords = vec!["openai".to_owned()];
    config.selection.min_items_per_domain = Some(2);

    let result = run_fixture_pipeline_with_output(
        &config,
        fixed_started_at(),
        &fixture_sources(&config),
        OutputMode::Json,
    )
    .expect("pipeline should succeed");

    assert!(result.filtered_items.len() >= 3);
    assert!(result.domain_summaries.len() >= 2);
}

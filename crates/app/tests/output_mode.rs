//! Output-mode integration tests for the app pipeline.

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

fn fixture_config() -> Result<trendradar_config::AppConfig, Box<dyn std::error::Error>> {
    let config_fixture = fs::read_to_string(system_fixture_path("config/minimal-valid.json"))?;
    Ok(load_config_from_json_str(&config_fixture)?)
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

#[test]
fn fixture_pipeline_renders_only_requested_reports() -> Result<(), Box<dyn std::error::Error>> {
    let config = fixture_config()?;
    let sources = fixture_sources(&config);
    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
        .single()
        .ok_or("invalid fixed timestamp")?;

    let json_result =
        run_fixture_pipeline_with_output(&config, started_at, &sources, OutputMode::Json)?;
    assert!(json_result.report_json.is_some());
    assert!(json_result.report_html.is_none());
    assert!(json_result.report_table.is_none());
    assert!(json_result.report_markdown.is_none());

    let html_result =
        run_fixture_pipeline_with_output(&config, started_at, &sources, OutputMode::Html)?;
    assert!(html_result.report_json.is_none());
    assert!(html_result.report_html.is_some());
    assert!(html_result.report_table.is_none());
    assert!(html_result.report_markdown.is_none());

    let both_result =
        run_fixture_pipeline_with_output(&config, started_at, &sources, OutputMode::Both)?;
    assert!(both_result.report_json.is_some());
    assert!(both_result.report_html.is_some());
    assert!(both_result.report_table.is_none());
    assert!(both_result.report_markdown.is_none());

    let table_result =
        run_fixture_pipeline_with_output(&config, started_at, &sources, OutputMode::Table)?;
    assert!(table_result.report_json.is_none());
    assert!(table_result.report_html.is_none());
    assert!(table_result.report_table.is_some());
    assert!(table_result.report_markdown.is_none());

    let markdown_result =
        run_fixture_pipeline_with_output(&config, started_at, &sources, OutputMode::Markdown)?;
    assert!(markdown_result.report_json.is_none());
    assert!(markdown_result.report_html.is_none());
    assert!(markdown_result.report_table.is_none());
    assert!(markdown_result.report_markdown.is_some());

    Ok(())
}

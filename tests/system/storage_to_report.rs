use anyhow::Result;
use chrono::TimeZone;
use insta::assert_json_snapshot;
use trendradar_domain::{NewsItem, RunContext};
use trendradar_report::render_news_json;
use trendradar_storage::{NewsRepository, SqliteNewsRepository};

use crate::common::load_json_fixture;

#[test]
fn storage_roundtrip_can_feed_report_snapshot() -> Result<()> {
    let items: Vec<NewsItem> = load_json_fixture("storage/news-roundtrip-input.json")?;
    let mut repository = SqliteNewsRepository::in_memory()?;

    for item in items {
        repository.save_news(item)?;
    }

    let stored = repository.list_news()?;
    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };

    let report = render_news_json(&stored, &context)?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert_json_snapshot!(
        report_value,
        @r#"
        {
          "items": [
            {
              "rank": 1,
              "source_id": "github-trending",
              "title": "Rust 1.85.0 released"
            },
            {
              "rank": 12,
              "source_id": "community-hotlist",
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

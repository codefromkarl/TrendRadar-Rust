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

#[test]
fn empty_storage_can_feed_empty_report_snapshot() -> Result<()> {
    let repository = SqliteNewsRepository::in_memory()?;
    let stored = repository.list_news()?;
    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 11, 10, 0, 0)
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
          "items": [],
          "meta": {
            "item_count": 0,
            "started_at": "2026-04-11T10:00:00Z",
            "timezone": "Asia/Shanghai"
          }
        }
        "#
    );
    let items = report_value["items"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("items should be an array"))?;
    assert!(items.is_empty());
    Ok(())
}

#[test]
fn duplicate_storage_entries_keep_best_rank_before_report() -> Result<()> {
    let mut repository = SqliteNewsRepository::in_memory()?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "github-trending".to_owned(),
        rank: 5,
    })?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "github-trending".to_owned(),
        rank: 2,
    })?;

    let stored = repository.list_news()?;
    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 11, 10, 30, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };

    let report = render_news_json(&stored, &context)?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert_eq!(report_value["meta"]["item_count"], 1);
    assert_eq!(report_value["items"][0]["rank"], 2);
    assert_eq!(report_value["items"][0]["title"], "Rust 1.85.0 released");
    Ok(())
}

#[test]
fn duplicate_storage_entries_with_same_rank_still_render_once() -> Result<()> {
    let mut repository = SqliteNewsRepository::in_memory()?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "github-trending".to_owned(),
        rank: 2,
    })?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "github-trending".to_owned(),
        rank: 2,
    })?;

    let stored = repository.list_news()?;
    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 11, 10, 5, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };

    let report = render_news_json(&stored, &context)?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;
    let items = report_value["items"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("items should be an array"))?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Rust 1.85.0 released");
    assert_eq!(items[0]["rank"], 2);
    Ok(())
}

#[test]
fn out_of_order_storage_writes_render_sorted_report() -> Result<()> {
    let mut repository = SqliteNewsRepository::in_memory()?;
    repository.save_news(NewsItem {
        title: "TrendRadar migration plan updated".to_owned(),
        source_id: "community-hotlist".to_owned(),
        rank: 12,
    })?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "github-trending".to_owned(),
        rank: 1,
    })?;

    let stored = repository.list_news()?;
    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 11, 10, 45, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };

    let report = render_news_json(&stored, &context)?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert_eq!(report_value["items"][0]["title"], "Rust 1.85.0 released");
    assert_eq!(
        report_value["items"][1]["title"],
        "TrendRadar migration plan updated"
    );
    Ok(())
}

#[test]
fn same_title_from_different_sources_remains_separate_in_report() -> Result<()> {
    let mut repository = SqliteNewsRepository::in_memory()?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "github-trending".to_owned(),
        rank: 1,
    })?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "rust-blog".to_owned(),
        rank: 2,
    })?;

    let stored = repository.list_news()?;
    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 11, 10, 50, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };

    let report = render_news_json(&stored, &context)?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert_eq!(report_value["meta"]["item_count"], 2);
    assert_eq!(report_value["items"][0]["source_id"], "github-trending");
    assert_eq!(report_value["items"][1]["source_id"], "rust-blog");
    Ok(())
}

#[test]
fn same_title_and_rank_from_different_sources_still_render_separately() -> Result<()> {
    let mut repository = SqliteNewsRepository::in_memory()?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "github-trending".to_owned(),
        rank: 1,
    })?;
    repository.save_news(NewsItem {
        title: "Rust 1.85.0 released".to_owned(),
        source_id: "rust-blog".to_owned(),
        rank: 1,
    })?;

    let stored = repository.list_news()?;
    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 11, 11, 20, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };

    let report = render_news_json(&stored, &context)?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;
    let items = report_value["items"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("items should be an array"))?;

    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["source_id"], "github-trending");
    assert_eq!(items[1]["source_id"], "rust-blog");
    assert_eq!(items[0]["rank"], 1);
    assert_eq!(items[1]["rank"], 1);
    Ok(())
}

#[test]
fn same_rank_items_render_in_source_then_title_order() -> Result<()> {
    let mut repository = SqliteNewsRepository::in_memory()?;
    repository.save_news(NewsItem {
        title: "Zeta release notes".to_owned(),
        source_id: "z-source".to_owned(),
        rank: 3,
    })?;
    repository.save_news(NewsItem {
        title: "Alpha release notes".to_owned(),
        source_id: "a-source".to_owned(),
        rank: 3,
    })?;
    repository.save_news(NewsItem {
        title: "Beta release notes".to_owned(),
        source_id: "a-source".to_owned(),
        rank: 3,
    })?;

    let stored = repository.list_news()?;
    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 11, 11, 0, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };

    let report = render_news_json(&stored, &context)?;
    let report_value: serde_json::Value = serde_json::from_str(&report)?;

    assert_eq!(report_value["items"][0]["source_id"], "a-source");
    assert_eq!(report_value["items"][0]["title"], "Alpha release notes");
    assert_eq!(report_value["items"][1]["source_id"], "a-source");
    assert_eq!(report_value["items"][1]["title"], "Beta release notes");
    assert_eq!(report_value["items"][2]["source_id"], "z-source");
    assert_eq!(report_value["items"][2]["title"], "Zeta release notes");
    Ok(())
}

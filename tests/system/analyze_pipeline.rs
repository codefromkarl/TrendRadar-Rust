use anyhow::Result;
use trendradar_analyze::{group_news_by_source, rank_news};
use trendradar_config::load_config_from_json_str;
use trendradar_domain::NewsItem;
use trendradar_schedule::decision_from_config;

use crate::common::{load_json_fixture, read_system_fixture};

#[test]
fn analyze_pipeline_uses_schedule_gate_and_produces_stable_results() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let decision = decision_from_config(&config);
    let items: Vec<NewsItem> = load_json_fixture("analyze/source-groups-input.json")?;

    assert!(decision.analyze);

    let ranked = rank_news(&items);
    let grouped = group_news_by_source(&items);

    assert_eq!(ranked[0].item.title, "Rust 1.85.0 released");
    assert_eq!(ranked[0].score, 100);
    assert_eq!(grouped[0].source_id, "github-trending");
    assert_eq!(grouped[0].item_count, 2);
    Ok(())
}

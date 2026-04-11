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

#[test]
fn analyze_pipeline_keeps_same_rank_items_in_stable_title_order() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let decision = decision_from_config(&config);
    let items: Vec<NewsItem> = load_json_fixture("analyze/same-rank-input.json")?;

    assert!(decision.analyze);

    let ranked = rank_news(&items);
    let ranked_titles: Vec<&str> = ranked
        .iter()
        .map(|entry| entry.item.title.as_str())
        .collect();

    assert_eq!(
        ranked_titles,
        vec![
            "Alpha release note",
            "Beta release note",
            "Zeta release note"
        ]
    );
    Ok(())
}

#[test]
fn analyze_pipeline_respects_disabled_analyze_gate() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/analyze-disabled.json")?)?;
    let decision = decision_from_config(&config);
    let items: Vec<NewsItem> = load_json_fixture("analyze/source-groups-input.json")?;

    assert!(!decision.analyze);

    let ranked = if decision.analyze {
        rank_news(&items)
    } else {
        Vec::new()
    };
    let grouped = if decision.analyze {
        group_news_by_source(&items)
    } else {
        Vec::new()
    };

    assert!(ranked.is_empty());
    assert!(grouped.is_empty());
    Ok(())
}

#[test]
fn analyze_pipeline_clamps_zero_rank_to_top_score() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let decision = decision_from_config(&config);
    let items: Vec<NewsItem> = load_json_fixture("analyze/zero-rank-input.json")?;

    assert!(decision.analyze);

    let ranked = rank_news(&items);

    assert_eq!(ranked[0].score, 100);
    assert_eq!(ranked[1].score, 100);
    Ok(())
}

#[test]
fn analyze_pipeline_returns_empty_outputs_for_empty_input() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let decision = decision_from_config(&config);
    let items: Vec<NewsItem> = load_json_fixture("analyze/empty-input.json")?;

    assert!(decision.analyze);

    let ranked = rank_news(&items);
    let grouped = group_news_by_source(&items);

    assert!(ranked.is_empty());
    assert!(grouped.is_empty());
    Ok(())
}

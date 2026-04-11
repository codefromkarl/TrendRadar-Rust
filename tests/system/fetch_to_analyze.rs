use anyhow::Result;
use trendradar_analyze::{group_news_by_source, rank_news};
use trendradar_config::load_config_from_json_str;
use trendradar_fetch::{Fetcher, FixtureHotlistFetcher, FixtureRssFetcher};

use crate::common::read_system_fixture;

#[test]
fn fetch_outputs_can_feed_analyze_pipeline() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    );

    let mut items = hotlist_fetcher.fetch()?;
    items.extend(rss_fetcher.fetch()?);

    let ranked = rank_news(&items);
    let grouped = group_news_by_source(&items);

    assert_eq!(ranked.len(), 4);
    assert_eq!(
        ranked[0].item.title,
        "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
    );
    assert_eq!(ranked[0].score, 100);
    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped[0].source_id, "rust-blog");
    assert_eq!(grouped[0].item_count, 2);
    assert_eq!(grouped[1].source_id, "weibo");
    assert_eq!(grouped[1].item_count, 2);
    Ok(())
}

#[test]
fn empty_fetch_outputs_keep_analyze_results_empty() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/empty-hotlist.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "empty-rss",
        crate::common::system_fixture_path("fetch/empty-rss.json"),
    );

    let mut items = hotlist_fetcher.fetch()?;
    items.extend(rss_fetcher.fetch()?);

    let ranked = rank_news(&items);
    let grouped = group_news_by_source(&items);

    assert!(ranked.is_empty());
    assert!(grouped.is_empty());
    Ok(())
}

#[test]
fn invalid_fetch_input_stops_fetch_to_analyze_chain() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "broken-rss",
        crate::common::system_fixture_path("fetch/invalid-rss.json"),
    );

    let items = hotlist_fetcher.fetch()?;
    let error = rss_fetcher
        .fetch()
        .expect_err("fixture should fail to parse");

    assert_eq!(items.len(), 2);
    let message = error.to_string();
    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-rss.json"));
    Ok(())
}

#[test]
fn partially_fetched_items_do_not_reach_analyze_when_later_source_fails() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "broken-rss",
        crate::common::system_fixture_path("fetch/invalid-rss.json"),
    );

    let run_chain = || -> anyhow::Result<(usize, usize)> {
        let mut items = hotlist_fetcher.fetch()?;
        items.extend(rss_fetcher.fetch()?);
        let ranked = rank_news(&items);
        let grouped = group_news_by_source(&items);
        Ok((ranked.len(), grouped.len()))
    };

    let error = run_chain().expect_err("later invalid source should abort the chain");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-rss.json"));
    Ok(())
}

#[test]
fn partially_fetched_rss_items_do_not_reach_analyze_when_later_hotlist_fails() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let rss_fetcher = FixtureRssFetcher::new(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    );
    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/invalid-hotlist.json"),
    );

    let run_chain = || -> anyhow::Result<(usize, usize)> {
        let mut items = rss_fetcher.fetch()?;
        items.extend(hotlist_fetcher.fetch()?);
        let ranked = rank_news(&items);
        let grouped = group_news_by_source(&items);
        Ok((ranked.len(), grouped.len()))
    };

    let error = run_chain().expect_err("later invalid source should abort the chain");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-hotlist.json"));
    Ok(())
}

#[test]
fn invalid_hotlist_input_stops_fetch_to_analyze_chain() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/invalid-hotlist.json"),
    );

    let error = hotlist_fetcher
        .fetch()
        .expect_err("fixture should fail to parse");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-hotlist.json"));
    Ok(())
}

#[test]
fn mixed_empty_and_non_empty_sources_still_analyze_available_items() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/empty-hotlist.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    );

    let mut items = hotlist_fetcher.fetch()?;
    items.extend(rss_fetcher.fetch()?);

    let ranked = rank_news(&items);
    let grouped = group_news_by_source(&items);

    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].item.source_id, "rust-blog");
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped[0].source_id, "rust-blog");
    assert_eq!(grouped[0].item_count, 2);
    Ok(())
}

#[test]
fn fetched_same_rank_items_keep_stable_title_order() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    );

    let mut items = hotlist_fetcher.fetch()?;
    items.extend(rss_fetcher.fetch()?);

    let ranked = rank_news(&items);

    assert_eq!(
        ranked[0].item.title,
        "Async Rust Patterns (https://blog.rust-lang.org/async-patterns)"
    );
    assert_eq!(ranked[1].item.title, "Rust 1.85.0 released");
    assert_eq!(ranked[0].score, ranked[1].score);
    Ok(())
}

#[test]
fn grouped_sources_prefer_better_best_rank_when_counts_tie() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/hotlist-low-ranks.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    );

    let mut items = hotlist_fetcher.fetch()?;
    items.extend(rss_fetcher.fetch()?);

    let grouped = group_news_by_source(&items);

    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped[0].source_id, "rust-blog");
    assert_eq!(grouped[0].item_count, 2);
    assert_eq!(grouped[0].best_rank, 1);
    assert_eq!(grouped[1].source_id, "weibo");
    assert_eq!(grouped[1].item_count, 2);
    assert_eq!(grouped[1].best_rank, 3);
    Ok(())
}

#[test]
fn grouped_sources_prefer_higher_item_count_over_best_rank() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/hotlist-three-items.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    );

    let mut items = hotlist_fetcher.fetch()?;
    items.extend(rss_fetcher.fetch()?);

    let grouped = group_news_by_source(&items);

    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped[0].source_id, "weibo");
    assert_eq!(grouped[0].item_count, 3);
    assert_eq!(grouped[0].best_rank, 5);
    assert_eq!(grouped[1].source_id, "rust-blog");
    assert_eq!(grouped[1].item_count, 2);
    assert_eq!(grouped[1].best_rank, 1);
    Ok(())
}

use anyhow::Result;
use trendradar_config::load_config_from_json_str;
use trendradar_fetch::{Fetcher, FixtureHotlistFetcher, FixtureRssFetcher};

use crate::common::read_system_fixture;

#[test]
fn hotlist_and_rss_fixtures_normalize_into_news_items() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/hotlist-weibo.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    );

    let hotlist_items = hotlist_fetcher.fetch()?;
    let rss_items = rss_fetcher.fetch()?;

    assert_eq!(hotlist_items.len(), 2);
    assert_eq!(hotlist_items[0].source_id, "weibo");
    assert_eq!(hotlist_items[0].rank, 1);

    assert_eq!(rss_items.len(), 2);
    assert_eq!(rss_items[0].source_id, "rust-blog");
    assert_eq!(rss_items[0].rank, 1);
    assert!(rss_items[0].title.contains("Async Rust Patterns"));
    Ok(())
}

#[test]
fn empty_rss_fixture_normalizes_into_empty_news_items() -> Result<()> {
    let rss_fetcher = FixtureRssFetcher::new(
        "empty-rss",
        crate::common::system_fixture_path("fetch/empty-rss.json"),
    );

    let rss_items = rss_fetcher.fetch()?;

    assert!(rss_items.is_empty());
    Ok(())
}

#[test]
fn invalid_rss_fixture_reports_parse_error() -> Result<()> {
    let rss_fetcher = FixtureRssFetcher::new(
        "broken-rss",
        crate::common::system_fixture_path("fetch/invalid-rss.json"),
    );

    let error = rss_fetcher
        .fetch()
        .expect_err("fixture should fail to parse");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-rss.json"));
    Ok(())
}

#[test]
fn empty_hotlist_fixture_normalizes_into_empty_news_items() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;
    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/empty-hotlist.json"),
    );

    let hotlist_items = hotlist_fetcher.fetch()?;

    assert!(hotlist_items.is_empty());
    Ok(())
}

#[test]
fn invalid_hotlist_fixture_reports_parse_error() -> Result<()> {
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
fn partially_fetched_rss_items_do_not_survive_when_later_hotlist_fails() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let rss_fetcher = FixtureRssFetcher::new(
        "rust-blog",
        crate::common::system_fixture_path("fetch/rss-rust-blog.json"),
    );
    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/invalid-hotlist.json"),
    );

    let run_chain = || -> anyhow::Result<usize> {
        let mut items = rss_fetcher.fetch()?;
        items.extend(hotlist_fetcher.fetch()?);
        Ok(items.len())
    };

    let error = run_chain().expect_err("later invalid source should abort the chain");
    let message = error.to_string();

    assert!(message.contains("failed to parse fetch fixture"));
    assert!(message.contains("invalid-hotlist.json"));
    Ok(())
}

#[test]
fn empty_hotlist_and_rss_fixtures_stay_empty_together() -> Result<()> {
    let config = load_config_from_json_str(&read_system_fixture("config/minimal-valid.json")?)?;

    let hotlist_fetcher = FixtureHotlistFetcher::new(
        config.platforms[0].clone(),
        crate::common::system_fixture_path("fetch/empty-hotlist.json"),
    );
    let rss_fetcher = FixtureRssFetcher::new(
        "empty-rss",
        crate::common::system_fixture_path("fetch/empty-rss.json"),
    );

    let hotlist_items = hotlist_fetcher.fetch()?;
    let rss_items = rss_fetcher.fetch()?;

    assert!(hotlist_items.is_empty());
    assert!(rss_items.is_empty());
    Ok(())
}

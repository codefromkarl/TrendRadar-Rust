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

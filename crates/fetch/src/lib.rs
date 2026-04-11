//! 抓取层骨架。

use serde::Deserialize;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use thiserror::Error;
use trendradar_domain::NewsItem;

/// 抓取层结果类型。
pub type Result<T> = std::result::Result<T, FetchError>;

/// 抓取错误。
#[derive(Debug, Error)]
pub enum FetchError {
    /// 读取 fixture 失败。
    #[error("failed to read fetch fixture {path}: {message}")]
    ReadFixture {
        /// fixture 路径。
        path: String,
        /// 具体原因。
        message: String,
    },
    /// 解析 fixture 失败。
    #[error("failed to parse fetch fixture {path}: {message}")]
    ParseFixture {
        /// fixture 路径。
        path: String,
        /// 具体原因。
        message: String,
    },
}

/// 抓取器接口。
pub trait Fetcher {
    /// 拉取一批新闻条目。
    fn fetch(&self) -> Result<Vec<NewsItem>>;
}

#[derive(Debug, Deserialize)]
struct HotlistFixtureItem {
    title: String,
    rank: u32,
}

#[derive(Debug, Deserialize)]
struct RssFixtureItem {
    title: String,
    url: String,
}

/// 基于 fixture 的热榜抓取器。
pub struct FixtureHotlistFetcher {
    platform_id: String,
    fixture_path: PathBuf,
}

impl FixtureHotlistFetcher {
    /// 创建一个热榜抓取器。
    #[must_use]
    pub fn new(platform_id: String, fixture_path: impl Into<PathBuf>) -> Self {
        Self {
            platform_id,
            fixture_path: fixture_path.into(),
        }
    }
}

impl Fetcher for FixtureHotlistFetcher {
    fn fetch(&self) -> Result<Vec<NewsItem>> {
        let items: Vec<HotlistFixtureItem> = load_fixture(&self.fixture_path)?;
        Ok(items
            .into_iter()
            .map(|item| NewsItem {
                title: item.title,
                source_id: self.platform_id.clone(),
                rank: item.rank,
            })
            .collect())
    }
}

/// 基于 fixture 的 RSS 抓取器。
pub struct FixtureRssFetcher {
    source_id: String,
    fixture_path: PathBuf,
}

impl FixtureRssFetcher {
    /// 创建一个 RSS 抓取器。
    #[must_use]
    pub fn new(source_id: impl Into<String>, fixture_path: impl Into<PathBuf>) -> Self {
        Self {
            source_id: source_id.into(),
            fixture_path: fixture_path.into(),
        }
    }
}

impl Fetcher for FixtureRssFetcher {
    fn fetch(&self) -> Result<Vec<NewsItem>> {
        let items: Vec<RssFixtureItem> = load_fixture(&self.fixture_path)?;
        Ok(items
            .into_iter()
            .enumerate()
            .map(|(index, item)| NewsItem {
                title: format!("{} ({})", item.title, item.url),
                source_id: self.source_id.clone(),
                rank: (index + 1) as u32,
            })
            .collect())
    }
}

fn load_fixture<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = read_to_string(path).map_err(|error| FetchError::ReadFixture {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    serde_json::from_str(&contents).map_err(|error| FetchError::ParseFixture {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{Fetcher, FixtureHotlistFetcher, FixtureRssFetcher};
    use std::error::Error;
    use std::fs::read_to_string;
    use trendradar_config::load_config_from_json_str;

    #[test]
    fn hotlist_fetcher_normalizes_platform_fixture() -> Result<(), Box<dyn Error>> {
        let config_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/config/minimal-valid.json"
        );
        let config = load_config_from_json_str(&read_to_string(config_path)?)?;
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/fetch/hotlist-weibo.json"
        );
        let fetcher = FixtureHotlistFetcher::new(config.platforms[0].clone(), fixture_path);

        let items = fetcher.fetch()?;

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].source_id, "weibo");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].title, "TrendRadar migration plan updated");
        Ok(())
    }

    #[test]
    fn rss_fetcher_normalizes_entries_into_news_items() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/fetch/rss-rust-blog.json"
        );
        let fetcher = FixtureRssFetcher::new("rust-blog", fixture_path);

        let items = fetcher.fetch()?;

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].source_id, "rust-blog");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].rank, 2);
        Ok(())
    }

    #[test]
    fn rss_fetcher_reports_parse_fixture_error() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/fetch/invalid-rss.json"
        );
        let fetcher = FixtureRssFetcher::new("broken-rss", fixture_path);

        let error = fetcher.fetch().expect_err("fixture should fail to parse");

        let message = error.to_string();
        assert!(message.contains("failed to parse fetch fixture"));
        assert!(message.contains("invalid-rss.json"));
        Ok(())
    }

    #[test]
    fn rss_fetcher_returns_empty_items_for_empty_fixture() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/fetch/empty-rss.json"
        );
        let fetcher = FixtureRssFetcher::new("empty-rss", fixture_path);

        let items = fetcher.fetch()?;

        assert!(items.is_empty());
        Ok(())
    }
}

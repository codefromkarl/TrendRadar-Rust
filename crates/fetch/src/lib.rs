//! 抓取层：fixture adapter 与 HTTP adapter。

use serde::Deserialize;
use std::fs::read_to_string;
use std::io::Cursor;
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
    /// 网络请求失败（连接、超时、DNS 等）。
    #[error("network error fetching {url}: {message}")]
    Network {
        /// 请求地址。
        url: String,
        /// 具体原因。
        message: String,
    },
    /// HTTP 响应状态异常（4xx / 5xx）。
    #[error("http {status} from {url}: {message}")]
    Http {
        /// 请求地址。
        url: String,
        /// HTTP 状态码。
        status: u16,
        /// 具体原因。
        message: String,
    },
    /// 解析远程内容失败（RSS XML / JSON）。
    #[error("failed to parse response from {url}: {message}")]
    ParseResponse {
        /// 请求地址。
        url: String,
        /// 具体原因。
        message: String,
    },
}

/// 抓取器接口。
pub trait Fetcher {
    /// 拉取一批新闻条目。
    fn fetch(&self) -> Result<Vec<NewsItem>>;
}

// ---------------------------------------------------------------------------
// Fixture adapters（已有）
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// HTTP adapters
// ---------------------------------------------------------------------------

/// 基于 HTTP 的 RSS 抓取器。
///
/// 从远程 RSS feed 拉取条目，解析 XML 并归一化为 `NewsItem`。
pub struct HttpRssFetcher {
    source_id: String,
    url: String,
    client: reqwest::blocking::Client,
}

impl HttpRssFetcher {
    /// 创建 HTTP RSS 抓取器。
    #[must_use]
    pub fn new(source_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            url: url.into(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Fetcher for HttpRssFetcher {
    fn fetch(&self) -> Result<Vec<NewsItem>> {
        let body = http_get_text(&self.client, &self.url)?;

        let channel = rss::Channel::read_from(Cursor::new(body.as_bytes())).map_err(|error| {
            FetchError::ParseResponse {
                url: self.url.clone(),
                message: error.to_string(),
            }
        })?;

        let items = channel
            .items()
            .iter()
            .filter_map(|item| item.title().map(|t| t.to_owned()))
            .enumerate()
            .map(|(index, title)| NewsItem {
                title,
                source_id: self.source_id.clone(),
                rank: (index + 1) as u32,
            })
            .collect();

        Ok(items)
    }
}

/// 基于 HTTP 的热榜抓取器。
///
/// 从远程 JSON API 拉取热榜数据，解析为 `NewsItem`。
pub struct HttpHotlistFetcher {
    platform_id: String,
    url: String,
    client: reqwest::blocking::Client,
}

impl HttpHotlistFetcher {
    /// 创建 HTTP 热榜抓取器。
    #[must_use]
    pub fn new(platform_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            platform_id: platform_id.into(),
            url: url.into(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Fetcher for HttpHotlistFetcher {
    fn fetch(&self) -> Result<Vec<NewsItem>> {
        let body = http_get_text(&self.client, &self.url)?;

        let items: Vec<HotlistFixtureItem> =
            serde_json::from_str(&body).map_err(|error| FetchError::ParseResponse {
                url: self.url.clone(),
                message: error.to_string(),
            })?;

        let items = items
            .into_iter()
            .map(|item| NewsItem {
                title: item.title,
                source_id: self.platform_id.clone(),
                rank: item.rank,
            })
            .collect();

        Ok(items)
    }
}

/// 发起 HTTP GET 请求并返回响应体文本。
///
/// 统一处理网络错误和 HTTP 状态错误。
fn http_get_text(client: &reqwest::blocking::Client, url: &str) -> Result<String> {
    let response = client
        .get(url)
        .send()
        .map_err(|error| FetchError::Network {
            url: url.to_owned(),
            message: error.to_string(),
        })?;

    let status = response.status().as_u16();
    if !(200..300).contains(&(status)) {
        return Err(FetchError::Http {
            url: url.to_owned(),
            status,
            message: format!("unexpected status code: {status}"),
        });
    }

    response.text().map_err(|error| FetchError::Network {
        url: url.to_owned(),
        message: error.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        Fetcher, FixtureHotlistFetcher, FixtureRssFetcher, HttpHotlistFetcher, HttpRssFetcher,
    };
    use std::error::Error;
    use std::fs::read_to_string;
    use trendradar_config::load_config_from_json_str;

    // -- Fixture adapter tests (已有) --

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

    // -- HTTP adapter tests --

    #[test]
    fn http_rss_fetcher_parses_valid_feed() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        let feed_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item><title>Rust 1.85 released</title></item>
    <item><title>Async patterns in Rust</title></item>
  </channel>
</rss>"#;

        let mock = server
            .mock("GET", "/feed.xml")
            .with_status(200)
            .with_header("content-type", "application/xml")
            .with_body(feed_xml)
            .create();

        let url = format!("{}/feed.xml", server.url());
        let fetcher = HttpRssFetcher::new("test-blog", &url);
        let items = fetcher.fetch()?;

        mock.assert();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Rust 1.85 released");
        assert_eq!(items[0].source_id, "test-blog");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].title, "Async patterns in Rust");
        assert_eq!(items[1].rank, 2);
        Ok(())
    }

    #[test]
    fn http_rss_fetcher_returns_empty_for_empty_channel() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        let feed_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Empty Feed</title>
  </channel>
</rss>"#;

        server
            .mock("GET", "/empty.xml")
            .with_status(200)
            .with_body(feed_xml)
            .create();

        let url = format!("{}/empty.xml", server.url());
        let fetcher = HttpRssFetcher::new("empty-blog", &url);
        let items = fetcher.fetch()?;

        assert!(items.is_empty());
        Ok(())
    }

    #[test]
    fn http_rss_fetcher_reports_http_error() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        server.mock("GET", "/broken").with_status(500).create();

        let url = format!("{}/broken", server.url());
        let fetcher = HttpRssFetcher::new("failing-blog", &url);
        let error = fetcher.fetch().expect_err("should fail on HTTP 500");

        let message = error.to_string();
        assert!(
            message.contains("http 500"),
            "expected http 500 in: {message}"
        );
        assert!(message.contains("/broken"), "expected url in: {message}");
        Ok(())
    }

    #[test]
    fn http_rss_fetcher_reports_parse_error_for_invalid_xml() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        server
            .mock("GET", "/bad-xml")
            .with_status(200)
            .with_body("this is not xml at all")
            .create();

        let url = format!("{}/bad-xml", server.url());
        let fetcher = HttpRssFetcher::new("bad-xml-blog", &url);
        let error = fetcher.fetch().expect_err("should fail on invalid XML");

        let message = error.to_string();
        assert!(
            message.contains("failed to parse response"),
            "expected parse error in: {message}"
        );
        Ok(())
    }

    #[test]
    fn http_rss_fetcher_reports_network_error() -> Result<(), Box<dyn Error>> {
        let fetcher = HttpRssFetcher::new("dead-blog", "http://127.0.0.1:1/rss.xml");
        let error = fetcher
            .fetch()
            .expect_err("should fail on unreachable host");

        let message = error.to_string();
        assert!(
            message.contains("network error"),
            "expected network error in: {message}"
        );
        Ok(())
    }

    #[test]
    fn http_hotlist_fetcher_parses_valid_json() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        let body = r#"[{"title":"Breaking news","rank":1},{"title":"Tech update","rank":2}]"#;

        let mock = server
            .mock("GET", "/hotlist")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let url = format!("{}/hotlist", server.url());
        let fetcher = HttpHotlistFetcher::new("test-platform", &url);
        let items = fetcher.fetch()?;

        mock.assert();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Breaking news");
        assert_eq!(items[0].source_id, "test-platform");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].title, "Tech update");
        assert_eq!(items[1].rank, 2);
        Ok(())
    }

    #[test]
    fn http_hotlist_fetcher_returns_empty_for_empty_array() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        server
            .mock("GET", "/empty")
            .with_status(200)
            .with_body("[]")
            .create();

        let url = format!("{}/empty", server.url());
        let fetcher = HttpHotlistFetcher::new("empty-platform", &url);
        let items = fetcher.fetch()?;

        assert!(items.is_empty());
        Ok(())
    }

    #[test]
    fn http_hotlist_fetcher_reports_http_error() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        server.mock("GET", "/fail").with_status(403).create();

        let url = format!("{}/fail", server.url());
        let fetcher = HttpHotlistFetcher::new("blocked-platform", &url);
        let error = fetcher.fetch().expect_err("should fail on HTTP 403");

        let message = error.to_string();
        assert!(
            message.contains("http 403"),
            "expected http 403 in: {message}"
        );
        Ok(())
    }

    #[test]
    fn http_hotlist_fetcher_reports_parse_error_for_invalid_json() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        server
            .mock("GET", "/bad-json")
            .with_status(200)
            .with_body("not json")
            .create();

        let url = format!("{}/bad-json", server.url());
        let fetcher = HttpHotlistFetcher::new("bad-platform", &url);
        let error = fetcher.fetch().expect_err("should fail on invalid JSON");

        let message = error.to_string();
        assert!(
            message.contains("failed to parse response"),
            "expected parse error in: {message}"
        );
        Ok(())
    }

    #[test]
    fn http_hotlist_fetcher_reports_network_error() -> Result<(), Box<dyn Error>> {
        let fetcher = HttpHotlistFetcher::new("dead-platform", "http://127.0.0.1:1/hotlist");
        let error = fetcher
            .fetch()
            .expect_err("should fail on unreachable host");

        let message = error.to_string();
        assert!(
            message.contains("network error"),
            "expected network error in: {message}"
        );
        Ok(())
    }
}

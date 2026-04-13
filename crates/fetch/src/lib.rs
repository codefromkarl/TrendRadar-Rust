//! 抓取层：fixture adapter 与 HTTP adapter。

use serde::Deserialize;
use std::fs::read_to_string;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Duration;
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
// Hotlist parsers（多平台热榜解析策略）
// ---------------------------------------------------------------------------

/// 热榜数据解析策略。
pub trait HotlistParser: Send + Sync {
    /// 将原始 JSON 文本解析为新闻条目。
    fn parse(&self, raw: &str, platform_id: &str) -> Result<Vec<NewsItem>>;
}

/// 通用热榜解析器（原有格式）。
#[derive(Debug)]
pub struct GenericHotlistParser;

impl HotlistParser for GenericHotlistParser {
    fn parse(&self, raw: &str, platform_id: &str) -> Result<Vec<NewsItem>> {
        let items: Vec<HotlistFixtureItem> =
            serde_json::from_str(raw).map_err(|error| FetchError::ParseResponse {
                url: platform_id.to_owned(),
                message: error.to_string(),
            })?;

        let items = items
            .into_iter()
            .map(|item| NewsItem {
                title: item.title,
                source_id: platform_id.to_owned(),
                rank: item.rank,
            })
            .collect();

        Ok(items)
    }
}

/// 微博热搜解析器。
#[derive(Debug)]
pub struct WeiboHotlistParser;

#[derive(Debug, Deserialize)]
struct WeiboHotlistData {
    realtime: Vec<WeiboHotlistItem>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WeiboHotlistItem {
    word: String,
    #[serde(default)]
    num: u64,
}

#[derive(Debug, Deserialize)]
struct WeiboHotlistResponse {
    data: WeiboHotlistData,
}

impl HotlistParser for WeiboHotlistParser {
    fn parse(&self, raw: &str, platform_id: &str) -> Result<Vec<NewsItem>> {
        let response: WeiboHotlistResponse =
            serde_json::from_str(raw).map_err(|error| FetchError::ParseResponse {
                url: platform_id.to_owned(),
                message: error.to_string(),
            })?;

        let items = response
            .data
            .realtime
            .into_iter()
            .enumerate()
            .map(|(index, item)| NewsItem {
                title: item.word,
                source_id: platform_id.to_owned(),
                rank: (index + 1) as u32,
            })
            .collect();

        Ok(items)
    }
}

/// 知乎热榜解析器。
#[derive(Debug)]
pub struct ZhihuHotlistParser;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhihuHotlistItem {
    title: String,
    #[serde(default)]
    detail_text: String,
}

#[derive(Debug, Deserialize)]
struct ZhihuHotlistResponse {
    data: Vec<ZhihuHotlistItem>,
}

impl HotlistParser for ZhihuHotlistParser {
    fn parse(&self, raw: &str, platform_id: &str) -> Result<Vec<NewsItem>> {
        let response: ZhihuHotlistResponse =
            serde_json::from_str(raw).map_err(|error| FetchError::ParseResponse {
                url: platform_id.to_owned(),
                message: error.to_string(),
            })?;

        let items = response
            .data
            .into_iter()
            .enumerate()
            .map(|(index, item)| NewsItem {
                title: item.title,
                source_id: platform_id.to_owned(),
                rank: (index + 1) as u32,
            })
            .collect();

        Ok(items)
    }
}

/// B站热榜解析器。
#[derive(Debug)]
pub struct BilibiliHotlistParser;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BilibiliHotlistItem {
    title: String,
    #[serde(default)]
    hot_value: u64,
}

#[derive(Debug, Deserialize)]
struct BilibiliHotlistData {
    list: Vec<BilibiliHotlistItem>,
}

#[derive(Debug, Deserialize)]
struct BilibiliHotlistResponse {
    data: BilibiliHotlistData,
}

impl HotlistParser for BilibiliHotlistParser {
    fn parse(&self, raw: &str, platform_id: &str) -> Result<Vec<NewsItem>> {
        let response: BilibiliHotlistResponse =
            serde_json::from_str(raw).map_err(|error| FetchError::ParseResponse {
                url: platform_id.to_owned(),
                message: error.to_string(),
            })?;

        let items = response
            .data
            .list
            .into_iter()
            .enumerate()
            .map(|(index, item)| NewsItem {
                title: item.title,
                source_id: platform_id.to_owned(),
                rank: (index + 1) as u32,
            })
            .collect();

        Ok(items)
    }
}

/// 根据数据源类型返回对应的解析器。
pub fn hotlist_parser_for(source_type: &str) -> Box<dyn HotlistParser> {
    match source_type {
        "weibo" => Box::new(WeiboHotlistParser),
        "zhihu" => Box::new(ZhihuHotlistParser),
        "bilibili" => Box::new(BilibiliHotlistParser),
        _ => Box::new(GenericHotlistParser),
    }
}

// ---------------------------------------------------------------------------
// HTTP adapters
// ---------------------------------------------------------------------------

/// 构建带超时的 reqwest blocking Client。
fn build_http_client(timeout: Duration) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

/// 基于 HTTP 的 RSS 抓取器。
///
/// 从远程 RSS feed 拉取条目，解析 XML 并归一化为 `NewsItem`。
pub struct HttpRssFetcher {
    source_id: String,
    url: String,
    client: reqwest::blocking::Client,
}

impl HttpRssFetcher {
    /// 创建 HTTP RSS 抓取器（默认 30 秒超时）。
    #[must_use]
    pub fn new(source_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self::with_timeout(source_id, url, Duration::from_secs(30))
    }

    /// 创建带自定义超时的 HTTP RSS 抓取器。
    #[must_use]
    pub fn with_timeout(
        source_id: impl Into<String>,
        url: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            url: url.into(),
            client: build_http_client(timeout),
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
    parser: Box<dyn HotlistParser>,
}

impl HttpHotlistFetcher {
    /// 创建 HTTP 热榜抓取器（默认 30 秒超时，使用通用解析器）。
    #[must_use]
    pub fn new(platform_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self::with_timeout(platform_id, url, Duration::from_secs(30))
    }

    /// 创建带自定义超时的 HTTP 热榜抓取器（使用通用解析器）。
    #[must_use]
    pub fn with_timeout(
        platform_id: impl Into<String>,
        url: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            platform_id: platform_id.into(),
            url: url.into(),
            client: build_http_client(timeout),
            parser: Box::new(GenericHotlistParser),
        }
    }

    /// 创建带自定义解析器的 HTTP 热榜抓取器。
    #[must_use]
    pub fn with_parser(
        platform_id: impl Into<String>,
        url: impl Into<String>,
        timeout: Duration,
        parser: Box<dyn HotlistParser>,
    ) -> Self {
        Self {
            platform_id: platform_id.into(),
            url: url.into(),
            client: build_http_client(timeout),
            parser,
        }
    }
}

impl Fetcher for HttpHotlistFetcher {
    fn fetch(&self) -> Result<Vec<NewsItem>> {
        let body = http_get_text(&self.client, &self.url)?;
        self.parser.parse(&body, &self.platform_id)
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
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        BilibiliHotlistParser, Fetcher, FixtureHotlistFetcher, FixtureRssFetcher,
        GenericHotlistParser, HotlistParser, HttpHotlistFetcher, HttpRssFetcher,
        WeiboHotlistParser, ZhihuHotlistParser, hotlist_parser_for,
    };
    use std::error::Error;
    use std::fs::read_to_string;
    use std::time::Duration;
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

    #[test]
    fn http_fetcher_with_custom_timeout_uses_configured_duration() -> Result<(), Box<dyn Error>> {
        // Short timeout should trigger network error for slow server
        let fetcher = HttpHotlistFetcher::with_timeout(
            "slow",
            "http://127.0.0.1:1/hotlist",
            Duration::from_millis(1),
        );
        let error = fetcher
            .fetch()
            .expect_err("should fail on unreachable host");

        assert!(
            error.to_string().contains("network error"),
            "expected network error"
        );
        Ok(())
    }

    // -- Parser tests --

    #[test]
    fn generic_hotlist_parser_parses_valid_json() -> Result<(), Box<dyn Error>> {
        let parser = GenericHotlistParser;
        let raw = r#"[{"title":"Breaking news","rank":1},{"title":"Tech update","rank":2}]"#;

        let items = parser.parse(raw, "test-platform")?;

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Breaking news");
        assert_eq!(items[0].source_id, "test-platform");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].title, "Tech update");
        assert_eq!(items[1].rank, 2);
        Ok(())
    }

    #[test]
    fn generic_hotlist_parser_returns_empty_for_empty_array() -> Result<(), Box<dyn Error>> {
        let parser = GenericHotlistParser;
        let raw = r#"[]"#;

        let items = parser.parse(raw, "empty-platform")?;

        assert!(items.is_empty());
        Ok(())
    }

    #[test]
    fn generic_hotlist_parser_reports_error_for_invalid_json() -> Result<(), Box<dyn Error>> {
        let parser = GenericHotlistParser;
        let raw = r#"not json"#;

        let error = parser
            .parse(raw, "bad-platform")
            .expect_err("should fail on invalid JSON");

        let message = error.to_string();
        assert!(
            message.contains("failed to parse response"),
            "expected parse error in: {message}"
        );
        Ok(())
    }

    #[test]
    fn weibo_hotlist_parser_parses_valid_json() -> Result<(), Box<dyn Error>> {
        let parser = WeiboHotlistParser;
        let raw = r#"{
            "data": {
                "realtime": [
                    {"word": "热搜话题1", "num": 1234567},
                    {"word": "热搜话题2", "num": 234567}
                ]
            }
        }"#;

        let items = parser.parse(raw, "weibo")?;

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "热搜话题1");
        assert_eq!(items[0].source_id, "weibo");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].title, "热搜话题2");
        assert_eq!(items[1].rank, 2);
        Ok(())
    }

    #[test]
    fn weibo_hotlist_parser_returns_empty_for_empty_data() -> Result<(), Box<dyn Error>> {
        let parser = WeiboHotlistParser;
        let raw = r#"{"data": {"realtime": []}}"#;

        let items = parser.parse(raw, "weibo")?;

        assert!(items.is_empty());
        Ok(())
    }

    #[test]
    fn weibo_hotlist_parser_handles_missing_fields_gracefully() -> Result<(), Box<dyn Error>> {
        let parser = WeiboHotlistParser;
        // 缺失 num 字段时使用默认值
        let raw = r#"{
            "data": {
                "realtime": [
                    {"word": "话题1"}
                ]
            }
        }"#;

        let items = parser.parse(raw, "weibo")?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "话题1");
        Ok(())
    }

    #[test]
    fn zhihu_hotlist_parser_parses_valid_json() -> Result<(), Box<dyn Error>> {
        let parser = ZhihuHotlistParser;
        let raw = r#"{
            "data": [
                {"title": "知乎问题1", "detail_text": "100万热度"},
                {"title": "知乎问题2", "detail_text": "50万热度"}
            ]
        }"#;

        let items = parser.parse(raw, "zhihu")?;

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "知乎问题1");
        assert_eq!(items[0].source_id, "zhihu");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].title, "知乎问题2");
        assert_eq!(items[1].rank, 2);
        Ok(())
    }

    #[test]
    fn zhihu_hotlist_parser_returns_empty_for_empty_data() -> Result<(), Box<dyn Error>> {
        let parser = ZhihuHotlistParser;
        let raw = r#"{"data": []}"#;

        let items = parser.parse(raw, "zhihu")?;

        assert!(items.is_empty());
        Ok(())
    }

    #[test]
    fn zhihu_hotlist_parser_handles_missing_detail_text() -> Result<(), Box<dyn Error>> {
        let parser = ZhihuHotlistParser;
        let raw = r#"{
            "data": [
                {"title": "问题1"}
            ]
        }"#;

        let items = parser.parse(raw, "zhihu")?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "问题1");
        Ok(())
    }

    #[test]
    fn bilibili_hotlist_parser_parses_valid_json() -> Result<(), Box<dyn Error>> {
        let parser = BilibiliHotlistParser;
        let raw = r#"{
            "data": {
                "list": [
                    {"title": "B站视频1", "hot_value": 5000000},
                    {"title": "B站视频2", "hot_value": 3000000}
                ]
            }
        }"#;

        let items = parser.parse(raw, "bilibili")?;

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "B站视频1");
        assert_eq!(items[0].source_id, "bilibili");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].title, "B站视频2");
        assert_eq!(items[1].rank, 2);
        Ok(())
    }

    #[test]
    fn bilibili_hotlist_parser_returns_empty_for_empty_data() -> Result<(), Box<dyn Error>> {
        let parser = BilibiliHotlistParser;
        let raw = r#"{"data": {"list": []}}"#;

        let items = parser.parse(raw, "bilibili")?;

        assert!(items.is_empty());
        Ok(())
    }

    #[test]
    fn bilibili_hotlist_parser_handles_missing_hot_value() -> Result<(), Box<dyn Error>> {
        let parser = BilibiliHotlistParser;
        let raw = r#"{
            "data": {
                "list": [
                    {"title": "视频1"}
                ]
            }
        }"#;

        let items = parser.parse(raw, "bilibili")?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "视频1");
        Ok(())
    }

    #[test]
    fn hotlist_parser_for_returns_correct_parsers() {
        // 测试 weibo parser
        let weibo_parser = hotlist_parser_for("weibo");
        let weibo_raw = r#"{"data": {"realtime": [{"word": "测试", "num": 100}]}}"#;
        assert!(weibo_parser.parse(weibo_raw, "weibo").is_ok());

        // 测试 zhihu parser
        let zhihu_parser = hotlist_parser_for("zhihu");
        let zhihu_raw = r#"{"data": [{"title": "测试"}]}"#;
        assert!(zhihu_parser.parse(zhihu_raw, "zhihu").is_ok());

        // 测试 bilibili parser
        let bilibili_parser = hotlist_parser_for("bilibili");
        let bilibili_raw = r#"{"data": {"list": [{"title": "测试"}]}}"#;
        assert!(bilibili_parser.parse(bilibili_raw, "bilibili").is_ok());

        // 测试 generic parser（默认）
        let generic_parser = hotlist_parser_for("generic");
        let generic_raw = r#"[{"title": "测试", "rank": 1}]"#;
        assert!(generic_parser.parse(generic_raw, "generic").is_ok());

        // 测试未知类型默认为 generic
        let unknown_parser = hotlist_parser_for("unknown");
        assert!(unknown_parser.parse(generic_raw, "unknown").is_ok());
    }

    #[test]
    fn http_hotlist_fetcher_with_parser_uses_custom_parser() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        let body = r#"{
            "data": {
                "realtime": [
                    {"word": "微博热搜1", "num": 1000000},
                    {"word": "微博热搜2", "num": 500000}
                ]
            }
        }"#;

        let mock = server
            .mock("GET", "/weibo")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let url = format!("{}/weibo", server.url());
        let parser = Box::new(WeiboHotlistParser);
        let fetcher =
            HttpHotlistFetcher::with_parser("weibo", &url, Duration::from_secs(30), parser);
        let items = fetcher.fetch()?;

        mock.assert();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "微博热搜1");
        assert_eq!(items[0].source_id, "weibo");
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[1].title, "微博热搜2");
        assert_eq!(items[1].rank, 2);
        Ok(())
    }
}

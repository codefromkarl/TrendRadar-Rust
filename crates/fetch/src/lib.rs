//! 抓取层骨架。

use trendradar_domain::NewsItem;

/// 抓取器接口。
pub trait Fetcher {
    /// 拉取一批新闻条目。
    fn fetch(&self) -> Vec<NewsItem>;
}

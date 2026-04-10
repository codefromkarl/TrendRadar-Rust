//! 存储抽象骨架。

use trendradar_domain::{NewsItem, Result};

/// 新闻存储接口。
pub trait NewsRepository {
    /// 保存一条新闻。
    fn save_news(&mut self, item: NewsItem) -> Result<()>;
}

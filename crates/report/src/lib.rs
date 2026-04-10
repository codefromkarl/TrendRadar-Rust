//! 输出层骨架。

use trendradar_domain::NewsItem;

/// 将新闻列表渲染为 JSON。
pub fn render_news_json(items: &[NewsItem]) -> serde_json::Result<String> {
    serde_json::to_string_pretty(items)
}

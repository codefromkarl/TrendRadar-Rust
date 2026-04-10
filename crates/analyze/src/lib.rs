//! 聚合与排序分析骨架。

use trendradar_domain::NewsItem;

/// 计算新闻的基础权重。
#[must_use]
pub fn score_news(item: &NewsItem) -> u32 {
    101_u32.saturating_sub(item.rank.min(100))
}

#[cfg(test)]
mod tests {
    use super::score_news;
    use trendradar_domain::NewsItem;

    #[test]
    fn lower_rank_scores_higher() {
        let item = NewsItem {
            title: "example".to_owned(),
            source_id: "weibo".to_owned(),
            rank: 1,
        };

        assert_eq!(score_news(&item), 100);
    }
}

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
    use std::error::Error;
    use std::fs::read_to_string;
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

    #[test]
    fn fixture_scores_match_expected_values() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/analyze/news-ranking-input.json"
        );
        let contents = read_to_string(fixture_path)?;
        let items: Vec<NewsItem> = serde_json::from_str(&contents)?;

        let scores: Vec<u32> = items.iter().map(score_news).collect();

        assert_eq!(scores, vec![100, 89, 1]);
        Ok(())
    }
}

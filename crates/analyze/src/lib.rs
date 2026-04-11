//! 聚合与排序分析骨架。

use std::collections::BTreeMap;

use trendradar_domain::NewsItem;

/// 带分数的新闻结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedNews {
    /// 原始新闻条目。
    pub item: NewsItem,
    /// 计算得到的分数。
    pub score: u32,
}

/// 来源聚合摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSummary {
    /// 来源标识。
    pub source_id: String,
    /// 来源下的条目数。
    pub item_count: usize,
    /// 来源下的最佳排名。
    pub best_rank: u32,
}

/// 计算新闻的基础权重。
#[must_use]
pub fn score_news(item: &NewsItem) -> u32 {
    let effective_rank = item.rank.clamp(1, 100);
    101_u32.saturating_sub(effective_rank)
}

/// 按分数对新闻排序。
#[must_use]
pub fn rank_news(items: &[NewsItem]) -> Vec<RankedNews> {
    let mut ranked: Vec<RankedNews> = items
        .iter()
        .cloned()
        .map(|item| RankedNews {
            score: score_news(&item),
            item,
        })
        .collect();

    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.item.rank.cmp(&right.item.rank))
            .then_with(|| left.item.title.cmp(&right.item.title))
    });

    ranked
}

/// 按来源聚合新闻条目。
#[must_use]
pub fn group_news_by_source(items: &[NewsItem]) -> Vec<SourceSummary> {
    let mut groups: BTreeMap<String, SourceSummary> = BTreeMap::new();

    for item in items {
        groups
            .entry(item.source_id.clone())
            .and_modify(|summary| {
                summary.item_count += 1;
                summary.best_rank = summary.best_rank.min(item.rank);
            })
            .or_insert_with(|| SourceSummary {
                source_id: item.source_id.clone(),
                item_count: 1,
                best_rank: item.rank,
            });
    }

    let mut summaries: Vec<SourceSummary> = groups.into_values().collect();
    summaries.sort_by(|left, right| {
        right
            .item_count
            .cmp(&left.item_count)
            .then_with(|| left.best_rank.cmp(&right.best_rank))
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    summaries
}

#[cfg(test)]
mod tests {
    use super::{group_news_by_source, rank_news, score_news};
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

    #[test]
    fn rank_news_orders_items_by_score_descending() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/analyze/news-ranking-input.json"
        );
        let contents = read_to_string(fixture_path)?;
        let items: Vec<NewsItem> = serde_json::from_str(&contents)?;

        let ranked = rank_news(&items);
        let ranked_titles: Vec<&str> = ranked
            .iter()
            .map(|entry| entry.item.title.as_str())
            .collect();
        let ranked_scores: Vec<u32> = ranked.iter().map(|entry| entry.score).collect();

        assert_eq!(
            ranked_titles,
            vec![
                "Rust 1.85.0 released",
                "TrendRadar migration plan updated",
                "Weekly engineering digest"
            ]
        );
        assert_eq!(ranked_scores, vec![100, 89, 1]);
        Ok(())
    }

    #[test]
    fn group_news_by_source_counts_items_and_best_rank() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/analyze/source-groups-input.json"
        );
        let contents = read_to_string(fixture_path)?;
        let items: Vec<NewsItem> = serde_json::from_str(&contents)?;

        let groups = group_news_by_source(&items);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].source_id, "github-trending");
        assert_eq!(groups[0].item_count, 2);
        assert_eq!(groups[0].best_rank, 1);
        assert_eq!(groups[1].source_id, "community-hotlist");
        assert_eq!(groups[1].item_count, 1);
        assert_eq!(groups[1].best_rank, 12);
        Ok(())
    }

    #[test]
    fn zero_rank_is_clamped_to_top_score() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/analyze/zero-rank-input.json"
        );
        let contents = read_to_string(fixture_path)?;
        let items: Vec<NewsItem> = serde_json::from_str(&contents)?;

        let scores: Vec<u32> = items.iter().map(score_news).collect();
        let ranked = rank_news(&items);

        assert_eq!(scores, vec![100, 100]);
        assert_eq!(ranked[0].score, 100);
        assert_eq!(ranked[1].score, 100);
        Ok(())
    }

    #[test]
    fn rank_news_uses_title_as_final_tiebreaker() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/analyze/same-rank-input.json"
        );
        let contents = read_to_string(fixture_path)?;
        let items: Vec<NewsItem> = serde_json::from_str(&contents)?;

        let ranked = rank_news(&items);
        let ranked_titles: Vec<&str> = ranked
            .iter()
            .map(|entry| entry.item.title.as_str())
            .collect();

        assert_eq!(
            ranked_titles,
            vec![
                "Alpha release note",
                "Beta release note",
                "Zeta release note"
            ]
        );
        Ok(())
    }
}

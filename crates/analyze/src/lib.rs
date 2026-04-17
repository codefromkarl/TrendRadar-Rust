//! 聚合、排序与关键词过滤分析。

use std::collections::BTreeMap;

use serde::Serialize;
use trendradar_domain::NewsItem;

/// 带分数的新闻结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RankedNews {
    /// 原始新闻条目。
    pub item: NewsItem,
    /// 计算得到的分数。
    pub score: u32,
}

/// 来源聚合摘要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSummary {
    /// 来源标识。
    pub source_id: String,
    /// 来源下的条目数。
    pub item_count: usize,
    /// 来源下的最佳排名。
    pub best_rank: u32,
}

/// 领域分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainCategory {
    /// 人工智能。
    Ai,
    /// 科技。
    Technology,
    /// 科学。
    Science,
    /// 财经。
    Finance,
    /// 健康。
    Health,
    /// 体育。
    Sports,
    /// 国际 / 时政。
    World,
    /// 商业。
    Business,
    /// 其他。
    General,
}

/// 领域聚合摘要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DomainSummary {
    /// 领域。
    pub domain: DomainCategory,
    /// 领域下的条目数。
    pub item_count: usize,
    /// 领域下的最佳排名。
    pub best_rank: u32,
}

fn normalized_title(title: &str) -> String {
    title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| text.contains(keyword))
}

/// 基于标题与来源信号做领域分类。
#[must_use]
pub fn classify_domain(item: &NewsItem) -> DomainCategory {
    let signal = format!(
        "{} {}",
        item.source_id.to_lowercase(),
        item.title.to_lowercase()
    );

    if contains_any(
        &signal,
        &[
            "finance",
            "bank",
            "banker",
            "stock",
            "stocks",
            "market",
            "markets",
            "economy",
            "economic",
            "fund",
            "funds",
            "investment",
            "investor",
            "earnings",
            "revenue",
            "profit",
            "profits",
            "loan",
            "loans",
        ],
    ) {
        return DomainCategory::Finance;
    }

    if contains_any(
        &signal,
        &[
            "health", "medical", "medicine", "hospital", "cancer", "disease", "drug", "vaccine",
            "virus", "lung", "doctor",
        ],
    ) {
        return DomainCategory::Health;
    }

    if contains_any(
        &signal,
        &[
            "sport",
            "sports",
            "fifa",
            "world cup",
            "nba",
            "nfl",
            "mlb",
            "goal",
            "match",
            "fixture",
            "playoff",
            "tennis",
            "soccer",
            "yahoo sports",
        ],
    ) {
        return DomainCategory::Sports;
    }

    if contains_any(
        &signal,
        &[
            "white house",
            "trump",
            "election",
            "government",
            "official",
            "officials",
            "war",
            "military",
            "iran",
            "israel",
            "ukraine",
            "china",
            "eu",
            "u.n",
            "united nations",
            "who)",
            "who ",
            "policy",
            "agencies",
            "agency",
            "airstrike",
            "blacklist",
            "minister",
            "ministers",
        ],
    ) {
        return DomainCategory::World;
    }

    if contains_any(
        &signal,
        &[
            "openai",
            "anthropic",
            "claude",
            "chatgpt",
            "gemini",
            "llm",
            "artificial intelligence",
            " ai ",
            " ai-",
            " ai,",
            "mythos",
            "codex",
            "machine learning",
            "deepseek",
            "copilot",
        ],
    ) {
        return DomainCategory::Ai;
    }

    if contains_any(
        &signal,
        &[
            "science",
            "scientific",
            "research",
            "researchers",
            "study",
            "studies",
            "nasa",
            "space",
            "physics",
            "biology",
            "chemistry",
            "laboratory",
        ],
    ) {
        return DomainCategory::Science;
    }

    if contains_any(
        &signal,
        &[
            "technology",
            "tech",
            "software",
            "hardware",
            "developer",
            "developers",
            "cyber",
            "security",
            "chip",
            "cloud",
            "rust",
            "cargo",
            "programming",
        ],
    ) {
        return DomainCategory::Technology;
    }

    if contains_any(
        &signal,
        &[
            "business",
            "company",
            "companies",
            "startup",
            "industry",
            "ceo",
            "acquisition",
            "merger",
        ],
    ) {
        return DomainCategory::Business;
    }

    DomainCategory::General
}

/// 按标题做跨来源全局去重。
#[must_use]
pub fn dedupe_news_by_title(items: &[NewsItem]) -> Vec<NewsItem> {
    let mut best_by_title: BTreeMap<String, NewsItem> = BTreeMap::new();

    for item in items {
        let key = normalized_title(&item.title);
        best_by_title
            .entry(key)
            .and_modify(|best| {
                if item.rank < best.rank
                    || (item.rank == best.rank && item.source_id < best.source_id)
                {
                    *best = item.clone();
                }
            })
            .or_insert_with(|| item.clone());
    }

    let mut deduped: Vec<NewsItem> = best_by_title.into_values().collect();
    deduped.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    deduped
}

/// 按领域聚合新闻条目。
#[must_use]
pub fn group_news_by_domain(items: &[NewsItem]) -> Vec<DomainSummary> {
    let mut groups: BTreeMap<DomainCategory, DomainSummary> = BTreeMap::new();

    for item in items {
        let domain = classify_domain(item);
        groups
            .entry(domain)
            .and_modify(|summary| {
                summary.item_count += 1;
                summary.best_rank = summary.best_rank.min(item.rank);
            })
            .or_insert_with(|| DomainSummary {
                domain,
                item_count: 1,
                best_rank: item.rank,
            });
    }

    let mut summaries: Vec<DomainSummary> = groups.into_values().collect();
    summaries.sort_by(|left, right| {
        right
            .item_count
            .cmp(&left.item_count)
            .then_with(|| left.best_rank.cmp(&right.best_rank))
            .then_with(|| left.domain.cmp(&right.domain))
    });
    summaries
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

/// 按关键词列表过滤新闻条目（不区分大小写）。
///
/// 若关键词列表为空，返回全部条目（不做过滤）。
/// 标题只需匹配任意一个关键词即保留。
#[must_use]
pub fn filter_by_keywords(items: &[NewsItem], keywords: &[String]) -> Vec<NewsItem> {
    if keywords.is_empty() {
        return items.to_vec();
    }

    let lower_keywords: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();

    items
        .iter()
        .filter(|item| {
            let lower_title = item.title.to_lowercase();
            lower_keywords.iter().any(|kw| lower_title.contains(kw))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        DomainCategory, classify_domain, dedupe_news_by_title, filter_by_keywords,
        group_news_by_domain, group_news_by_source, rank_news, score_news,
    };
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

    // -- Keyword filtering tests --

    #[test]
    fn filter_by_keywords_returns_all_when_keywords_empty() {
        let items = vec![
            NewsItem {
                title: "Rust 1.85".to_owned(),
                source_id: "a".to_owned(),
                rank: 1,
            },
            NewsItem {
                title: "Python 3.13".to_owned(),
                source_id: "b".to_owned(),
                rank: 2,
            },
        ];

        let result = filter_by_keywords(&items, &[]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedupe_news_by_title_keeps_best_ranked_copy_across_sources() {
        let items = vec![
            NewsItem {
                title: "OpenAI launches new AI model for life sciences research - Axios".to_owned(),
                source_id: "ai-today".to_owned(),
                rank: 4,
            },
            NewsItem {
                title: "OpenAI launches new AI model for life sciences research - Axios".to_owned(),
                source_id: "openai-today".to_owned(),
                rank: 2,
            },
            NewsItem {
                title: "Different headline".to_owned(),
                source_id: "world".to_owned(),
                rank: 1,
            },
        ];

        let deduped = dedupe_news_by_title(&items);
        assert_eq!(deduped.len(), 2);
        assert!(
            deduped
                .iter()
                .any(|item| item.source_id == "openai-today" && item.rank == 2)
        );
    }

    #[test]
    fn classify_domain_detects_ai_finance_and_sports() {
        let ai = NewsItem {
            title: "OpenAI launches new AI model for life sciences research - Axios".to_owned(),
            source_id: "news".to_owned(),
            rank: 1,
        };
        let finance = NewsItem {
            title: "Finance ministers and top bankers raise serious concerns about Mythos AI model - BBC".to_owned(),
            source_id: "news".to_owned(),
            rank: 1,
        };
        let sports = NewsItem {
            title:
                "2026 FIFA World Cup schedule: Qualified teams, groups, match dates - Yahoo Sports"
                    .to_owned(),
            source_id: "news".to_owned(),
            rank: 1,
        };

        assert_eq!(classify_domain(&ai), DomainCategory::Ai);
        assert_eq!(classify_domain(&finance), DomainCategory::Finance);
        assert_eq!(classify_domain(&sports), DomainCategory::Sports);
    }

    #[test]
    fn group_news_by_domain_summarizes_mixed_items() {
        let items = vec![
            NewsItem {
                title: "OpenAI launches new AI model for life sciences research - Axios".to_owned(),
                source_id: "a".to_owned(),
                rank: 4,
            },
            NewsItem {
                title: "Codex for (almost) everything - OpenAI".to_owned(),
                source_id: "b".to_owned(),
                rank: 1,
            },
            NewsItem {
                title: "Finance ministers and top bankers raise serious concerns about Mythos AI model - BBC".to_owned(),
                source_id: "c".to_owned(),
                rank: 2,
            },
        ];

        let grouped = group_news_by_domain(&items);
        assert_eq!(grouped[0].domain, DomainCategory::Ai);
        assert_eq!(grouped[0].item_count, 2);
        assert_eq!(grouped[0].best_rank, 1);
        assert_eq!(grouped[1].domain, DomainCategory::Finance);
    }

    #[test]
    fn filter_by_keywords_matches_case_insensitive() {
        let items = vec![
            NewsItem {
                title: "Rust 1.85 Released".to_owned(),
                source_id: "a".to_owned(),
                rank: 1,
            },
            NewsItem {
                title: "Python 3.13 Released".to_owned(),
                source_id: "b".to_owned(),
                rank: 2,
            },
        ];

        let keywords = vec!["rust".to_owned()];
        let result = filter_by_keywords(&items, &keywords);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Rust 1.85 Released");
    }

    #[test]
    fn filter_by_keywords_matches_any_keyword() {
        let items = vec![
            NewsItem {
                title: "Rust 1.85".to_owned(),
                source_id: "a".to_owned(),
                rank: 1,
            },
            NewsItem {
                title: "Python 3.13".to_owned(),
                source_id: "b".to_owned(),
                rank: 2,
            },
            NewsItem {
                title: "Go 1.22".to_owned(),
                source_id: "c".to_owned(),
                rank: 3,
            },
        ];

        let keywords = vec!["rust".to_owned(), "go".to_owned()];
        let result = filter_by_keywords(&items, &keywords);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_by_keywords_returns_empty_when_no_match() {
        let items = vec![NewsItem {
            title: "Rust 1.85".to_owned(),
            source_id: "a".to_owned(),
            rank: 1,
        }];

        let keywords = vec!["python".to_owned()];
        let result = filter_by_keywords(&items, &keywords);
        assert!(result.is_empty());
    }
}

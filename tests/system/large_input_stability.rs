use anyhow::Result;
use chrono::TimeZone;
use serde::Deserialize;
use std::collections::BTreeMap;
use trendradar_domain::{NewsItem, RunContext};
use trendradar_report::render_news_json;
use trendradar_storage::{NewsRepository, SqliteNewsRepository};

fn build_large_input() -> Vec<NewsItem> {
    let sources = ["weibo", "zhihu", "bilibili", "baidu", "toutiao", "pengpai"];
    let mut items = Vec::new();

    for source in sources {
        for idx in 0..40_u32 {
            let title = format!("Topic {idx:03}");
            let base_rank = ((idx * 7 + source.len() as u32) % 20) + 1;
            items.push(NewsItem {
                title: title.clone(),
                source_id: source.to_owned(),
                rank: base_rank,
            });

            if idx % 4 == 0 {
                items.push(NewsItem {
                    title: title.clone(),
                    source_id: source.to_owned(),
                    rank: base_rank + 10,
                });
            }

            if idx % 6 == 0 {
                items.push(NewsItem {
                    title,
                    source_id: source.to_owned(),
                    rank: base_rank.saturating_sub(1).max(1),
                });
            }
        }
    }

    items
}

fn expected_stable_items(items: &[NewsItem]) -> Vec<NewsItem> {
    let mut best_by_key: BTreeMap<(String, String), u32> = BTreeMap::new();

    for item in items {
        let key = (item.source_id.clone(), item.title.clone());
        best_by_key
            .entry(key)
            .and_modify(|rank| *rank = (*rank).min(item.rank))
            .or_insert(item.rank);
    }

    let mut expected: Vec<NewsItem> = best_by_key
        .into_iter()
        .map(|((source_id, title), rank)| NewsItem {
            title,
            source_id,
            rank,
        })
        .collect();
    expected.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.source_id.cmp(&right.source_id))
            .then_with(|| left.title.cmp(&right.title))
    });
    expected
}

#[derive(Debug, Deserialize)]
struct JsonReport {
    meta: JsonReportMeta,
    items: Vec<NewsItem>,
}

#[derive(Debug, Deserialize)]
struct JsonReportMeta {
    item_count: usize,
}

#[test]
fn large_input_roundtrip_remains_stably_sorted_and_deduplicated() -> Result<()> {
    let input = build_large_input();
    let expected = expected_stable_items(&input);
    let mut repository = SqliteNewsRepository::in_memory()?;

    repository.save_news_batch(&input)?;
    let stored = repository.list_news()?;

    assert_eq!(stored.len(), 240);
    assert_eq!(stored, expected);

    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 13, 16, 0, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };
    let report = render_news_json(&stored, &context)?;
    let parsed: JsonReport = serde_json::from_str(&report)?;

    assert_eq!(parsed.meta.item_count, expected.len());
    assert_eq!(parsed.items, expected);
    assert_eq!(parsed.items.first().map(|item| item.rank), Some(1));
    assert_eq!(parsed.items.last().map(|item| item.rank), Some(20));
    Ok(())
}

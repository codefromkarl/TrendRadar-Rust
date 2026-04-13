use anyhow::Result;
use chrono::TimeZone;
use serde::Deserialize;
use std::collections::BTreeMap;
use trendradar_domain::{NewsItem, RunContext};
use trendradar_report::{
    render_news_html, render_news_json, render_news_markdown, render_news_table,
};
use trendradar_storage::{NewsRepository, SqliteNewsRepository};

fn build_large_input() -> Vec<NewsItem> {
    let sources = ["weibo", "zhihu", "bilibili", "baidu", "toutiao", "pengpai"];
    let mut items = Vec::new();

    for source in sources {
        for idx in 0..36_u32 {
            let title = format!("Large Topic {idx:03}");
            let base_rank = ((idx * 5 + source.len() as u32) % 18) + 1;
            items.push(NewsItem {
                title: title.clone(),
                source_id: source.to_owned(),
                rank: base_rank,
            });

            if idx % 3 == 0 {
                items.push(NewsItem {
                    title: title.clone(),
                    source_id: source.to_owned(),
                    rank: base_rank + 5,
                });
            }

            if idx % 8 == 0 {
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
fn large_input_produces_consistent_multi_format_reports() -> Result<()> {
    let input = build_large_input();
    let expected = expected_stable_items(&input);
    let mut repository = SqliteNewsRepository::in_memory()?;

    repository.save_news_batch(&input)?;
    let stored = repository.list_news()?;
    assert_eq!(stored, expected);

    let context = RunContext {
        started_at: chrono::Utc
            .with_ymd_and_hms(2026, 4, 13, 18, 0, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?,
        timezone: "Asia/Shanghai".to_owned(),
    };

    let json = render_news_json(&stored, &context)?;
    let parsed: JsonReport = serde_json::from_str(&json)?;
    let html = render_news_html(&stored, &context);
    let table = render_news_table(&stored, &context);
    let markdown = render_news_markdown(&stored, &context);

    assert_eq!(parsed.meta.item_count, stored.len());
    assert_eq!(parsed.items, stored);

    let first = stored
        .first()
        .ok_or_else(|| anyhow::anyhow!("stored items should not be empty"))?;
    let last = stored
        .last()
        .ok_or_else(|| anyhow::anyhow!("stored items should not be empty"))?;

    assert!(html.contains(&first.title));
    assert!(html.contains(&last.title));
    assert!(table.contains(&first.title));
    assert!(table.contains(&last.title));
    assert!(markdown.contains(&first.title));
    assert!(markdown.contains(&last.title));

    assert!(markdown.contains("| # | 标题 | 来源 |"));
    assert!(table.contains("TrendRadar Report"));
    Ok(())
}

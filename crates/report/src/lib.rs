//! 输出层骨架。

use serde::Serialize;
use trendradar_domain::{NewsItem, RunContext};

#[derive(Debug, Serialize)]
struct ReportMetadata<'a> {
    started_at: chrono::DateTime<chrono::Utc>,
    timezone: &'a str,
    item_count: usize,
}

#[derive(Debug, Serialize)]
struct NewsReport<'a> {
    meta: ReportMetadata<'a>,
    items: &'a [NewsItem],
}

/// 将新闻列表渲染为 JSON。
pub fn render_news_json(items: &[NewsItem], context: &RunContext) -> serde_json::Result<String> {
    let report = NewsReport {
        meta: ReportMetadata {
            started_at: context.started_at,
            timezone: &context.timezone,
            item_count: items.len(),
        },
        items,
    };
    serde_json::to_string_pretty(&report)
}

#[cfg(test)]
mod tests {
    use super::render_news_json;
    use chrono::TimeZone;
    use std::error::Error;
    use std::fs::read_to_string;
    use trendradar_domain::{NewsItem, RunContext};

    #[test]
    fn render_news_json_includes_run_metadata_and_items() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/report/news-report-input.json"
        );
        let contents = read_to_string(fixture_path)?;
        let items: Vec<NewsItem> = serde_json::from_str(&contents)?;
        let context = RunContext {
            started_at: chrono::Utc
                .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
                .single()
                .ok_or("invalid fixed timestamp")?,
            timezone: "Asia/Shanghai".to_owned(),
        };

        let rendered = render_news_json(&items, &context)?;
        let value: serde_json::Value = serde_json::from_str(&rendered)?;

        assert_eq!(value["meta"]["timezone"], "Asia/Shanghai");
        assert_eq!(value["meta"]["item_count"], 2);
        assert_eq!(value["items"][0]["title"], "Rust 1.85.0 released");
        assert_eq!(value["items"][1]["rank"], 12);
        Ok(())
    }
}

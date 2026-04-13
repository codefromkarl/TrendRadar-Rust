//! 输出层：JSON 与 HTML 报告渲染。

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

/// 将新闻列表渲染为自包含 HTML5 文件。
///
/// 输出包含内联 CSS 样式的完整 HTML 页面，无需外部依赖即可在浏览器中展示。
pub fn render_news_html(items: &[NewsItem], context: &RunContext) -> String {
    let timestamp = context.started_at.to_rfc3339();
    let rows: Vec<String> = items
        .iter()
        .map(|item| {
            let rank = html_escape(&item.rank.to_string());
            let title = html_escape(&item.title);
            let source_id = html_escape(&item.source_id);
            format!("<tr><td>{rank}</td><td>{title}</td><td>{source_id}</td></tr>")
        })
        .collect();

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>TrendRadar Report</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; margin: 2rem auto; max-width: 960px; color: #1a1a1a; background: #fafafa; }}
h1 {{ font-size: 1.5rem; margin-bottom: 0.25rem; }}
.meta {{ color: #666; font-size: 0.875rem; margin-bottom: 1.5rem; }}
table {{ width: 100%; border-collapse: collapse; background: #fff; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }}
th {{ background: #f5f5f5; text-align: left; padding: 0.5rem 0.75rem; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.05em; border-bottom: 2px solid #e0e0e0; }}
td {{ padding: 0.5rem 0.75rem; border-bottom: 1px solid #eee; font-size: 0.9rem; }}
tr:hover {{ background: #f9f9f9; }}
td:first-child {{ font-weight: 600; color: #e63946; width: 3rem; text-align: center; }}
td:last-child {{ color: #888; font-size: 0.8rem; }}
.empty {{ text-align: center; padding: 2rem; color: #999; }}
</style>
</head>
<body>
<h1>TrendRadar Report</h1>
<div class="meta">{timezone} &middot; {timestamp} &middot; {count} items</div>
<table>
<thead><tr><th>Rank</th><th>Title</th><th>Source</th></tr></thead>
<tbody>
{body}
</tbody>
</table>
</body>
</html>"#,
        timezone = html_escape(&context.timezone),
        timestamp = html_escape(&timestamp),
        count = items.len(),
        body = if items.is_empty() {
            r#"<tr><td colspan="3" class="empty">No items found</td></tr>"#.to_owned()
        } else {
            rows.join("")
        },
    )
}

/// 将新闻列表渲染为终端彩色表格。
///
/// 使用 `comfy-table` 生成彩色终端表格，前三名使用红色高亮显示。
/// 标题超过 80 字符会被截断。空数据时显示 "No items found" 行。
pub fn render_news_table(items: &[NewsItem], context: &RunContext) -> String {
    use comfy_table::modifiers::UTF8_ROUND_CORNERS;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::{Cell, Color, ContentArrangement, Table};

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["排名", "标题", "来源"]);

    if items.is_empty() {
        table.add_row(vec![
            Cell::new("No items found").fg(Color::DarkGrey),
            Cell::new(""),
            Cell::new(""),
        ]);
    } else {
        for item in items {
            let rank_cell = if item.rank <= 3 {
                Cell::new(item.rank).fg(Color::Red)
            } else {
                Cell::new(item.rank)
            };

            let title = if item.title.len() > 80 {
                format!("{}...", &item.title[..77])
            } else {
                item.title.clone()
            };

            table.add_row(vec![
                rank_cell,
                Cell::new(title),
                Cell::new(&item.source_id),
            ]);
        }
    }

    let timestamp = context
        .started_at
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    let header = format!("TrendRadar Report — {} @ {}", context.timezone, timestamp);

    format!("{header}\n{table}")
}

/// 将新闻列表渲染为 GFM Markdown 表格。
///
/// 生成标准的 GitHub Flavored Markdown 表格格式，包含标题行、元数据和表格数据。
/// 空数据时输出 "No items found." 文本。
pub fn render_news_markdown(items: &[NewsItem], context: &RunContext) -> String {
    let timestamp = context
        .started_at
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();

    let mut output = String::new();
    output.push_str("## TrendRadar Report\n\n");
    output.push_str(&format!("**Timezone**: {}\n", context.timezone));
    output.push_str(&format!("**Generated**: {}\n\n", timestamp));

    if items.is_empty() {
        output.push_str("No items found.\n");
    } else {
        output.push_str("| # | 标题 | 来源 |\n");
        output.push_str("|---|------|------|\n");

        for item in items {
            let title = escape_markdown_table(&item.title);
            let source = escape_markdown_table(&item.source_id);
            output.push_str(&format!("| {} | {} | {} |\n", item.rank, title, source));
        }
    }

    output
}

/// 转义 Markdown 表格特殊字符。
fn escape_markdown_table(input: &str) -> String {
    input.replace('|', "\\|").replace('\n', " ")
}

/// HTML 特殊字符转义。
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::{
        escape_markdown_table, render_news_html, render_news_json, render_news_markdown,
        render_news_table,
    };
    use chrono::TimeZone;
    use std::error::Error;
    use std::fs::read_to_string;
    use trendradar_domain::{NewsItem, RunContext};

    fn test_context() -> Result<RunContext, Box<dyn Error>> {
        Ok(RunContext {
            started_at: chrono::Utc
                .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
                .single()
                .ok_or("invalid fixed timestamp")?,
            timezone: "Asia/Shanghai".to_owned(),
        })
    }

    #[test]
    fn render_news_json_includes_run_metadata_and_items() -> Result<(), Box<dyn Error>> {
        let fixture_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/system/report/news-report-input.json"
        );
        let contents = read_to_string(fixture_path)?;
        let items: Vec<NewsItem> = serde_json::from_str(&contents)?;
        let context = test_context()?;

        let rendered = render_news_json(&items, &context)?;
        let value: serde_json::Value = serde_json::from_str(&rendered)?;

        assert_eq!(value["meta"]["timezone"], "Asia/Shanghai");
        assert_eq!(value["meta"]["item_count"], 2);
        assert_eq!(value["items"][0]["title"], "Rust 1.85.0 released");
        assert_eq!(value["items"][1]["rank"], 12);
        Ok(())
    }

    #[test]
    fn render_news_json_keeps_empty_items_shape() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;

        let rendered = render_news_json(&[], &context)?;
        let value: serde_json::Value = serde_json::from_str(&rendered)?;

        assert_eq!(value["meta"]["timezone"], "Asia/Shanghai");
        assert_eq!(value["meta"]["item_count"], 0);
        assert_eq!(value["items"], serde_json::json!([]));
        Ok(())
    }

    // -- HTML report tests --

    #[test]
    fn render_news_html_produces_valid_html5() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let items = vec![NewsItem {
            title: "Test item".to_owned(),
            source_id: "test".to_owned(),
            rank: 1,
        }];

        let html = render_news_html(&items, &context);

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"zh-CN\">"));
        assert!(html.contains("</html>"));
        Ok(())
    }

    #[test]
    fn render_news_html_includes_item_data() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let items = vec![NewsItem {
            title: "Rust 1.85 released".to_owned(),
            source_id: "github-trending".to_owned(),
            rank: 3,
        }];

        let html = render_news_html(&items, &context);

        assert!(html.contains("Rust 1.85 released"));
        assert!(html.contains("github-trending"));
        assert!(html.contains(">3<"));
        Ok(())
    }

    #[test]
    fn render_news_html_escapes_special_characters() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let items = vec![NewsItem {
            title: "A <b>bold</b> & \"quoted\" thing".to_owned(),
            source_id: "test".to_owned(),
            rank: 1,
        }];

        let html = render_news_html(&items, &context);

        assert!(html.contains("&lt;b&gt;bold&lt;/b&gt;"));
        assert!(html.contains("&amp;"));
        assert!(html.contains("&quot;quoted&quot;"));
        assert!(!html.contains("<b>bold</b>"));
        Ok(())
    }

    #[test]
    fn render_news_html_includes_metadata() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let items = vec![NewsItem {
            title: "Test".to_owned(),
            source_id: "a".to_owned(),
            rank: 1,
        }];

        let html = render_news_html(&items, &context);

        assert!(html.contains("Asia/Shanghai"));
        assert!(html.contains("1 items"));
        assert!(html.contains("2026-04-11"));
        Ok(())
    }

    #[test]
    fn render_news_html_shows_empty_state() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let html = render_news_html(&[], &context);

        assert!(html.contains("No items found"));
        assert!(html.contains("0 items"));
        assert!(html.starts_with("<!DOCTYPE html>"));
        Ok(())
    }

    // -- Table report tests --

    #[test]
    fn render_news_table_produces_output_with_items() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let items = vec![
            NewsItem {
                title: "First item".to_owned(),
                source_id: "source-a".to_owned(),
                rank: 1,
            },
            NewsItem {
                title: "Second item".to_owned(),
                source_id: "source-b".to_owned(),
                rank: 2,
            },
        ];

        let table = render_news_table(&items, &context);

        assert!(table.contains("TrendRadar Report"));
        assert!(table.contains("Asia/Shanghai"));
        assert!(table.contains("First item"));
        assert!(table.contains("source-a"));
        assert!(table.contains("排名"));
        Ok(())
    }

    #[test]
    fn render_news_table_shows_empty_state() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let table = render_news_table(&[], &context);

        assert!(table.contains("No items found"));
        assert!(table.contains("TrendRadar Report"));
        Ok(())
    }

    #[test]
    fn render_news_table_truncates_long_title() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let long_title = "A".repeat(100);
        let items = vec![NewsItem {
            title: long_title.clone(),
            source_id: "test".to_owned(),
            rank: 1,
        }];

        let table = render_news_table(&items, &context);

        // Should be truncated to 80 chars + "..."
        assert!(table.contains("..."));
        assert!(!table.contains(&long_title)); // Full title should not be present
        Ok(())
    }

    // -- Markdown report tests --

    #[test]
    fn render_news_markdown_produces_gfm_table() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let items = vec![NewsItem {
            title: "Test item".to_owned(),
            source_id: "test-source".to_owned(),
            rank: 5,
        }];

        let markdown = render_news_markdown(&items, &context);

        assert!(markdown.contains("## TrendRadar Report"));
        assert!(markdown.contains("| # | 标题 | 来源 |"));
        assert!(markdown.contains("|---|------|------|"));
        assert!(markdown.contains("| 5 | Test item | test-source |"));
        Ok(())
    }

    #[test]
    fn render_news_markdown_includes_metadata() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let items = vec![NewsItem {
            title: "Item".to_owned(),
            source_id: "src".to_owned(),
            rank: 1,
        }];

        let markdown = render_news_markdown(&items, &context);

        assert!(markdown.contains("**Timezone**: Asia/Shanghai"));
        assert!(markdown.contains("**Generated**:"));
        assert!(markdown.contains("2026-04-11"));
        Ok(())
    }

    #[test]
    fn render_news_markdown_shows_empty_state() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let markdown = render_news_markdown(&[], &context);

        assert!(markdown.contains("No items found."));
        assert!(markdown.contains("## TrendRadar Report"));
        assert!(!markdown.contains("| # |"));
        Ok(())
    }

    #[test]
    fn render_news_markdown_escapes_pipe_characters() -> Result<(), Box<dyn Error>> {
        let context = test_context()?;
        let items = vec![NewsItem {
            title: "Title | with | pipes".to_owned(),
            source_id: "source|with|pipes".to_owned(),
            rank: 1,
        }];

        let markdown = render_news_markdown(&items, &context);

        // Pipes should be escaped with backslash
        assert!(markdown.contains("\\|"));
        Ok(())
    }

    #[test]
    fn escape_markdown_table_handles_special_chars() -> Result<(), Box<dyn Error>> {
        assert_eq!(escape_markdown_table("a|b"), "a\\|b");
        assert_eq!(escape_markdown_table("line1\nline2"), "line1 line2");
        assert_eq!(escape_markdown_table("normal"), "normal");
        Ok(())
    }
}

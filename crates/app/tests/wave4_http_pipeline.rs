//! HTTP pipeline 集成测试。

use chrono::TimeZone;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use trendradar_app::run_config_pipeline;
use trendradar_config::load_config_from_json_str;

fn config_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/system/config")
        .join(name)
}

#[test]
fn config_pipeline_fetches_rss_and_hotlist_via_http() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new();

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item><title>Rust 1.85 released</title></item>
  </channel>
</rss>"#;

    let rss_mock = server
        .mock("GET", "/rss.xml")
        .with_status(200)
        .with_body(rss_xml)
        .create();

    let hotlist_body = r#"[{"title":"Breaking news","rank":1}]"#;
    let hotlist_mock = server
        .mock("GET", "/hotlist")
        .with_status(200)
        .with_body(hotlist_body)
        .create();

    let config_json = format!(
        r#"{{
            "timezone":"Asia/Shanghai",
            "rss_feeds":[{{"source_id":"test-blog","url":"{base}/rss.xml"}}],
            "hotlist_apis":[{{"platform_id":"test-platform","url":"{base}/hotlist"}}]
        }}"#,
        base = server.url()
    );
    let config = load_config_from_json_str(&config_json)?;

    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 10, 0, 0)
        .single()
        .ok_or("invalid timestamp")?;

    let result = run_config_pipeline(&config, started_at, None)?;

    rss_mock.assert();
    hotlist_mock.assert();

    // 1 RSS + 1 hotlist = 2 items collected
    assert_eq!(result.collected_items.len(), 2);
    assert_eq!(result.stored_items.len(), 2);
    assert!(result.report_json.is_some());

    let report = result.report_json.as_ref().ok_or("missing report")?;
    assert!(report.contains("Rust 1.85 released"));
    assert!(report.contains("Breaking news"));
    Ok(())
}

#[test]
fn config_pipeline_with_empty_sources_produces_empty_report() -> Result<(), Box<dyn Error>> {
    let config_json = r#"{"timezone":"Asia/Shanghai"}"#;
    let config = load_config_from_json_str(config_json)?;

    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 10, 0, 0)
        .single()
        .ok_or("invalid timestamp")?;

    let result = run_config_pipeline(&config, started_at, None)?;

    assert!(result.collected_items.is_empty());
    assert!(result.report_json.is_some());

    let report = result.report_json.as_ref().ok_or("missing report")?;
    assert!(report.contains("\"items\": []"));
    Ok(())
}

#[test]
fn config_pipeline_routes_multi_platform_hotlist_parsers() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new();

    let toutiao_mock = server
        .mock("GET", "/toutiao")
        .with_status(200)
        .with_body(
            r#"{
                "data": [
                    {"ClusterIdStr": "1", "Title": "头条测试", "HotValue": "123"}
                ]
            }"#,
        )
        .create();

    let baidu_mock = server
        .mock("GET", "/baidu")
        .with_status(200)
        .with_body(
            r#"<!doctype html><!--s-data:{"data":{"cards":[{"content":[{"word":"百度测试","rawUrl":"https://example.com/baidu"}]}]}}-->"#,
        )
        .create();

    let pengpai_mock = server
        .mock("GET", "/pengpai")
        .with_status(200)
        .with_body(
            r#"{
                "data": {
                    "hotNews": [
                        {"contId": "2", "name": "澎湃测试", "pubTimeLong": "1710000000000"}
                    ]
                }
            }"#,
        )
        .create();

    let cls_mock = server
        .mock("GET", "/cls")
        .with_status(200)
        .with_body(
            r#"{
                "data": [
                    {
                        "id": 3,
                        "brief": "财联社测试",
                        "shareurl": "https://example.com/cls",
                        "ctime": 1710000000,
                        "is_ad": 0
                    }
                ]
            }"#,
        )
        .create();

    let fixture = fs::read_to_string(config_fixture_path("hotlist-multi-platform-http.json"))?;
    let config_json = fixture.replace("__BASE_URL__", &server.url());
    let config = load_config_from_json_str(&config_json)?;

    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 13, 10, 0, 0)
        .single()
        .ok_or("invalid timestamp")?;

    let result = run_config_pipeline(&config, started_at, None)?;

    toutiao_mock.assert();
    baidu_mock.assert();
    pengpai_mock.assert();
    cls_mock.assert();

    assert_eq!(result.collected_items.len(), 4);
    assert_eq!(result.stored_items.len(), 4);

    let titles: Vec<&str> = result
        .collected_items
        .iter()
        .map(|item| item.title.as_str())
        .collect();
    assert!(titles.contains(&"头条测试"));
    assert!(titles.contains(&"百度测试"));
    assert!(titles.contains(&"澎湃测试"));
    assert!(titles.contains(&"财联社测试"));
    Ok(())
}

#[test]
fn config_pipeline_propagates_http_error() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new();

    server.mock("GET", "/fail").with_status(500).create();

    let config_json = format!(
        r#"{{
            "timezone":"Asia/Shanghai",
            "rss_feeds":[{{"source_id":"broken","url":"{base}/fail"}}]
        }}"#,
        base = server.url()
    );
    let config = load_config_from_json_str(&config_json)?;

    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 10, 0, 0)
        .single()
        .ok_or("invalid timestamp")?;

    // HTTP errors are now handled gracefully (logged, skipped) rather than propagated.
    // The pipeline should succeed but with empty collected items.
    let result = run_config_pipeline(&config, started_at, None)?;

    assert!(result.collected_items.is_empty());
    Ok(())
}

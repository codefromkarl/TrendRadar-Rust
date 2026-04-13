//! HTTP pipeline 集成测试。

use chrono::TimeZone;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use trendradar_app::{OutputMode, run_config_pipeline, run_config_pipeline_with_output};
use trendradar_config::load_config_from_json_str;

#[derive(Clone)]
struct ResponseSpec {
    status: u16,
    content_type: &'static str,
    body: &'static str,
    delay: Duration,
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

fn start_test_server(
    routes: HashMap<&'static str, ResponseSpec>,
    expected_requests: usize,
) -> Result<
    (
        String,
        thread::JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>,
    ),
    Box<dyn Error>,
> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let base_url = format!("http://{}", listener.local_addr()?);

    let handle = thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut workers = Vec::new();

        for _ in 0..expected_requests {
            let (mut stream, _) = listener.accept()?;
            let routes = routes.clone();
            workers.push(thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
                let mut buffer = [0_u8; 4096];
                let read = stream.read(&mut buffer)?;
                let request = String::from_utf8_lossy(&buffer[..read]);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .ok_or("missing request path")?;
                let response = routes.get(path).ok_or("unexpected request path")?.clone();

                thread::sleep(response.delay);

                let body = response.body.as_bytes();
                let head = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response.status,
                    reason_phrase(response.status),
                    response.content_type,
                    body.len()
                );
                stream.write_all(head.as_bytes())?;
                stream.write_all(body)?;
                stream.flush()?;
                Ok(())
            }));
        }

        for worker in workers {
            worker.join().map_err(|_| "response worker panicked")??;
        }

        Ok(())
    });

    Ok((base_url, handle))
}

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
fn config_pipeline_keeps_successful_http_sources_when_one_source_fails()
-> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new();

    let rss_mock = server
        .mock("GET", "/rss.xml")
        .with_status(200)
        .with_body(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item><title>Rust resilient success</title></item>
  </channel>
</rss>"#,
        )
        .create();

    let hotlist_ok_mock = server
        .mock("GET", "/toutiao")
        .with_status(200)
        .with_body(
            r#"{
                "data": [
                    {"ClusterIdStr": "1", "Title": "头条成功项", "HotValue": "100"}
                ]
            }"#,
        )
        .create();

    let hotlist_fail_mock = server
        .mock("GET", "/broken-hotlist")
        .with_status(500)
        .create();

    let config_json = format!(
        r#"{{
            "timezone":"Asia/Shanghai",
            "rss_feeds":[{{"source_id":"test-blog","url":"{base}/rss.xml"}}],
            "hotlist_apis":[
                {{"platform_id":"toutiao","url":"{base}/toutiao","source_type":"toutiao"}},
                {{"platform_id":"broken","url":"{base}/broken-hotlist","source_type":"generic"}}
            ]
        }}"#,
        base = server.url()
    );
    let config = load_config_from_json_str(&config_json)?;

    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 13, 10, 0, 0)
        .single()
        .ok_or("invalid timestamp")?;

    let result = run_config_pipeline(&config, started_at, None)?;

    rss_mock.assert();
    hotlist_ok_mock.assert();
    hotlist_fail_mock.assert();

    assert_eq!(result.collected_items.len(), 2);
    assert_eq!(result.stored_items.len(), 2);

    let report = result.report_json.as_ref().ok_or("missing report")?;
    assert!(report.contains("Rust resilient success"));
    assert!(report.contains("头条成功项"));
    Ok(())
}

#[test]
fn config_pipeline_skips_invalid_new_platform_payload_but_keeps_other_sources()
-> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new();

    let baidu_mock = server
        .mock("GET", "/baidu")
        .with_status(200)
        .with_body(
            r#"<!doctype html><!--s-data:{"data":{"cards":[{"content":[{"word":"百度保留项","rawUrl":"https://example.com/baidu"}]}]}}-->"#,
        )
        .create();

    let pengpai_bad_mock = server
        .mock("GET", "/pengpai")
        .with_status(200)
        .with_body(r#"{"data":{"hotNews":[{"contId":"2"}]}}"#)
        .create();

    let rss_mock = server
        .mock("GET", "/rss.xml")
        .with_status(200)
        .with_body(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item><title>RSS 保留项</title></item>
  </channel>
</rss>"#,
        )
        .create();

    let config_json = format!(
        r#"{{
            "timezone":"Asia/Shanghai",
            "rss_feeds":[{{"source_id":"test-blog","url":"{base}/rss.xml"}}],
            "hotlist_apis":[
                {{"platform_id":"baidu","url":"{base}/baidu","source_type":"baidu"}},
                {{"platform_id":"pengpai","url":"{base}/pengpai","source_type":"pengpai"}}
            ]
        }}"#,
        base = server.url()
    );
    let config = load_config_from_json_str(&config_json)?;

    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 13, 10, 0, 0)
        .single()
        .ok_or("invalid timestamp")?;

    let result = run_config_pipeline(&config, started_at, None)?;

    baidu_mock.assert();
    pengpai_bad_mock.assert();
    rss_mock.assert();

    assert_eq!(result.collected_items.len(), 2);

    let titles: Vec<&str> = result
        .collected_items
        .iter()
        .map(|item| item.title.as_str())
        .collect();
    assert!(titles.contains(&"百度保留项"));
    assert!(titles.contains(&"RSS 保留项"));
    assert!(!titles.contains(&"澎湃测试"));
    Ok(())
}

#[test]
fn config_pipeline_keeps_output_stable_under_complex_concurrent_slow_and_failed_sources()
-> Result<(), Box<dyn Error>> {
    let mut routes = HashMap::new();
    routes.insert(
        "/rss-slow",
        ResponseSpec {
            status: 200,
            content_type: "application/xml",
            body: r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Slow Feed</title>
    <item><title>Alpha RSS success</title></item>
  </channel>
</rss>"#,
            delay: Duration::from_millis(180),
        },
    );
    routes.insert(
        "/baidu-fast",
        ResponseSpec {
            status: 200,
            content_type: "application/json",
            body: r#"<!doctype html><!--s-data:{"data":{"cards":[{"content":[{"word":"Bravo Baidu success","rawUrl":"https://example.com/baidu"}]}]}}-->"#,
            delay: Duration::from_millis(20),
        },
    );
    routes.insert(
        "/toutiao-slow",
        ResponseSpec {
            status: 200,
            content_type: "application/json",
            body: r#"{
                "data": [
                    {"ClusterIdStr": "1", "Title": "Charlie Toutiao success", "HotValue": "100"}
                ]
            }"#,
            delay: Duration::from_millis(140),
        },
    );
    routes.insert(
        "/broken-1",
        ResponseSpec {
            status: 500,
            content_type: "application/json",
            body: r#"{"error":"boom"}"#,
            delay: Duration::from_millis(15),
        },
    );
    routes.insert(
        "/broken-2",
        ResponseSpec {
            status: 500,
            content_type: "application/json",
            body: r#"{"error":"boom"}"#,
            delay: Duration::from_millis(30),
        },
    );

    let (base_url, handle) = start_test_server(routes, 5)?;
    let config_json = format!(
        r#"{{
            "timezone":"Asia/Shanghai",
            "rss_feeds":[{{"source_id":"slow-rss","url":"{base}/rss-slow"}}],
            "hotlist_apis":[
                {{"platform_id":"baidu","url":"{base}/baidu-fast","source_type":"baidu"}},
                {{"platform_id":"toutiao","url":"{base}/toutiao-slow","source_type":"toutiao"}},
                {{"platform_id":"broken-1","url":"{base}/broken-1","source_type":"generic"}},
                {{"platform_id":"broken-2","url":"{base}/broken-2","source_type":"generic"}}
            ]
        }}"#,
        base = base_url
    );
    let config = load_config_from_json_str(&config_json)?;

    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 13, 17, 0, 0)
        .single()
        .ok_or("invalid timestamp")?;

    let result = run_config_pipeline_with_output(&config, started_at, None, OutputMode::Markdown)?;
    handle
        .join()
        .map_err(|_| "test server panicked")?
        .map_err(|e| e as Box<dyn Error>)?;

    let stored_titles: Vec<&str> = result
        .stored_items
        .iter()
        .map(|item| item.title.as_str())
        .collect();
    assert_eq!(
        stored_titles,
        vec![
            "Bravo Baidu success",
            "Alpha RSS success",
            "Charlie Toutiao success"
        ]
    );

    let markdown = result
        .report_markdown
        .as_ref()
        .ok_or("missing markdown report")?;
    let bravo = markdown
        .find("Bravo Baidu success")
        .ok_or("missing bravo")?;
    let alpha = markdown.find("Alpha RSS success").ok_or("missing alpha")?;
    let charlie = markdown
        .find("Charlie Toutiao success")
        .ok_or("missing charlie")?;
    assert!(bravo < alpha && alpha < charlie);
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

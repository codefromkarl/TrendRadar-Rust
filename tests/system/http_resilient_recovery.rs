use anyhow::{Context, Result};
use chrono::TimeZone;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use trendradar_app::run_config_pipeline;
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
) -> Result<(String, thread::JoinHandle<Result<()>>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let base_url = format!("http://{}", listener.local_addr()?);
    let routes = Arc::new(routes);

    let handle = thread::spawn(move || -> Result<()> {
        let mut workers = Vec::new();
        for _ in 0..expected_requests {
            let (mut stream, _) = listener.accept()?;
            let routes = Arc::clone(&routes);
            workers.push(thread::spawn(move || -> Result<()> {
                let mut buffer = [0_u8; 2048];
                let read = stream.read(&mut buffer)?;
                let request = String::from_utf8_lossy(&buffer[..read]);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .context("missing request path")?;
                let response = routes
                    .get(path)
                    .with_context(|| format!("unexpected request path: {path}"))?
                    .clone();

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
            worker
                .join()
                .map_err(|_| anyhow::anyhow!("response worker panicked"))??;
        }

        Ok(())
    });

    Ok((base_url, handle))
}

#[test]
fn config_pipeline_retains_slow_successes_when_multiple_http_sources_fail() -> Result<()> {
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
    <item><title>RSS slow success</title></item>
  </channel>
</rss>"#,
            delay: Duration::from_millis(180),
        },
    );
    routes.insert(
        "/toutiao-slow",
        ResponseSpec {
            status: 200,
            content_type: "application/json",
            body: r#"{
                "data": [
                    {"ClusterIdStr": "1", "Title": "头条慢源成功项", "HotValue": "100"}
                ]
            }"#,
            delay: Duration::from_millis(160),
        },
    );
    routes.insert(
        "/broken-1",
        ResponseSpec {
            status: 500,
            content_type: "application/json",
            body: r#"{"error":"boom"}"#,
            delay: Duration::from_millis(10),
        },
    );
    routes.insert(
        "/broken-2",
        ResponseSpec {
            status: 500,
            content_type: "application/json",
            body: r#"{"error":"boom"}"#,
            delay: Duration::from_millis(20),
        },
    );

    let (base_url, handle) = start_test_server(routes, 4)?;
    let config_json = format!(
        r#"{{
            "timezone":"Asia/Shanghai",
            "rss_feeds":[{{"source_id":"slow-rss","url":"{base}/rss-slow"}}],
            "hotlist_apis":[
                {{"platform_id":"toutiao","url":"{base}/toutiao-slow","source_type":"toutiao"}},
                {{"platform_id":"broken-1","url":"{base}/broken-1","source_type":"generic"}},
                {{"platform_id":"broken-2","url":"{base}/broken-2","source_type":"generic"}}
            ]
        }}"#,
        base = base_url
    );
    let config = load_config_from_json_str(&config_json)?;

    let started_at = chrono::Utc
        .with_ymd_and_hms(2026, 4, 13, 16, 30, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;

    let result = run_config_pipeline(&config, started_at, None)?;
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("test server panicked"))??;

    assert_eq!(result.collected_items.len(), 2);
    assert_eq!(result.stored_items.len(), 2);

    let stored_sources: Vec<&str> = result
        .stored_items
        .iter()
        .map(|item| item.source_id.as_str())
        .collect();
    assert_eq!(stored_sources, vec!["slow-rss", "toutiao"]);

    let report = result.report_json.as_ref().context("missing report_json")?;
    assert!(report.contains("RSS slow success"));
    assert!(report.contains("头条慢源成功项"));
    Ok(())
}

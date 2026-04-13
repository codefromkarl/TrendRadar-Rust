#![allow(clippy::expect_used, missing_docs)]
//! TrendRadar app pipeline benchmark baseline.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::TimeZone;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use trendradar_analyze::{filter_by_keywords, group_news_by_source, rank_news};
use trendradar_app::{FixtureSource, PipelineResult, run_fixture_pipeline};
use trendradar_config::{AppConfig, load_config_from_json_str};
use trendradar_domain::RunContext;
use trendradar_fetch::{Fetcher, FixtureHotlistFetcher, FixtureRssFetcher};
use trendradar_report::{
    render_news_html, render_news_json, render_news_markdown, render_news_table,
};
use trendradar_storage::{NewsRepository, SqliteNewsRepository};

fn system_fixture_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/system")
        .join(relative_path)
}

fn fixed_started_at() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc
        .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
        .single()
        .expect("fixed benchmark timestamp must be valid")
}

fn load_fixture_config() -> AppConfig {
    let config_fixture = fs::read_to_string(system_fixture_path("config/minimal-valid.json"))
        .expect("fixture config must exist");
    load_config_from_json_str(&config_fixture).expect("fixture config must parse")
}

fn load_fixture_sources(config: &AppConfig) -> Vec<FixtureSource> {
    vec![
        FixtureSource::hotlist(
            config
                .platforms
                .first()
                .expect("fixture config must contain at least one platform")
                .clone(),
            system_fixture_path("fetch/hotlist-weibo.json"),
        ),
        FixtureSource::rss("rust-blog", system_fixture_path("fetch/rss-rust-blog.json")),
    ]
}

fn load_fixture_items() -> Vec<trendradar_domain::NewsItem> {
    let hotlist_fetcher = FixtureHotlistFetcher::new(
        "weibo".to_owned(),
        system_fixture_path("fetch/hotlist-weibo.json"),
    );
    let rss_fetcher =
        FixtureRssFetcher::new("rust-blog", system_fixture_path("fetch/rss-rust-blog.json"));

    let hotlist_items = hotlist_fetcher.fetch().expect("hotlist fixture must parse");
    let rss_items = rss_fetcher.fetch().expect("rss fixture must parse");

    hotlist_items.into_iter().chain(rss_items).collect()
}

fn benchmark_fixture_pipeline(criterion: &mut Criterion) {
    let config = load_fixture_config();
    let sources = load_fixture_sources(&config);
    let started_at = fixed_started_at();

    let mut group = criterion.benchmark_group("pipeline_total");
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("fixture_pipeline_minimal", |bencher| {
        bencher.iter(|| {
            let result = run_fixture_pipeline(&config, started_at, &sources)
                .expect("fixture pipeline benchmark must succeed");
            consume_pipeline_result(&result);
        });
    });
    group.finish();
}

fn benchmark_fetch_stage(criterion: &mut Criterion) {
    let hotlist_fetcher = FixtureHotlistFetcher::new(
        "weibo".to_owned(),
        system_fixture_path("fetch/hotlist-weibo.json"),
    );
    let rss_fetcher =
        FixtureRssFetcher::new("rust-blog", system_fixture_path("fetch/rss-rust-blog.json"));

    let mut group = criterion.benchmark_group("pipeline_stage");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("fetch_fixture_sources", |bencher| {
        bencher.iter(|| {
            let hotlist = hotlist_fetcher.fetch().expect("hotlist fetch must succeed");
            let rss = rss_fetcher.fetch().expect("rss fetch must succeed");
            black_box(hotlist.len() + rss.len());
        });
    });
    group.finish();
}

fn benchmark_analyze_stage(criterion: &mut Criterion) {
    let items = load_fixture_items();
    let keywords = vec!["rust".to_owned(), "trend".to_owned()];

    let mut group = criterion.benchmark_group("pipeline_stage");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("analyze_filter_rank_group", |bencher| {
        bencher.iter(|| {
            let filtered = filter_by_keywords(&items, &keywords);
            let ranked = rank_news(&filtered);
            let grouped = group_news_by_source(&filtered);
            black_box((filtered.len(), ranked.len(), grouped.len()));
        });
    });
    group.finish();
}

fn benchmark_storage_stage(criterion: &mut Criterion) {
    let items = load_fixture_items();

    let mut group = criterion.benchmark_group("pipeline_stage");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("storage_in_memory_roundtrip", |bencher| {
        bencher.iter(|| {
            let mut repository =
                SqliteNewsRepository::in_memory().expect("in-memory sqlite must initialize");
            repository
                .save_news_batch(&items)
                .expect("fixture batch must store");
            let stored = repository.list_news().expect("stored items must list");
            black_box(stored.len());
        });
    });
    group.finish();
}

fn benchmark_report_stage(criterion: &mut Criterion) {
    let items = load_fixture_items();
    let context = RunContext {
        started_at: fixed_started_at(),
        timezone: "Asia/Shanghai".to_owned(),
    };

    let mut group = criterion.benchmark_group("pipeline_stage");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("report_render_all_formats", |bencher| {
        bencher.iter(|| {
            let json = render_news_json(&items, &context).expect("json render must succeed");
            let html = render_news_html(&items, &context);
            let table = render_news_table(&items, &context);
            let markdown = render_news_markdown(&items, &context);
            black_box((json.len(), html.len(), table.len(), markdown.len()));
        });
    });
    group.finish();
}

fn consume_pipeline_result(result: &PipelineResult) {
    black_box((
        result.collected_items.len(),
        result.filtered_items.len(),
        result.ranked_items.len(),
        result.source_summaries.len(),
        result.stored_items.len(),
        result.report_json.as_ref().map(std::string::String::len),
        result.report_html.as_ref().map(std::string::String::len),
        result.report_table.as_ref().map(std::string::String::len),
        result
            .report_markdown
            .as_ref()
            .map(std::string::String::len),
    ));
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(20).warm_up_time(Duration::from_secs(1));
    targets =
        benchmark_fixture_pipeline,
        benchmark_fetch_stage,
        benchmark_analyze_stage,
        benchmark_storage_stage,
        benchmark_report_stage
);
criterion_main!(benches);

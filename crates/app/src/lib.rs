//! 应用编排层：fixture 与 HTTP pipeline 共享调度/分析/存储/报告逻辑。

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Timelike, Utc};
use tracing::{debug, info, warn};
use trendradar_analyze::{
    RankedNews, SourceSummary, filter_by_keywords, group_news_by_source, rank_news,
};
use trendradar_config::{AppConfig, load_default_config, validate_config};
use trendradar_domain::{NewsItem, RunContext};
use trendradar_fetch::{
    Fetcher, FixtureHotlistFetcher, FixtureRssFetcher, HttpHotlistFetcher, HttpRssFetcher,
};
use trendradar_report::{
    render_news_html, render_news_json, render_news_markdown, render_news_table,
};
use trendradar_schedule::{
    ScheduleContext, ScheduleDecision, decision_from_config, decision_from_config_at,
};
use trendradar_storage::{NewsRepository, SqliteNewsRepository};

/// 默认数据库文件名。
const DEFAULT_DB_FILENAME: &str = "trendradar.db";

/// 返回应用标识。
#[must_use]
pub fn app_name() -> &'static str {
    "trendradar-rust"
}

/// 验证基础编排依赖是否可用。
pub fn bootstrap() -> anyhow::Result<()> {
    let config = load_default_config()?;
    bootstrap_with_config(&config)?;
    Ok(())
}

/// 验证给定配置能否通过基础编排校验。
pub fn bootstrap_with_config(config: &AppConfig) -> anyhow::Result<()> {
    let _ = validate_config(config.clone())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Source definitions
// ---------------------------------------------------------------------------

/// Fixture 数据源类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureSourceKind {
    /// 热榜数据源。
    Hotlist,
    /// RSS 数据源。
    Rss,
}

/// Fixture 数据源定义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureSource {
    /// 数据源标识。
    pub source_id: String,
    /// 数据源类型。
    pub kind: FixtureSourceKind,
    /// fixture 文件路径。
    pub fixture_path: PathBuf,
}

impl FixtureSource {
    /// 创建热榜 fixture 数据源。
    #[must_use]
    pub fn hotlist(source_id: impl Into<String>, fixture_path: impl Into<PathBuf>) -> Self {
        Self {
            source_id: source_id.into(),
            kind: FixtureSourceKind::Hotlist,
            fixture_path: fixture_path.into(),
        }
    }

    /// 创建 RSS fixture 数据源。
    #[must_use]
    pub fn rss(source_id: impl Into<String>, fixture_path: impl Into<PathBuf>) -> Self {
        Self {
            source_id: source_id.into(),
            kind: FixtureSourceKind::Rss,
            fixture_path: fixture_path.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Pipeline result
// ---------------------------------------------------------------------------

/// 最小 pipeline 的聚合输出。
#[derive(Debug)]
pub struct PipelineResult {
    /// 调度决策。
    pub decision: ScheduleDecision,
    /// 抓取到的原始条目。
    pub collected_items: Vec<NewsItem>,
    /// 关键词过滤后条目。
    pub filtered_items: Vec<NewsItem>,
    /// 分析排序结果。
    pub ranked_items: Vec<RankedNews>,
    /// 来源聚合结果。
    pub source_summaries: Vec<SourceSummary>,
    /// 落库后回读的条目。
    pub stored_items: Vec<NewsItem>,
    /// 结构化 JSON 输出。
    pub report_json: Option<String>,
    /// HTML 报告输出。
    pub report_html: Option<String>,
    /// 终端彩色表格输出。
    pub report_table: Option<String>,
    /// Markdown 表格输出。
    pub report_markdown: Option<String>,
}

// ---------------------------------------------------------------------------
// Core pipeline logic (shared by fixture and HTTP paths)
// ---------------------------------------------------------------------------

/// 从配置和 fetcher 列表运行 pipeline。
///
/// 这是所有 pipeline 变体的共享核心，负责调度决策、抓取、关键词过滤、分析、存储和报告。
/// 调用方负责构建 fetcher 列表。
///
/// `resilient` 为 true 时，单个数据源抓取失败仅记录警告并跳过（适合 HTTP 生产场景）；
/// 为 false 时，抓取错误直接传播（适合 fixture 开发场景，快速暴露配置错误）。
fn run_pipeline_with_fetchers(
    config: &AppConfig,
    started_at: DateTime<Utc>,
    fetchers: &[Box<dyn Fetcher>],
    db_path: Option<&Path>,
    resilient: bool,
) -> anyhow::Result<PipelineResult> {
    bootstrap_with_config(config)?;
    info!(timezone = %config.timezone, "bootstrap completed");

    let decision = compute_decision(config, started_at)?;
    info!(
        collect = decision.collect,
        analyze = decision.analyze,
        push = decision.push,
        "schedule decision"
    );

    // -- Collect --
    let mut collected_items = Vec::new();
    if decision.collect {
        for fetcher in fetchers {
            match fetcher.fetch() {
                Ok(items) => {
                    debug!(count = items.len(), "fetched items");
                    collected_items.extend(items);
                }
                Err(error) if resilient => {
                    warn!(%error, "fetch failed, skipping source");
                }
                Err(error) => {
                    return Err(error.into());
                }
            }
        }
        info!(total = collected_items.len(), "collection completed");
    }

    // -- Keyword filtering --
    let filtered_items = filter_by_keywords(&collected_items, &config.keywords);
    if !config.keywords.is_empty() {
        info!(
            before = collected_items.len(),
            after = filtered_items.len(),
            keywords = ?config.keywords,
            "keyword filtering applied"
        );
    }

    // -- Analyze --
    let ranked_items = if decision.analyze {
        let ranked = rank_news(&filtered_items);
        debug!(count = ranked.len(), "ranked items");
        ranked
    } else {
        Vec::new()
    };
    let source_summaries = if decision.analyze {
        group_news_by_source(&filtered_items)
    } else {
        Vec::new()
    };

    // -- Store --
    let mut repository = match db_path {
        Some(path) => {
            info!(path = %path.display(), "opening file database");
            SqliteNewsRepository::open(path)?
        }
        None => SqliteNewsRepository::in_memory()?,
    };
    for item in &filtered_items {
        repository.save_news(item.clone())?;
    }
    let stored_items = repository.list_news()?;
    info!(count = stored_items.len(), "storage completed");

    // -- Report --
    let (report_json, report_html, report_table, report_markdown) = if decision.push {
        let context = RunContext {
            started_at,
            timezone: config.timezone.clone(),
        };
        let json = Some(render_news_json(&stored_items, &context)?);
        let html = Some(render_news_html(&stored_items, &context));
        let table = Some(render_news_table(&stored_items, &context));
        let markdown = Some(render_news_markdown(&stored_items, &context));

        // -- Notify --
        if config.notification.enabled {
            let notifiers = trendradar_notification::build_notifiers(
                config.notification.enabled,
                config.notification.webhook_url.as_deref(),
            );
            let subject = format!("TrendRadar: {} items collected", stored_items.len());
            let body = json.as_deref().unwrap_or("{}");
            for notifier in &notifiers {
                match notifier.send(&subject, body) {
                    Ok(()) => info!("notification sent"),
                    Err(error) => warn!(%error, "notification failed"),
                }
            }
        }

        (json, html, table, markdown)
    } else {
        (None, None, None, None)
    };

    Ok(PipelineResult {
        decision,
        collected_items,
        filtered_items,
        ranked_items,
        source_summaries,
        stored_items,
        report_json,
        report_html,
        report_table,
        report_markdown,
    })
}

/// 根据配置计算调度决策。
fn compute_decision(
    config: &AppConfig,
    started_at: DateTime<Utc>,
) -> anyhow::Result<ScheduleDecision> {
    match config.schedule.window {
        Some(_) => {
            let timezone: chrono_tz::Tz = config
                .timezone
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid timezone in config: {}", config.timezone))?;
            let local_hour = started_at.with_timezone(&timezone).hour() as u8;
            Ok(decision_from_config_at(
                config,
                ScheduleContext { local_hour },
            ))
        }
        None => Ok(decision_from_config(config)),
    }
}

// ---------------------------------------------------------------------------
// Fixture pipeline (existing)
// ---------------------------------------------------------------------------

/// 运行最小 fixture pipeline（内存数据库）。
pub fn run_fixture_pipeline(
    config: &AppConfig,
    started_at: DateTime<Utc>,
    sources: &[FixtureSource],
) -> anyhow::Result<PipelineResult> {
    let fetchers: Vec<Box<dyn Fetcher>> = sources
        .iter()
        .map(|source| -> Box<dyn Fetcher> {
            match source.kind {
                FixtureSourceKind::Hotlist => Box::new(FixtureHotlistFetcher::new(
                    source.source_id.clone(),
                    &source.fixture_path,
                )),
                FixtureSourceKind::Rss => Box::new(FixtureRssFetcher::new(
                    source.source_id.clone(),
                    &source.fixture_path,
                )),
            }
        })
        .collect();

    run_pipeline_with_fetchers(config, started_at, &fetchers, None, false)
}

// ---------------------------------------------------------------------------
// Config-driven HTTP pipeline (new)
// ---------------------------------------------------------------------------

/// 运行配置驱动的 HTTP pipeline。
///
/// 从 `AppConfig` 的 `rss_feeds` 和 `hotlist_apis` 字段构建 HTTP fetcher，
/// 自动完成调度→抓取→关键词过滤→分析→存储→报告流程。
pub fn run_config_pipeline(
    config: &AppConfig,
    started_at: DateTime<Utc>,
    db_path: Option<&Path>,
) -> anyhow::Result<PipelineResult> {
    let timeout = Duration::from_secs(config.http_timeout_secs);
    let mut fetchers: Vec<Box<dyn Fetcher>> = Vec::new();

    for feed in &config.rss_feeds {
        fetchers.push(Box::new(HttpRssFetcher::with_timeout(
            &feed.source_id,
            &feed.url,
            timeout,
        )));
    }

    for api in &config.hotlist_apis {
        let source_type = api.source_type.as_deref().unwrap_or("generic");
        let parser = trendradar_fetch::hotlist_parser_for(source_type);
        fetchers.push(Box::new(HttpHotlistFetcher::with_parser(
            &api.platform_id,
            &api.url,
            timeout,
            parser,
        )));
    }

    info!(
        rss_feeds = config.rss_feeds.len(),
        hotlist_apis = config.hotlist_apis.len(),
        timeout_secs = config.http_timeout_secs,
        "HTTP pipeline configured"
    );

    run_pipeline_with_fetchers(config, started_at, &fetchers, db_path, true)
}

// ---------------------------------------------------------------------------
// Config file discovery (G4)
// ---------------------------------------------------------------------------

/// 按优先级搜索配置文件路径。
///
/// 搜索顺序：
/// 1. 当前目录 `config.json`
/// 2. `~/.config/trendradar/config.json`
/// 3. `/etc/trendradar/config.json`
///
/// 返回第一个找到的路径；若均不存在则返回 None。
pub fn discover_config_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("config.json"),
        home_config_path(),
        PathBuf::from("/etc/trendradar/config.json"),
    ];

    for path in &candidates {
        if path.exists() {
            return Some(path.clone());
        }
    }

    None
}

/// 构建 `~/.config/trendradar/config.json` 路径。
fn home_config_path() -> PathBuf {
    std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".config/trendradar/config.json"))
        .unwrap_or_else(|_| PathBuf::from(".config/trendradar/config.json"))
}

/// 根据 CLI 参数解析配置文件路径。
///
/// 若 `--config` 未指定，尝试自动发现。
pub fn resolve_config_path(explicit: Option<&Path>) -> Option<PathBuf> {
    explicit.map(PathBuf::from).or_else(discover_config_path)
}

/// 默认数据库路径（在配置文件同目录下）。
pub fn default_db_path(config_path: Option<&Path>) -> PathBuf {
    config_path
        .and_then(|p| p.parent())
        .map(|dir| dir.join(DEFAULT_DB_FILENAME))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DB_FILENAME))
}

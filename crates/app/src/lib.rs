//! 应用编排层：fixture 与 HTTP pipeline 共享调度/分析/存储/报告逻辑。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use trendradar_ai::{ProviderConfig, provider_for, render_ai_analysis_markdown};
use trendradar_analyze::{
    DomainSummary, RankedNews, SourceSummary, classify_domain, dedupe_news_by_title,
    filter_by_keywords, group_news_by_domain, group_news_by_source, rank_news,
};
use trendradar_config::{
    AppConfig, NotificationConfig, NotificationSinkKind as ConfigNotificationSinkKind,
    SelectionConfig, StorageBackend, load_default_config, validate_config,
};
use trendradar_domain::{NewsItem, RunContext};
use trendradar_fetch::{
    Fetcher, FixtureHotlistFetcher, FixtureRssFetcher, HttpHotlistFetcher, HttpRssFetcher,
};
use trendradar_notification::{
    NotificationSinkKind, NotificationSinkSpec, build_notifiers_from_specs,
};
use trendradar_report::{
    render_news_html, render_news_json, render_news_markdown, render_news_table,
};
use trendradar_schedule::{
    ScheduleContext, ScheduleDecision, decision_from_config, decision_from_config_at,
};
use trendradar_storage::{
    FileObjectStoreNewsRepository, NewsRepository, OpendalObjectStoreNewsRepository,
    SqliteNewsRepository,
};

/// 默认数据库文件名。
const DEFAULT_DB_FILENAME: &str = "trendradar.db";
/// 冷却状态文件名。
const DEFAULT_COOLDOWN_STATE_FILENAME: &str = "trendradar.cooldown.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CooldownState {
    last_success_at: DateTime<Utc>,
}

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
    /// 领域聚合结果。
    pub domain_summaries: Vec<DomainSummary>,
    /// 去重后的条目。
    pub deduped_items: Vec<NewsItem>,
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
    /// AI 分析 Markdown 输出。
    pub ai_analysis_markdown: Option<String>,
}

/// 报告输出模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// 只输出 JSON。
    Json,
    /// 只输出 HTML。
    Html,
    /// 同时输出 JSON 与 HTML。
    Both,
    /// 只输出终端表格。
    Table,
    /// 只输出 Markdown。
    Markdown,
    /// 输出全部格式，供兼容旧调用方使用。
    All,
}

impl OutputMode {
    fn includes_json(self) -> bool {
        matches!(self, Self::Json | Self::Both | Self::All)
    }

    fn includes_html(self) -> bool {
        matches!(self, Self::Html | Self::Both | Self::All)
    }

    fn includes_table(self) -> bool {
        matches!(self, Self::Table | Self::All)
    }

    fn includes_markdown(self) -> bool {
        matches!(self, Self::Markdown | Self::All)
    }
}

fn notification_sink_kind(kind: ConfigNotificationSinkKind) -> NotificationSinkKind {
    match kind {
        ConfigNotificationSinkKind::Webhook => NotificationSinkKind::Webhook,
        ConfigNotificationSinkKind::Feishu => NotificationSinkKind::Feishu,
        ConfigNotificationSinkKind::Dingtalk => NotificationSinkKind::Dingtalk,
        ConfigNotificationSinkKind::Wecom => NotificationSinkKind::Wecom,
        ConfigNotificationSinkKind::Slack => NotificationSinkKind::Slack,
        ConfigNotificationSinkKind::Discord => NotificationSinkKind::Discord,
        ConfigNotificationSinkKind::Ntfy => NotificationSinkKind::Ntfy,
    }
}

fn push_notification_sink_spec(
    specs: &mut Vec<NotificationSinkSpec>,
    kind: NotificationSinkKind,
    url: &str,
) {
    if url.is_empty() {
        return;
    }

    if specs
        .iter()
        .any(|spec| spec.kind == kind && spec.url == url)
    {
        return;
    }

    specs.push(NotificationSinkSpec {
        kind,
        url: url.to_owned(),
    });
}

fn notification_sink_specs(config: &NotificationConfig) -> Vec<NotificationSinkSpec> {
    let mut specs = Vec::new();

    for sink in &config.sinks {
        push_notification_sink_spec(
            &mut specs,
            notification_sink_kind(sink.kind.clone()),
            &sink.url,
        );
    }

    if let Some(url) = config.webhook_url.as_deref() {
        push_notification_sink_spec(&mut specs, NotificationSinkKind::Webhook, url);
    }
    if let Some(url) = config.feishu_webhook_url.as_deref() {
        push_notification_sink_spec(&mut specs, NotificationSinkKind::Feishu, url);
    }
    if let Some(url) = config.dingtalk_webhook_url.as_deref() {
        push_notification_sink_spec(&mut specs, NotificationSinkKind::Dingtalk, url);
    }
    if let Some(url) = config.wecom_webhook_url.as_deref() {
        push_notification_sink_spec(&mut specs, NotificationSinkKind::Wecom, url);
    }
    if let Some(url) = config.discord_webhook_url.as_deref() {
        push_notification_sink_spec(&mut specs, NotificationSinkKind::Discord, url);
    }
    if let Some(url) = config.ntfy_topic_url.as_deref() {
        push_notification_sink_spec(&mut specs, NotificationSinkKind::Ntfy, url);
    }

    specs
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
    concurrent_fetch: bool,
    output_mode: OutputMode,
) -> anyhow::Result<PipelineResult> {
    bootstrap_with_config(config)?;
    info!(timezone = %config.timezone, "bootstrap completed");

    let decision = compute_decision(config, started_at, db_path)?;
    info!(
        collect = decision.collect,
        analyze = decision.analyze,
        push = decision.push,
        "schedule decision"
    );

    // -- Collect --
    let collected_items = if decision.collect {
        let collected = if concurrent_fetch {
            collect_items(fetchers, resilient)?
        } else {
            collect_items_sequentially(fetchers, resilient)?
        };
        info!(total = collected.len(), "collection completed");
        collected
    } else {
        Vec::new()
    };

    // -- Item selection --
    let filtered_items = select_items(&collected_items, &config.keywords, &config.selection);
    if !config.keywords.is_empty()
        || config.selection.high_rank_fallback_max_rank.is_some()
        || config.selection.min_items_per_source.is_some()
        || config.selection.min_items_per_domain.is_some()
    {
        info!(
            before = collected_items.len(),
            after = filtered_items.len(),
            keywords = ?config.keywords,
            high_rank_fallback_max_rank = ?config.selection.high_rank_fallback_max_rank,
            min_items_per_source = ?config.selection.min_items_per_source,
            min_items_per_domain = ?config.selection.min_items_per_domain,
            "item selection applied"
        );
    }

    let deduped_items = dedupe_news_by_title(&filtered_items);
    if deduped_items.len() != filtered_items.len() {
        info!(
            before = filtered_items.len(),
            after = deduped_items.len(),
            removed = filtered_items.len().saturating_sub(deduped_items.len()),
            "cross-source title dedup applied"
        );
    }

    // -- Analyze --
    let ranked_items = if decision.analyze {
        let ranked = rank_news(&deduped_items);
        debug!(count = ranked.len(), "ranked items");
        ranked
    } else {
        Vec::new()
    };
    let source_summaries = if decision.analyze {
        group_news_by_source(&deduped_items)
    } else {
        Vec::new()
    };
    let domain_summaries = if decision.analyze {
        group_news_by_domain(&deduped_items)
    } else {
        Vec::new()
    };

    // -- Store --
    let mut repository: Box<dyn NewsRepository> = match config.storage.backend {
        StorageBackend::Sqlite => match db_path {
            Some(path) => {
                info!(path = %path.display(), "opening file database");
                Box::new(SqliteNewsRepository::open(path)?)
            }
            None => Box::new(SqliteNewsRepository::in_memory()?),
        },
        StorageBackend::S3 => {
            let remote = config.storage.remote.as_ref().ok_or_else(|| {
                anyhow::anyhow!("remote storage config is required for s3 backend")
            })?;
            let provider = remote
                .provider
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("remote storage provider is required"))?;
            let prefix = remote.prefix.as_deref().unwrap_or("trendradar");

            match provider {
                "mock-s3" => {
                    let root = remote
                        .endpoint
                        .as_deref()
                        .ok_or_else(|| anyhow::anyhow!("mock-s3 endpoint path is required"))?;
                    Box::new(FileObjectStoreNewsRepository::open(
                        Path::new(root),
                        prefix,
                    )?)
                }
                "s3" | "aws-s3" => {
                    let bucket = remote
                        .bucket
                        .as_deref()
                        .ok_or_else(|| anyhow::anyhow!("remote storage bucket is required"))?;
                    Box::new(OpendalObjectStoreNewsRepository::open_s3(
                        bucket,
                        remote.endpoint.as_deref(),
                        remote.region.as_deref(),
                        prefix,
                    )?)
                }
                "oss" | "aliyun-oss" => {
                    let bucket = remote
                        .bucket
                        .as_deref()
                        .ok_or_else(|| anyhow::anyhow!("remote storage bucket is required"))?;
                    let endpoint = remote.endpoint.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("remote storage endpoint is required for oss provider")
                    })?;
                    Box::new(OpendalObjectStoreNewsRepository::open_oss(
                        bucket, endpoint, prefix,
                    )?)
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "unsupported remote storage provider: {provider}"
                    ));
                }
            }
        }
    };
    repository.save_news_batch(&deduped_items)?;
    let stored_items = repository.list_news()?;
    info!(count = stored_items.len(), "storage completed");

    let context = RunContext {
        started_at,
        timezone: config.timezone.clone(),
    };

    // -- Report --
    let (report_json, report_html, report_table, report_markdown) = if decision.push {
        let json = if output_mode.includes_json() {
            Some(render_news_json(&stored_items, &context)?)
        } else {
            None
        };
        let html = if output_mode.includes_html() {
            Some(render_news_html(&stored_items, &context))
        } else {
            None
        };
        let table = if output_mode.includes_table() {
            Some(render_news_table(&stored_items, &context))
        } else {
            None
        };
        let markdown = if output_mode.includes_markdown() {
            Some(render_news_markdown(&stored_items, &context))
        } else {
            None
        };

        // -- Notify --
        if config.notification.enabled {
            let sink_specs = notification_sink_specs(&config.notification);
            let notifiers = build_notifiers_from_specs(config.notification.enabled, &sink_specs);
            let subject = format!("TrendRadar: {} items collected", stored_items.len());
            let body = json
                .as_deref()
                .or(html.as_deref())
                .or(table.as_deref())
                .or(markdown.as_deref())
                .unwrap_or("");
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

    let ai_analysis_markdown = if config.ai_analysis.enabled {
        let provider_config = ProviderConfig {
            provider: config.ai_analysis.provider.clone(),
            timeout_secs: config.ai_analysis.timeout_secs,
            retry_attempts: config.ai_analysis.retry_attempts,
            max_items: config.ai_analysis.max_items,
            prompt: config.ai_analysis.prompt.clone(),
            model: config.ai_analysis.model.clone(),
            base_url: config.ai_analysis.base_url.clone(),
            api_key: config.ai_analysis.api_key.clone(),
            api_key_env: config.ai_analysis.api_key_env.clone(),
        };
        match provider_for(&provider_config) {
            Ok(provider) => match provider.analyze(&stored_items, &context) {
                Ok(analysis) => Some(render_ai_analysis_markdown(&analysis)),
                Err(error) => {
                    warn!(%error, "ai analysis failed");
                    None
                }
            },
            Err(error) => {
                warn!(%error, "ai analysis provider is unavailable");
                None
            }
        }
    } else {
        None
    };

    if decision.collect || decision.analyze || decision.push {
        write_cooldown_state(db_path, started_at)?;
    }

    Ok(PipelineResult {
        decision,
        collected_items,
        filtered_items,
        ranked_items,
        source_summaries,
        domain_summaries,
        deduped_items,
        stored_items,
        report_json,
        report_html,
        report_table,
        report_markdown,
        ai_analysis_markdown,
    })
}

fn item_key(item: &NewsItem) -> (String, String) {
    (item.source_id.clone(), item.title.clone())
}

fn select_items(
    items: &[NewsItem],
    keywords: &[String],
    selection: &SelectionConfig,
) -> Vec<NewsItem> {
    if keywords.is_empty()
        && selection.high_rank_fallback_max_rank.is_none()
        && selection.min_items_per_source.is_none()
        && selection.min_items_per_domain.is_none()
    {
        return items.to_vec();
    }

    let keyword_filtered = filter_by_keywords(items, keywords);
    let mut selected_keys: BTreeSet<(String, String)> =
        keyword_filtered.iter().map(item_key).collect();

    let mut ranked_candidates: Vec<&NewsItem> = items.iter().collect();
    ranked_candidates.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.source_id.cmp(&right.source_id))
    });

    if let Some(max_rank) = selection.high_rank_fallback_max_rank {
        for item in &ranked_candidates {
            if item.rank <= max_rank {
                selected_keys.insert(item_key(item));
            }
        }
    }

    if let Some(min_items_per_source) = selection.min_items_per_source.filter(|value| *value > 0) {
        let mut kept_counts: BTreeMap<String, usize> = BTreeMap::new();
        for item in items {
            if selected_keys.contains(&item_key(item)) {
                *kept_counts.entry(item.source_id.clone()).or_insert(0) += 1;
            }
        }

        for item in &ranked_candidates {
            let count = kept_counts.entry(item.source_id.clone()).or_insert(0);
            if *count >= min_items_per_source {
                continue;
            }

            if selected_keys.insert(item_key(item)) {
                *count += 1;
            }
        }
    }

    if let Some(min_items_per_domain) = selection.min_items_per_domain.filter(|value| *value > 0) {
        let mut kept_counts: BTreeMap<_, usize> = BTreeMap::new();
        for item in items {
            if selected_keys.contains(&item_key(item)) {
                *kept_counts.entry(classify_domain(item)).or_insert(0) += 1;
            }
        }

        for item in &ranked_candidates {
            let domain = classify_domain(item);
            let count = kept_counts.entry(domain).or_insert(0);
            if *count >= min_items_per_domain {
                continue;
            }

            if selected_keys.insert(item_key(item)) {
                *count += 1;
            }
        }
    }

    let mut selected: Vec<NewsItem> = items
        .iter()
        .filter(|item| selected_keys.contains(&item_key(item)))
        .cloned()
        .collect();
    selected.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    selected
}

fn collect_items(fetchers: &[Box<dyn Fetcher>], resilient: bool) -> anyhow::Result<Vec<NewsItem>> {
    thread::scope(|scope| {
        let handles: Vec<_> = fetchers
            .iter()
            .map(|fetcher| scope.spawn(move || fetcher.fetch()))
            .collect();

        let mut collected_items = Vec::new();
        for handle in handles {
            let fetch_result = handle.join().map_err(|panic_payload| {
                let panic_message = if let Some(message) = panic_payload.downcast_ref::<&str>() {
                    (*message).to_owned()
                } else if let Some(message) = panic_payload.downcast_ref::<String>() {
                    message.clone()
                } else {
                    "unknown panic payload".to_owned()
                };
                anyhow::anyhow!("fetch worker panicked: {panic_message}")
            })?;

            match fetch_result {
                Ok(items) => {
                    debug!(count = items.len(), "fetched items");
                    collected_items.extend(items);
                }
                Err(error) if resilient => {
                    warn!(%error, "fetch failed, skipping source");
                }
                Err(error) => return Err(error.into()),
            }
        }

        Ok(collected_items)
    })
}

fn collect_items_sequentially(
    fetchers: &[Box<dyn Fetcher>],
    resilient: bool,
) -> anyhow::Result<Vec<NewsItem>> {
    let mut collected_items = Vec::new();
    for fetcher in fetchers {
        match fetcher.fetch() {
            Ok(items) => {
                debug!(count = items.len(), "fetched items");
                collected_items.extend(items);
            }
            Err(error) if resilient => {
                warn!(%error, "fetch failed, skipping source");
            }
            Err(error) => return Err(error.into()),
        }
    }

    Ok(collected_items)
}

/// 根据配置计算调度决策。
fn compute_decision(
    config: &AppConfig,
    started_at: DateTime<Utc>,
    db_path: Option<&Path>,
) -> anyhow::Result<ScheduleDecision> {
    let last_success_at = read_last_success_at(db_path);
    match (
        config.schedule.window.is_some(),
        config.schedule.cooldown_minutes.is_some(),
        config.schedule.weekday.is_some(),
        config.schedule.weekend.is_some(),
    ) {
        (true, _, _, _) | (_, true, _, _) | (_, _, true, _) | (_, _, _, true) => {
            let timezone: chrono_tz::Tz = config
                .timezone
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid timezone in config: {}", config.timezone))?;
            let local_time = started_at.with_timezone(&timezone);
            Ok(decision_from_config_at(
                config,
                ScheduleContext {
                    local_hour: local_time.hour() as u8,
                    is_weekend: matches!(local_time.weekday(), Weekday::Sat | Weekday::Sun),
                    current_time: started_at,
                    last_success_at,
                },
            ))
        }
        (false, false, false, false) => Ok(decision_from_config(config)),
    }
}

fn cooldown_state_path(db_path: Option<&Path>) -> Option<PathBuf> {
    db_path.map(|path| {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(DEFAULT_COOLDOWN_STATE_FILENAME)
    })
}

fn read_last_success_at(db_path: Option<&Path>) -> Option<DateTime<Utc>> {
    let path = cooldown_state_path(db_path)?;
    let contents = std::fs::read_to_string(path).ok()?;
    let state: CooldownState = serde_json::from_str(&contents).ok()?;
    Some(state.last_success_at)
}

fn write_cooldown_state(
    db_path: Option<&Path>,
    last_success_at: DateTime<Utc>,
) -> anyhow::Result<()> {
    let Some(path) = cooldown_state_path(db_path) else {
        return Ok(());
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let state = CooldownState { last_success_at };
    let json = serde_json::to_string_pretty(&state)?;
    std::fs::write(path, json)?;
    Ok(())
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
    run_fixture_pipeline_with_output(config, started_at, sources, OutputMode::All)
}

/// 按指定输出模式运行最小 fixture pipeline（内存数据库）。
pub fn run_fixture_pipeline_with_output(
    config: &AppConfig,
    started_at: DateTime<Utc>,
    sources: &[FixtureSource],
    output_mode: OutputMode,
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

    run_pipeline_with_fetchers(
        config,
        started_at,
        &fetchers,
        None,
        false,
        false,
        output_mode,
    )
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
    run_config_pipeline_with_output(config, started_at, db_path, OutputMode::All)
}

/// 按指定输出模式运行配置驱动的 HTTP pipeline。
pub fn run_config_pipeline_with_output(
    config: &AppConfig,
    started_at: DateTime<Utc>,
    db_path: Option<&Path>,
    output_mode: OutputMode,
) -> anyhow::Result<PipelineResult> {
    let timeout = Duration::from_secs(config.http_timeout_secs);
    let mut fetchers: Vec<Box<dyn Fetcher>> = Vec::new();

    for feed in &config.rss_feeds {
        fetchers.push(Box::new(HttpRssFetcher::with_timeout_and_limit(
            &feed.source_id,
            &feed.url,
            timeout,
            feed.max_items,
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

    run_pipeline_with_fetchers(
        config,
        started_at,
        &fetchers,
        db_path,
        true,
        true,
        output_mode,
    )
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

#[cfg(test)]
mod tests {
    use super::{
        OutputMode, collect_items, notification_sink_specs, read_last_success_at,
        run_pipeline_with_fetchers, select_items, write_cooldown_state,
    };
    use chrono::TimeZone;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use trendradar_config::{
        AiAnalysisConfig, AppConfig, NotificationConfig, NotificationSinkConfig,
        NotificationSinkKind, ScheduleConfig, StorageBackend, StorageConfig,
    };
    use trendradar_domain::NewsItem;
    use trendradar_fetch::{FetchError, Fetcher};
    use trendradar_notification::NotificationSinkKind as NotifierSinkKind;

    struct ProbeFetcher {
        source_id: String,
        active: Arc<AtomicUsize>,
        max_active: Arc<AtomicUsize>,
        sleep_for: Duration,
    }

    impl Fetcher for ProbeFetcher {
        fn fetch(&self) -> trendradar_fetch::Result<Vec<NewsItem>> {
            let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_active.fetch_max(current, Ordering::SeqCst);
            std::thread::sleep(self.sleep_for);
            self.active.fetch_sub(1, Ordering::SeqCst);

            Ok(vec![NewsItem {
                title: format!("{} item", self.source_id),
                source_id: self.source_id.clone(),
                rank: 1,
            }])
        }
    }

    struct ErrorFetcher;

    impl Fetcher for ErrorFetcher {
        fn fetch(&self) -> trendradar_fetch::Result<Vec<NewsItem>> {
            Err(FetchError::Network {
                url: "https://example.invalid/fail".to_owned(),
                message: "simulated error".to_owned(),
            })
        }
    }

    fn test_config() -> AppConfig {
        AppConfig {
            timezone: "Asia/Shanghai".to_owned(),
            platforms: Vec::new(),
            schedule: ScheduleConfig {
                collect: true,
                analyze: false,
                push: false,
                window: None,
                cooldown_minutes: None,
                weekday: None,
                weekend: None,
            },
            rss_feeds: Vec::new(),
            hotlist_apis: Vec::new(),
            http_timeout_secs: 5,
            storage: StorageConfig::default(),
            ai_analysis: AiAnalysisConfig::default(),
            keywords: Vec::new(),
            notification: NotificationConfig::default(),
            selection: Default::default(),
        }
    }

    #[test]
    fn notification_sink_specs_merge_extensible_and_legacy_channels() {
        let mut notification = NotificationConfig {
            enabled: true,
            ..NotificationConfig::default()
        };
        notification.sinks = vec![
            NotificationSinkConfig {
                kind: NotificationSinkKind::Slack,
                url: "https://hooks.slack.com/services/one".to_owned(),
            },
            NotificationSinkConfig {
                kind: NotificationSinkKind::Webhook,
                url: "https://hooks.example.com/webhook".to_owned(),
            },
        ];
        notification.webhook_url = Some("https://hooks.example.com/webhook".to_owned());
        notification.wecom_webhook_url =
            Some("https://qyapi.weixin.qq.com/cgi-bin/webhook/send".to_owned());
        notification.discord_webhook_url = Some("https://discord.com/api/webhooks/two".to_owned());
        notification.ntfy_topic_url = Some("https://ntfy.sh/trendradar".to_owned());

        let specs = notification_sink_specs(&notification);

        assert_eq!(specs.len(), 5);
        assert_eq!(specs[0].kind, NotifierSinkKind::Slack);
        assert_eq!(specs[1].kind, NotifierSinkKind::Webhook);
        assert_eq!(specs[2].kind, NotifierSinkKind::Wecom);
        assert_eq!(specs[3].kind, NotifierSinkKind::Discord);
        assert_eq!(specs[4].kind, NotifierSinkKind::Ntfy);
    }

    #[test]
    fn pipeline_uses_extensible_slack_sink_for_notifications() -> anyhow::Result<()> {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/slack")
            .match_header("content-type", "application/json")
            .with_status(200)
            .create();

        let fetchers: Vec<Box<dyn Fetcher>> = vec![Box::new(ProbeFetcher {
            source_id: "weibo".to_owned(),
            active: Arc::new(AtomicUsize::new(0)),
            max_active: Arc::new(AtomicUsize::new(0)),
            sleep_for: Duration::from_millis(5),
        })];

        let mut config = test_config();
        config.schedule.push = true;
        config.notification = NotificationConfig {
            enabled: true,
            sinks: vec![NotificationSinkConfig {
                kind: NotificationSinkKind::Slack,
                url: format!("{}/slack", server.url()),
            }],
            ..NotificationConfig::default()
        };

        let result = run_pipeline_with_fetchers(
            &config,
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::Json,
        )?;

        assert!(result.report_json.is_some());
        mock.assert();
        Ok(())
    }

    #[test]
    fn collect_items_fetches_sources_concurrently() -> anyhow::Result<()> {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let fetchers: Vec<Box<dyn Fetcher>> = vec![
            Box::new(ProbeFetcher {
                source_id: "weibo".to_owned(),
                active: Arc::clone(&active),
                max_active: Arc::clone(&max_active),
                sleep_for: Duration::from_millis(80),
            }),
            Box::new(ProbeFetcher {
                source_id: "rss".to_owned(),
                active: Arc::clone(&active),
                max_active: Arc::clone(&max_active),
                sleep_for: Duration::from_millis(80),
            }),
        ];

        let collected = collect_items(&fetchers, false)?;
        assert_eq!(collected.len(), 2);
        assert!(
            max_active.load(Ordering::SeqCst) >= 2,
            "fetchers should overlap when collected concurrently"
        );
        Ok(())
    }

    #[test]
    fn resilient_collection_keeps_successful_items() -> anyhow::Result<()> {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let fetchers: Vec<Box<dyn Fetcher>> = vec![
            Box::new(ErrorFetcher),
            Box::new(ProbeFetcher {
                source_id: "rss".to_owned(),
                active,
                max_active,
                sleep_for: Duration::from_millis(10),
            }),
        ];

        let result = run_pipeline_with_fetchers(
            &test_config(),
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::All,
        )?;

        assert_eq!(result.collected_items.len(), 1);
        assert_eq!(result.collected_items[0].source_id, "rss");
        Ok(())
    }

    #[test]
    fn concurrent_resilient_collection_keeps_multiple_successes_with_multiple_failures()
    -> anyhow::Result<()> {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let fetchers: Vec<Box<dyn Fetcher>> = vec![
            Box::new(ProbeFetcher {
                source_id: "weibo".to_owned(),
                active: Arc::clone(&active),
                max_active: Arc::clone(&max_active),
                sleep_for: Duration::from_millis(60),
            }),
            Box::new(ErrorFetcher),
            Box::new(ProbeFetcher {
                source_id: "rss".to_owned(),
                active: Arc::clone(&active),
                max_active: Arc::clone(&max_active),
                sleep_for: Duration::from_millis(20),
            }),
            Box::new(ErrorFetcher),
        ];

        let result = run_pipeline_with_fetchers(
            &test_config(),
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::All,
        )?;

        assert!(
            max_active.load(Ordering::SeqCst) >= 2,
            "successful fetchers should overlap even when failures are present"
        );
        assert_eq!(result.collected_items.len(), 2);
        assert_eq!(result.stored_items.len(), 2);

        let collected_sources: Vec<&str> = result
            .collected_items
            .iter()
            .map(|item| item.source_id.as_str())
            .collect();
        assert_eq!(collected_sources, vec!["weibo", "rss"]);

        let stored_sources: Vec<&str> = result
            .stored_items
            .iter()
            .map(|item| item.source_id.as_str())
            .collect();
        assert_eq!(
            stored_sources,
            vec!["rss", "weibo"],
            "stored output should stay stable regardless of fetch completion order"
        );
        Ok(())
    }

    #[test]
    fn concurrent_resilient_collection_stable_output_ignores_fetcher_order() -> anyhow::Result<()> {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let fetchers: Vec<Box<dyn Fetcher>> = vec![
            Box::new(ProbeFetcher {
                source_id: "zhihu".to_owned(),
                active: Arc::clone(&active),
                max_active: Arc::clone(&max_active),
                sleep_for: Duration::from_millis(10),
            }),
            Box::new(ProbeFetcher {
                source_id: "bilibili".to_owned(),
                active: Arc::clone(&active),
                max_active: Arc::clone(&max_active),
                sleep_for: Duration::from_millis(70),
            }),
            Box::new(ErrorFetcher),
            Box::new(ProbeFetcher {
                source_id: "baidu".to_owned(),
                active,
                max_active,
                sleep_for: Duration::from_millis(40),
            }),
        ];

        let result = run_pipeline_with_fetchers(
            &test_config(),
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::All,
        )?;

        let stored_sources: Vec<&str> = result
            .stored_items
            .iter()
            .map(|item| item.source_id.as_str())
            .collect();
        assert_eq!(
            stored_sources,
            vec!["baidu", "bilibili", "zhihu"],
            "stored ordering should be deterministic under concurrent collection"
        );
        Ok(())
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_nanos();
        std::env::temp_dir().join(format!("trendradar-{name}-{nanos}"))
    }

    #[test]
    fn cooldown_state_blocks_recent_config_runs() -> anyhow::Result<()> {
        let base_dir = unique_test_dir("cooldown");
        std::fs::create_dir_all(&base_dir)?;
        let db_path = base_dir.join("trendradar.db");
        let started_at = chrono::Utc
            .with_ymd_and_hms(2026, 4, 14, 10, 0, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;
        let last_success_at = chrono::Utc
            .with_ymd_and_hms(2026, 4, 14, 9, 45, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("invalid fixed timestamp"))?;

        let mut config = test_config();
        config.schedule.cooldown_minutes = Some(30);

        write_cooldown_state(Some(&db_path), last_success_at)?;

        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let fetchers: Vec<Box<dyn Fetcher>> = vec![Box::new(ProbeFetcher {
            source_id: "weibo".to_owned(),
            active,
            max_active,
            sleep_for: Duration::from_millis(10),
        })];

        let result = run_pipeline_with_fetchers(
            &config,
            started_at,
            &fetchers,
            Some(&db_path),
            true,
            true,
            OutputMode::All,
        )?;

        assert!(result.collected_items.is_empty());
        assert_eq!(read_last_success_at(Some(&db_path)), Some(last_success_at));

        let _ = std::fs::remove_dir_all(base_dir);
        Ok(())
    }

    #[test]
    fn real_s3_provider_requires_bucket_config() {
        let fetchers: Vec<Box<dyn Fetcher>> = Vec::new();
        let mut config = test_config();
        config.storage.backend = StorageBackend::S3;
        config.storage.remote = Some(trendradar_config::RemoteStorageConfig {
            provider: Some("s3".to_owned()),
            bucket: None,
            endpoint: Some("http://127.0.0.1:9000".to_owned()),
            region: Some("auto".to_owned()),
            prefix: Some("trendradar".to_owned()),
        });

        let Err(error) = run_pipeline_with_fetchers(
            &config,
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::All,
        ) else {
            unreachable!("missing bucket should be rejected");
        };

        assert!(
            error
                .to_string()
                .contains("remote storage bucket is required")
        );
    }

    #[test]
    fn mock_s3_backend_writes_and_reads_via_file_object_store() -> anyhow::Result<()> {
        let fetchers: Vec<Box<dyn Fetcher>> = vec![Box::new(ProbeFetcher {
            source_id: "weibo".to_owned(),
            active: Arc::new(AtomicUsize::new(0)),
            max_active: Arc::new(AtomicUsize::new(0)),
            sleep_for: Duration::from_millis(5),
        })];
        let dir = unique_test_dir("mock-s3");
        std::fs::create_dir_all(&dir)?;
        let mut config = test_config();
        config.storage.backend = StorageBackend::S3;
        config.storage.remote = Some(trendradar_config::RemoteStorageConfig {
            provider: Some("mock-s3".to_owned()),
            bucket: Some("trendradar".to_owned()),
            endpoint: Some(dir.display().to_string()),
            region: None,
            prefix: Some("trendradar".to_owned()),
        });

        let result = run_pipeline_with_fetchers(
            &config,
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::All,
        )?;

        assert_eq!(result.stored_items.len(), 1);
        assert!(dir.join("trendradar/index/latest.json").exists());
        let _ = std::fs::remove_dir_all(dir);
        Ok(())
    }

    #[test]
    fn ai_analysis_generates_markdown_when_enabled() -> anyhow::Result<()> {
        let fetchers: Vec<Box<dyn Fetcher>> = vec![Box::new(ProbeFetcher {
            source_id: "weibo".to_owned(),
            active: Arc::new(AtomicUsize::new(0)),
            max_active: Arc::new(AtomicUsize::new(0)),
            sleep_for: Duration::from_millis(5),
        })];
        let mut config = test_config();
        config.ai_analysis.enabled = true;
        config.ai_analysis.provider = "mock".to_owned();
        config.ai_analysis.max_items = 1;

        let result = run_pipeline_with_fetchers(
            &config,
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::All,
        )?;

        let markdown = result
            .ai_analysis_markdown
            .ok_or_else(|| anyhow::anyhow!("missing ai analysis markdown"))?;
        assert!(markdown.contains("## AI Analysis"));
        assert!(markdown.contains("weibo item"));
        Ok(())
    }

    #[test]
    fn select_items_keeps_multiple_domains_when_minimum_per_domain_is_set() {
        let items = vec![
            NewsItem {
                title: "OpenAI launches new AI model for life sciences research - Axios".to_owned(),
                source_id: "ai-today".to_owned(),
                rank: 4,
            },
            NewsItem {
                title: "Finance ministers and top bankers raise serious concerns about Mythos AI model - BBC".to_owned(),
                source_id: "finance-today".to_owned(),
                rank: 3,
            },
            NewsItem {
                title: "2026 FIFA World Cup schedule: Qualified teams, groups, match dates - Yahoo Sports".to_owned(),
                source_id: "sports-today".to_owned(),
                rank: 5,
            },
        ];

        let selection = trendradar_config::SelectionConfig {
            high_rank_fallback_max_rank: None,
            min_items_per_source: None,
            min_items_per_domain: Some(1),
        };

        let selected = select_items(&items, &["openai".to_owned()], &selection);
        assert_eq!(selected.len(), 3);
    }

    #[test]
    fn pipeline_dedupes_duplicate_titles_before_storage() -> anyhow::Result<()> {
        struct DuplicateFetcher;

        impl Fetcher for DuplicateFetcher {
            fn fetch(&self) -> trendradar_fetch::Result<Vec<NewsItem>> {
                Ok(vec![
                    NewsItem {
                        title: "OpenAI launches new AI model for life sciences research - Axios"
                            .to_owned(),
                        source_id: "ai-today".to_owned(),
                        rank: 4,
                    },
                    NewsItem {
                        title: "OpenAI launches new AI model for life sciences research - Axios"
                            .to_owned(),
                        source_id: "openai-today".to_owned(),
                        rank: 2,
                    },
                ])
            }
        }

        let fetchers: Vec<Box<dyn Fetcher>> = vec![Box::new(DuplicateFetcher)];
        let mut config = test_config();
        config.schedule.analyze = true;
        config.schedule.push = true;

        let result = run_pipeline_with_fetchers(
            &config,
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::Json,
        )?;

        assert_eq!(result.collected_items.len(), 2);
        assert_eq!(result.filtered_items.len(), 2);
        assert_eq!(result.deduped_items.len(), 1);
        assert_eq!(result.stored_items.len(), 1);
        Ok(())
    }

    #[test]
    fn unsupported_ai_provider_does_not_break_pipeline() -> anyhow::Result<()> {
        let fetchers: Vec<Box<dyn Fetcher>> = vec![Box::new(ProbeFetcher {
            source_id: "rss".to_owned(),
            active: Arc::new(AtomicUsize::new(0)),
            max_active: Arc::new(AtomicUsize::new(0)),
            sleep_for: Duration::from_millis(5),
        })];
        let mut config = test_config();
        config.ai_analysis.enabled = true;
        config.ai_analysis.provider = "openai".to_owned();

        let result = run_pipeline_with_fetchers(
            &config,
            chrono::Utc::now(),
            &fetchers,
            None,
            true,
            true,
            OutputMode::All,
        )?;

        assert_eq!(result.collected_items.len(), 1);
        assert!(result.ai_analysis_markdown.is_none());
        Ok(())
    }
}

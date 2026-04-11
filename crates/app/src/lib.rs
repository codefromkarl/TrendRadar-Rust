//! 应用编排层骨架。

use std::path::PathBuf;

use chrono::{DateTime, Timelike, Utc};
use trendradar_analyze::{RankedNews, SourceSummary, group_news_by_source, rank_news};
use trendradar_config::{AppConfig, load_default_config, validate_config};
use trendradar_domain::{NewsItem, RunContext};
use trendradar_fetch::{Fetcher, FixtureHotlistFetcher, FixtureRssFetcher};
use trendradar_report::render_news_json;
use trendradar_schedule::{
    ScheduleContext, ScheduleDecision, decision_from_config, decision_from_config_at,
};
use trendradar_storage::{NewsRepository, SqliteNewsRepository};

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

/// 最小 pipeline 的聚合输出。
#[derive(Debug)]
pub struct PipelineResult {
    /// 调度决策。
    pub decision: ScheduleDecision,
    /// 抓取到的原始条目。
    pub collected_items: Vec<NewsItem>,
    /// 分析排序结果。
    pub ranked_items: Vec<RankedNews>,
    /// 来源聚合结果。
    pub source_summaries: Vec<SourceSummary>,
    /// 落库后回读的条目。
    pub stored_items: Vec<NewsItem>,
    /// 结构化 JSON 输出。
    pub report_json: Option<String>,
}

/// 运行最小 fixture pipeline。
pub fn run_fixture_pipeline(
    config: &AppConfig,
    started_at: DateTime<Utc>,
    sources: &[FixtureSource],
) -> anyhow::Result<PipelineResult> {
    bootstrap_with_config(config)?;

    let decision = match config.schedule.window {
        Some(_) => {
            let timezone: chrono_tz::Tz = config
                .timezone
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid timezone in config: {}", config.timezone))?;
            let local_hour = started_at.with_timezone(&timezone).hour() as u8;
            decision_from_config_at(config, ScheduleContext { local_hour })
        }
        None => decision_from_config(config),
    };
    let mut collected_items = Vec::new();

    if decision.collect {
        for source in sources {
            let items = match source.kind {
                FixtureSourceKind::Hotlist => {
                    FixtureHotlistFetcher::new(source.source_id.clone(), &source.fixture_path)
                        .fetch()?
                }
                FixtureSourceKind::Rss => {
                    FixtureRssFetcher::new(source.source_id.clone(), &source.fixture_path)
                        .fetch()?
                }
            };
            collected_items.extend(items);
        }
    }

    let ranked_items = if decision.analyze {
        rank_news(&collected_items)
    } else {
        Vec::new()
    };
    let source_summaries = if decision.analyze {
        group_news_by_source(&collected_items)
    } else {
        Vec::new()
    };

    let mut repository = SqliteNewsRepository::in_memory()?;
    for item in &collected_items {
        repository.save_news(item.clone())?;
    }
    let stored_items = repository.list_news()?;

    let report_json = if decision.push {
        let context = RunContext {
            started_at,
            timezone: config.timezone.clone(),
        };
        Some(render_news_json(&stored_items, &context)?)
    } else {
        None
    };

    Ok(PipelineResult {
        decision,
        collected_items,
        ranked_items,
        source_summaries,
        stored_items,
        report_json,
    })
}

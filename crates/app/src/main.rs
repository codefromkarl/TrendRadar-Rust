//! TrendRadar CLI 入口。

use std::path::PathBuf;
use std::process;

use chrono::Utc;
use clap::{Parser, ValueEnum};
use tracing_subscriber::EnvFilter;
use trendradar_app::{
    OutputMode, default_db_path, resolve_config_path, run_config_pipeline_with_output,
};
use trendradar_config::load_config_from_file;

/// CLI 输出格式。
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliOutputFormat {
    /// JSON 输出。
    Json,
    /// HTML 输出。
    Html,
    /// JSON + HTML 双输出。
    Both,
    /// 终端表格输出。
    Table,
    /// Markdown 输出。
    Markdown,
}

impl From<CliOutputFormat> for OutputMode {
    fn from(value: CliOutputFormat) -> Self {
        match value {
            CliOutputFormat::Json => Self::Json,
            CliOutputFormat::Html => Self::Html,
            CliOutputFormat::Both => Self::Both,
            CliOutputFormat::Table => Self::Table,
            CliOutputFormat::Markdown => Self::Markdown,
        }
    }
}

/// TrendRadar — 热榜与 RSS 聚合雷达。
#[derive(Debug, Parser)]
#[command(name = "trendradar", version, about)]
struct Cli {
    /// 配置文件路径（不指定则自动搜索）。
    #[arg(short, long = "config")]
    config: Option<PathBuf>,

    /// 数据库文件路径（不指定则使用配置文件同目录下的 trendradar.db）。
    #[arg(short, long = "db")]
    db: Option<PathBuf>,

    /// 输出格式：json / html / both / table / markdown。
    #[arg(short, long = "output", value_enum, default_value_t = CliOutputFormat::Json)]
    output: CliOutputFormat,

    /// 详细日志输出。
    #[arg(short, long = "verbose")]
    verbose: bool,

    /// 仅打印配置和调度决策，不实际执行。
    #[arg(long = "dry-run")]
    dry_run: bool,
}

fn main() {
    let cli = Cli::parse();

    // -- 初始化 tracing --
    let filter = if cli.verbose {
        EnvFilter::new("trendradar=debug")
    } else {
        EnvFilter::new("trendradar=info")
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "trendradar starting");

    // -- 解析配置路径 --
    let config_path = resolve_config_path(cli.config.as_deref());

    let config = match config_path
        .as_deref()
        .map(load_config_from_file)
        .transpose()
    {
        Ok(config) => config.unwrap_or_else(|| {
            tracing::warn!("no config file found, using defaults");
            trendradar_config::AppConfig::default()
        }),
        Err(error) => {
            tracing::error!(%error, "failed to load config");
            process::exit(1);
        }
    };

    tracing::info!(
        timezone = %config.timezone,
        rss_feeds = config.rss_feeds.len(),
        hotlist_apis = config.hotlist_apis.len(),
        keywords = config.keywords.len(),
        timeout_secs = config.http_timeout_secs,
        "config loaded"
    );

    // -- Dry-run --
    if cli.dry_run {
        tracing::info!("dry-run mode, exiting");
        return;
    }

    // -- 解析 DB 路径 --
    let db_path = cli.db.or_else(|| {
        let resolved = default_db_path(config_path.as_deref());
        tracing::debug!(path = %resolved.display(), "using default db path");
        Some(resolved)
    });

    // -- 执行 pipeline --
    let started_at = Utc::now();
    match run_config_pipeline_with_output(
        &config,
        started_at,
        db_path.as_deref(),
        cli.output.into(),
    ) {
        Ok(result) => {
            tracing::info!(
                collected = result.collected_items.len(),
                stored = result.stored_items.len(),
                "pipeline completed"
            );
            match cli.output {
                CliOutputFormat::Html => {
                    if let Some(html) = &result.report_html {
                        println!("{html}");
                    }
                }
                CliOutputFormat::Table => {
                    if let Some(table) = &result.report_table {
                        println!("{table}");
                    }
                }
                CliOutputFormat::Markdown => {
                    if let Some(markdown) = &result.report_markdown {
                        println!("{markdown}");
                    }
                }
                CliOutputFormat::Both => {
                    if let Some(json) = &result.report_json {
                        println!("{json}");
                    }
                    if let Some(html) = &result.report_html {
                        eprintln!("\n--- HTML Report ---\n{html}");
                    }
                }
                CliOutputFormat::Json => {
                    if let Some(json) = &result.report_json {
                        println!("{json}");
                    }
                }
            }
        }
        Err(error) => {
            tracing::error!(%error, "pipeline failed");
            process::exit(1);
        }
    }
}

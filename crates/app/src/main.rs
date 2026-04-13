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
use trendradar_domain::TrendRadarError;
use trendradar_fetch::FetchError;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitCode {
    Config = 1,
    Network = 2,
    Storage = 3,
    Unknown = 4,
}

impl ExitCode {
    const fn code(self) -> i32 {
        self as i32
    }
}

fn classify_domain_error(error: &TrendRadarError) -> ExitCode {
    match error {
        TrendRadarError::InvalidConfig { .. } => ExitCode::Config,
        TrendRadarError::Storage { .. } => ExitCode::Storage,
    }
}

fn classify_runtime_error(error: &anyhow::Error) -> ExitCode {
    for cause in error.chain() {
        if let Some(domain_error) = cause.downcast_ref::<TrendRadarError>() {
            return classify_domain_error(domain_error);
        }
        if let Some(fetch_error) = cause.downcast_ref::<FetchError>() {
            return match fetch_error {
                FetchError::Network { .. }
                | FetchError::Http { .. }
                | FetchError::ParseResponse { .. } => ExitCode::Network,
                FetchError::ReadFixture { .. } | FetchError::ParseFixture { .. } => {
                    ExitCode::Unknown
                }
            };
        }
    }

    let message = error.to_string();
    if message.contains("invalid timezone in config") || message.contains("invalid config:") {
        ExitCode::Config
    } else if message.contains("storage error:") {
        ExitCode::Storage
    } else if message.contains("network error")
        || message.contains("http ")
        || message.contains("failed to parse response")
    {
        ExitCode::Network
    } else {
        ExitCode::Unknown
    }
}

fn exit_with_code(code: ExitCode) -> ! {
    process::exit(code.code());
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
            exit_with_code(classify_domain_error(&error));
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

            if let Some(ai_markdown) = &result.ai_analysis_markdown {
                eprintln!("\n--- AI Analysis ---\n{ai_markdown}");
            }
        }
        Err(error) => {
            tracing::error!(%error, "pipeline failed");
            exit_with_code(classify_runtime_error(&error));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExitCode, classify_domain_error, classify_runtime_error};
    use trendradar_domain::TrendRadarError;
    use trendradar_fetch::FetchError;

    #[test]
    fn invalid_config_maps_to_config_exit_code() {
        let error = TrendRadarError::InvalidConfig {
            message: "timezone must not be empty".to_owned(),
        };

        assert_eq!(classify_domain_error(&error), ExitCode::Config);
    }

    #[test]
    fn storage_error_maps_to_storage_exit_code() {
        let error = anyhow::Error::new(TrendRadarError::Storage {
            message: "failed to open sqlite database".to_owned(),
        });

        assert_eq!(classify_runtime_error(&error), ExitCode::Storage);
    }

    #[test]
    fn fetch_network_error_maps_to_network_exit_code() {
        let error = anyhow::Error::new(FetchError::Network {
            url: "http://127.0.0.1:1".to_owned(),
            message: "connection refused".to_owned(),
        });

        assert_eq!(classify_runtime_error(&error), ExitCode::Network);
    }

    #[test]
    fn unknown_error_maps_to_unknown_exit_code() {
        let error = anyhow::anyhow!("unexpected failure");

        assert_eq!(classify_runtime_error(&error), ExitCode::Unknown);
    }
}

//! TrendRadar CLI 入口。

use std::env;
use std::path::Path;

use chrono::Utc;
use trendradar_app::run_config_pipeline;
use trendradar_config::load_config_from_file;

fn main() -> anyhow::Result<()> {
    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "config.json".to_owned());

    let config = load_config_from_file(Path::new(&config_path))?;
    let started_at = Utc::now();

    let result = run_config_pipeline(&config, started_at)?;

    if let Some(json) = &result.report_json {
        println!("{json}");
    }

    Ok(())
}

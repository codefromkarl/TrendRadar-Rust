//! 领域模型与共享错误定义。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 通用结果类型。
pub type Result<T> = std::result::Result<T, TrendRadarError>;

/// 项目级错误类型。
#[derive(Debug, Error)]
pub enum TrendRadarError {
    /// 配置无效。
    #[error("invalid config: {message}")]
    InvalidConfig {
        /// 具体错误说明。
        message: String,
    },
    /// 存储失败。
    #[error("storage error: {message}")]
    Storage {
        /// 具体错误说明。
        message: String,
    },
}

/// 热榜新闻条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsItem {
    /// 标题。
    pub title: String,
    /// 来源平台标识。
    pub source_id: String,
    /// 排名。
    pub rank: u32,
}

/// RSS 条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RssItem {
    /// 标题。
    pub title: String,
    /// 订阅源标识。
    pub feed_id: String,
    /// 原文链接。
    pub url: String,
}

/// 一次运行的元数据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunContext {
    /// 运行时间。
    pub started_at: DateTime<Utc>,
    /// 时区名称。
    pub timezone: String,
}

#[cfg(test)]
mod tests {
    use super::{NewsItem, RssItem, RunContext};
    use chrono::TimeZone;
    use std::error::Error;
    use std::fs::read_to_string;

    fn domain_fixture_path(name: &str) -> String {
        format!(
            "{}/../../fixtures/system/domain/{name}",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    #[test]
    fn news_item_serializes_with_stable_field_names() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(domain_fixture_path("news-item.json"))?;
        let item: NewsItem = serde_json::from_str(&fixture)?;

        assert_eq!(
            item,
            NewsItem {
                title: "Rust 1.85.0 released".to_owned(),
                source_id: "github-trending".to_owned(),
                rank: 1,
            }
        );
        assert_eq!(serde_json::to_string_pretty(&item)?, fixture.trim_end());
        Ok(())
    }

    #[test]
    fn rss_item_roundtrips_fixture_json() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(domain_fixture_path("rss-item.json"))?;
        let item: RssItem = serde_json::from_str(&fixture)?;

        assert_eq!(item.feed_id, "rust-blog");
        assert_eq!(item.url, "https://blog.rust-lang.org/async-patterns");
        assert_eq!(serde_json::to_string_pretty(&item)?, fixture.trim_end());
        Ok(())
    }

    #[test]
    fn run_context_uses_rfc3339_utc_timestamp() -> Result<(), Box<dyn Error>> {
        let fixture = read_to_string(domain_fixture_path("run-context.json"))?;
        let context: RunContext = serde_json::from_str(&fixture)?;

        assert_eq!(
            context,
            RunContext {
                started_at: chrono::Utc
                    .with_ymd_and_hms(2026, 4, 11, 9, 30, 0)
                    .single()
                    .ok_or("invalid fixed timestamp")?,
                timezone: "Asia/Shanghai".to_owned(),
            }
        );
        assert_eq!(serde_json::to_string_pretty(&context)?, fixture.trim_end());
        Ok(())
    }
}

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

//! 调度解析骨架。

use serde::{Deserialize, Serialize};

/// 调度决策。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleDecision {
    /// 是否抓取。
    pub collect: bool,
    /// 是否分析。
    pub analyze: bool,
    /// 是否推送。
    pub push: bool,
}

impl Default for ScheduleDecision {
    fn default() -> Self {
        Self {
            collect: true,
            analyze: true,
            push: true,
        }
    }
}

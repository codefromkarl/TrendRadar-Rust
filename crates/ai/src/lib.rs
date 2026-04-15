//! AI 分析层：提供可选的摘要与主题分析旁路能力。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use trendradar_domain::{NewsItem, RunContext};

/// AI 分析结果类型。
pub type Result<T> = std::result::Result<T, AiError>;

/// AI 分析错误。
#[derive(Debug, Error)]
pub enum AiError {
    /// 不支持的 provider。
    #[error("unsupported ai provider: {provider}")]
    UnsupportedProvider {
        /// provider 名称。
        provider: String,
    },
    /// provider 执行失败。
    #[error("ai provider error: {message}")]
    Provider {
        /// 错误详情。
        message: String,
    },
}

/// AI 分析结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiAnalysis {
    /// 摘要。
    pub summary: String,
    /// 关键主题。
    pub key_topics: Vec<String>,
    /// 来源亮点。
    pub source_highlights: Vec<String>,
}

/// AI 分析 provider 接口。
pub trait AnalysisProvider: Send + Sync {
    /// 基于新闻条目生成分析结果。
    fn analyze(&self, items: &[NewsItem], context: &RunContext) -> Result<AiAnalysis>;
}

/// Mock provider：使用稳定规则生成可复查的分析结果。
#[derive(Debug, Clone)]
pub struct MockAnalysisProvider {
    max_items: usize,
    prompt: Option<String>,
}

impl MockAnalysisProvider {
    /// 创建一个 mock provider。
    #[must_use]
    pub fn new(max_items: usize, prompt: Option<String>) -> Self {
        Self { max_items, prompt }
    }
}

impl AnalysisProvider for MockAnalysisProvider {
    fn analyze(&self, items: &[NewsItem], context: &RunContext) -> Result<AiAnalysis> {
        if items.is_empty() {
            return Ok(AiAnalysis {
                summary: format!(
                    "No items available for AI analysis at {} ({})",
                    context.started_at.to_rfc3339(),
                    context.timezone
                ),
                key_topics: Vec::new(),
                source_highlights: Vec::new(),
            });
        }

        let key_topics: Vec<String> = items
            .iter()
            .take(self.max_items.max(1))
            .map(|item| item.title.clone())
            .collect();

        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for item in items {
            *counts.entry(item.source_id.clone()).or_default() += 1;
        }
        let mut source_highlights: Vec<String> = counts
            .into_iter()
            .map(|(source, count)| format!("{source}: {count} items"))
            .collect();
        source_highlights.sort();

        let prompt_hint = self
            .prompt
            .as_deref()
            .map(|prompt| format!(" Prompt hint: {prompt}"))
            .unwrap_or_default();

        Ok(AiAnalysis {
            summary: format!(
                "Analyzed {} items across {} sources. Top topics: {}.{}",
                items.len(),
                source_highlights.len(),
                key_topics.join("; "),
                prompt_hint
            ),
            key_topics,
            source_highlights,
        })
    }
}

/// 根据 provider 名称创建分析 provider。
pub fn provider_for(
    provider: &str,
    max_items: usize,
    prompt: Option<String>,
) -> Result<Box<dyn AnalysisProvider>> {
    match provider {
        "mock" => Ok(Box::new(MockAnalysisProvider::new(max_items, prompt))),
        _ => Err(AiError::UnsupportedProvider {
            provider: provider.to_owned(),
        }),
    }
}

/// 将 AI 分析结果渲染为 Markdown。
#[must_use]
pub fn render_ai_analysis_markdown(analysis: &AiAnalysis) -> String {
    let mut output = String::new();
    output.push_str("## AI Analysis\n\n");
    output.push_str(&analysis.summary);
    output.push_str("\n\n### Key Topics\n");
    if analysis.key_topics.is_empty() {
        output.push_str("- None\n");
    } else {
        for topic in &analysis.key_topics {
            output.push_str(&format!("- {topic}\n"));
        }
    }
    output.push_str("\n### Source Highlights\n");
    if analysis.source_highlights.is_empty() {
        output.push_str("- None\n");
    } else {
        for highlight in &analysis.source_highlights {
            output.push_str(&format!("- {highlight}\n"));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{provider_for, render_ai_analysis_markdown};
    use chrono::TimeZone;
    use std::error::Error;
    use trendradar_domain::{NewsItem, RunContext};

    fn context() -> Result<RunContext, Box<dyn Error>> {
        Ok(RunContext {
            started_at: chrono::Utc
                .with_ymd_and_hms(2026, 4, 13, 20, 0, 0)
                .single()
                .ok_or("invalid fixed timestamp")?,
            timezone: "Asia/Shanghai".to_owned(),
        })
    }

    #[test]
    fn mock_provider_produces_stable_analysis() -> Result<(), Box<dyn Error>> {
        let provider = provider_for("mock", 2, Some("focus on ai".to_owned()))?;
        let items = vec![
            NewsItem {
                title: "AI chip rally".to_owned(),
                source_id: "weibo".to_owned(),
                rank: 1,
            },
            NewsItem {
                title: "Rust release".to_owned(),
                source_id: "zhihu".to_owned(),
                rank: 2,
            },
        ];

        let analysis = provider.analyze(&items, &context()?)?;
        assert!(analysis.summary.contains("Analyzed 2 items"));
        assert_eq!(analysis.key_topics, vec!["AI chip rally", "Rust release"]);
        assert_eq!(
            analysis.source_highlights,
            vec!["weibo: 1 items", "zhihu: 1 items"]
        );
        Ok(())
    }

    #[test]
    fn markdown_renderer_outputs_sections() -> Result<(), Box<dyn Error>> {
        let provider = provider_for("mock", 1, None)?;
        let items = vec![NewsItem {
            title: "AI chip rally".to_owned(),
            source_id: "weibo".to_owned(),
            rank: 1,
        }];
        let analysis = provider.analyze(&items, &context()?)?;
        let markdown = render_ai_analysis_markdown(&analysis);

        assert!(markdown.contains("## AI Analysis"));
        assert!(markdown.contains("### Key Topics"));
        assert!(markdown.contains("AI chip rally"));
        Ok(())
    }

    #[test]
    fn unsupported_provider_returns_error() {
        let Err(error) = provider_for("openai", 3, None) else {
            unreachable!("provider should fail");
        };
        assert!(error.to_string().contains("unsupported ai provider"));
    }
}

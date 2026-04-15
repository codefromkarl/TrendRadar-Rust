//! AI 分析层：提供可选的摘要与主题分析旁路能力。

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::env;
use std::time::Duration;
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

/// 分析 provider 的最小配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConfig {
    /// provider 名称。
    pub provider: String,
    /// 超时秒数。
    pub timeout_secs: u64,
    /// 重试次数。
    pub retry_attempts: u8,
    /// 最多纳入分析的条目数。
    pub max_items: usize,
    /// 可选提示词。
    pub prompt: Option<String>,
    /// 真实 provider 使用的模型名。
    pub model: Option<String>,
    /// 真实 provider 的基础 URL。
    pub base_url: Option<String>,
    /// 直接提供的 API key。
    pub api_key: Option<String>,
    /// API key 对应的环境变量名。
    pub api_key_env: Option<String>,
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

#[derive(Debug)]
struct OpenAiCompatibleProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    max_items: usize,
    prompt: Option<String>,
    retry_attempts: u8,
}

impl OpenAiCompatibleProvider {
    fn new(config: &ProviderConfig) -> Result<Self> {
        let model = config.model.clone().ok_or_else(|| AiError::Provider {
            message: "ai model is required for openai-compatible provider".to_owned(),
        })?;
        let base_url = config.base_url.clone().ok_or_else(|| AiError::Provider {
            message: "ai base_url is required for openai-compatible provider".to_owned(),
        })?;
        let api_key = resolve_api_key(config)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(1)))
            .build()
            .map_err(|error| AiError::Provider {
                message: format!("failed to build ai http client: {error}"),
            })?;

        Ok(Self {
            client,
            base_url,
            api_key,
            model,
            max_items: config.max_items,
            prompt: config.prompt.clone(),
            retry_attempts: config.retry_attempts,
        })
    }
}

impl AnalysisProvider for OpenAiCompatibleProvider {
    fn analyze(&self, items: &[NewsItem], context: &RunContext) -> Result<AiAnalysis> {
        let input = build_analysis_input(items, context, self.max_items, self.prompt.as_deref())?;
        let request_body = json!({
            "model": self.model,
            "input": input,
        });

        let mut last_error = None;
        for _ in 0..=self.retry_attempts {
            match self
                .client
                .post(&self.base_url)
                .bearer_auth(&self.api_key)
                .json(&request_body)
                .send()
            {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().map_err(|error| AiError::Provider {
                        message: format!("failed to read ai provider response: {error}"),
                    })?;
                    if !status.is_success() {
                        last_error = Some(format!("ai provider returned http {status}: {body}"));
                        continue;
                    }

                    return parse_analysis_response(&body);
                }
                Err(error) => {
                    last_error = Some(format!("failed to call ai provider: {error}"));
                }
            }
        }

        Err(AiError::Provider {
            message: last_error.unwrap_or_else(|| "ai provider request failed".to_owned()),
        })
    }
}

fn resolve_api_key(config: &ProviderConfig) -> Result<String> {
    if let Some(api_key) = config.api_key.clone() {
        return Ok(api_key);
    }

    if let Some(env_name) = config.api_key_env.as_deref() {
        return env::var(env_name).map_err(|error| AiError::Provider {
            message: format!("failed to read ai api key from env {env_name}: {error}"),
        });
    }

    Err(AiError::Provider {
        message: "ai api key is required for openai-compatible provider".to_owned(),
    })
}

fn build_analysis_input(
    items: &[NewsItem],
    context: &RunContext,
    max_items: usize,
    prompt: Option<&str>,
) -> Result<String> {
    let limited_items: Vec<Value> = items
        .iter()
        .take(max_items.max(1))
        .map(|item| {
            json!({
                "title": item.title,
                "source_id": item.source_id,
                "rank": item.rank,
            })
        })
        .collect();

    serde_json::to_string_pretty(&json!({
        "instructions": "Return strict JSON with fields summary:string, key_topics:string[], source_highlights:string[].",
        "prompt": prompt,
        "context": {
            "started_at": context.started_at.to_rfc3339(),
            "timezone": context.timezone,
        },
        "items": limited_items,
    }))
    .map_err(|error| AiError::Provider {
        message: format!("failed to serialize ai input payload: {error}"),
    })
}

fn parse_analysis_response(body: &str) -> Result<AiAnalysis> {
    let value = serde_json::from_str::<Value>(body).map_err(|error| AiError::Provider {
        message: format!("failed to parse ai provider response: {error}"),
    })?;

    let output_text = value
        .get("output_text")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| extract_output_text(&value))
        .ok_or_else(|| AiError::Provider {
            message: "ai provider response does not contain output_text".to_owned(),
        })?;

    serde_json::from_str::<AiAnalysis>(&output_text).map_err(|error| AiError::Provider {
        message: format!("failed to parse ai analysis payload: {error}"),
    })
}

fn extract_output_text(value: &Value) -> Option<String> {
    let outputs = value.get("output")?.as_array()?;
    let mut parts = Vec::new();

    for output in outputs {
        let Some(contents) = output.get("content").and_then(Value::as_array) else {
            continue;
        };
        for content in contents {
            if let Some(text) = content.get("text").and_then(Value::as_str) {
                parts.push(text.to_owned());
            }
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// 根据 provider 配置创建分析 provider。
pub fn provider_for(config: &ProviderConfig) -> Result<Box<dyn AnalysisProvider>> {
    match config.provider.as_str() {
        "mock" => Ok(Box::new(MockAnalysisProvider::new(
            config.max_items,
            config.prompt.clone(),
        ))),
        "openai-compatible" => Ok(Box::new(OpenAiCompatibleProvider::new(config)?)),
        _ => Err(AiError::UnsupportedProvider {
            provider: config.provider.clone(),
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
    use super::{ProviderConfig, provider_for, render_ai_analysis_markdown};
    use chrono::TimeZone;
    use mockito::Server;
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
        let provider = provider_for(&ProviderConfig {
            provider: "mock".to_owned(),
            timeout_secs: 5,
            retry_attempts: 0,
            max_items: 2,
            prompt: Some("focus on ai".to_owned()),
            model: None,
            base_url: None,
            api_key: None,
            api_key_env: None,
        })?;
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
        let provider = provider_for(&ProviderConfig {
            provider: "mock".to_owned(),
            timeout_secs: 5,
            retry_attempts: 0,
            max_items: 1,
            prompt: None,
            model: None,
            base_url: None,
            api_key: None,
            api_key_env: None,
        })?;
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
    fn openai_compatible_provider_requires_model() {
        let config = ProviderConfig {
            provider: "openai-compatible".to_owned(),
            timeout_secs: 5,
            retry_attempts: 0,
            max_items: 3,
            prompt: None,
            model: None,
            base_url: Some("https://example.com/v1/responses".to_owned()),
            api_key: Some("test-key".to_owned()),
            api_key_env: None,
        };

        let Err(error) = provider_for(&config) else {
            unreachable!("provider should fail");
        };
        assert!(error.to_string().contains("ai model is required"));
    }

    #[test]
    fn openai_compatible_provider_calls_responses_api() -> Result<(), Box<dyn Error>> {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/v1/responses")
            .match_header("authorization", "Bearer test-key")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_body(
                r#"{
                    "output_text":"{\"summary\":\"Real provider summary\",\"key_topics\":[\"AI chip rally\"],\"source_highlights\":[\"weibo: 1 items\"]}"
                }"#,
            )
            .create();

        let config = ProviderConfig {
            provider: "openai-compatible".to_owned(),
            timeout_secs: 5,
            retry_attempts: 0,
            max_items: 2,
            prompt: Some("focus on ai".to_owned()),
            model: Some("gpt-4.1-mini".to_owned()),
            base_url: Some(format!("{}/v1/responses", server.url())),
            api_key: Some("test-key".to_owned()),
            api_key_env: None,
        };
        let provider = provider_for(&config)?;
        let items = vec![NewsItem {
            title: "AI chip rally".to_owned(),
            source_id: "weibo".to_owned(),
            rank: 1,
        }];

        let analysis = provider.analyze(&items, &context()?)?;
        assert_eq!(analysis.summary, "Real provider summary");
        assert_eq!(analysis.key_topics, vec!["AI chip rally"]);
        mock.assert();
        Ok(())
    }

    #[test]
    fn unsupported_provider_returns_error() {
        let config = ProviderConfig {
            provider: "unknown-provider".to_owned(),
            timeout_secs: 5,
            retry_attempts: 0,
            max_items: 3,
            prompt: None,
            model: None,
            base_url: None,
            api_key: None,
            api_key_env: None,
        };

        let Err(error) = provider_for(&config) else {
            unreachable!("provider should fail");
        };
        assert!(error.to_string().contains("unsupported ai provider"));
    }
}

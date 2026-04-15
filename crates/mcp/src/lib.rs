//! 最小查询型 MCP/工具服务入口。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use trendradar_ai::{ProviderConfig, provider_for, render_ai_analysis_markdown};
use trendradar_domain::RunContext;
use trendradar_report::render_news_json;
use trendradar_storage::{NewsRepository, SqliteNewsRepository};

/// 工具定义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称。
    pub name: String,
    /// 工具说明。
    pub description: String,
}

/// 列出当前支持的查询工具。
#[must_use]
pub fn list_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "storage.list_news".to_owned(),
            description: "List stored news items from a sqlite database".to_owned(),
        },
        ToolDefinition {
            name: "report.render_json".to_owned(),
            description: "Render the stored news items as report JSON".to_owned(),
        },
        ToolDefinition {
            name: "ai.analyze".to_owned(),
            description: "Generate optional AI analysis markdown from stored news items".to_owned(),
        },
    ]
}

#[derive(Debug, Deserialize)]
struct ToolCallArgs {
    db_path: String,
    #[serde(default = "default_timezone")]
    timezone: String,
    #[serde(default = "default_ai_provider")]
    provider: String,
    #[serde(default = "default_ai_max_items")]
    max_items: usize,
}

fn default_timezone() -> String {
    "Asia/Shanghai".to_owned()
}

fn default_ai_provider() -> String {
    "mock".to_owned()
}

const fn default_ai_max_items() -> usize {
    5
}

fn load_stored_items(db_path: &str) -> Result<Vec<trendradar_domain::NewsItem>> {
    let repository = SqliteNewsRepository::open(std::path::Path::new(db_path))
        .with_context(|| format!("failed to open database: {db_path}"))?;
    repository.list_news().context("failed to list stored news")
}

fn run_context(timezone: &str) -> Result<RunContext> {
    Ok(RunContext {
        started_at: chrono::Utc::now(),
        timezone: timezone.to_owned(),
    })
}

fn call_tool(name: &str, args: ToolCallArgs) -> Result<Value> {
    let items = load_stored_items(&args.db_path)?;

    match name {
        "storage.list_news" => Ok(json!({ "items": items })),
        "report.render_json" => {
            let context = run_context(&args.timezone)?;
            let report = render_news_json(&items, &context)?;
            let parsed: Value = serde_json::from_str(&report)?;
            Ok(parsed)
        }
        "ai.analyze" => {
            let context = run_context(&args.timezone)?;
            let provider = provider_for(&ProviderConfig {
                provider: args.provider,
                timeout_secs: 15,
                retry_attempts: 0,
                max_items: args.max_items,
                prompt: None,
                model: None,
                base_url: None,
                api_key: None,
                api_key_env: None,
            })
            .context("failed to initialize ai provider")?;
            let analysis = provider
                .analyze(&items, &context)
                .context("failed to generate ai analysis")?;
            Ok(json!({ "markdown": render_ai_analysis_markdown(&analysis) }))
        }
        _ => Err(anyhow::anyhow!("unknown tool: {name}")),
    }
}

#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: Value,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

/// 处理一条 JSON 请求并返回 JSON 响应。
pub fn handle_request(input: &str) -> Result<String> {
    let request: RpcRequest = serde_json::from_str(input).context("failed to parse request")?;

    let response = match request.method.as_str() {
        "tools/list" => json!({
            "id": request.id,
            "result": list_tools()
        }),
        "tools/call" => {
            let params = request.params.context("missing params for tools/call")?;
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .context("missing tool name")?;
            let arguments = params
                .get("arguments")
                .cloned()
                .context("missing tool arguments")?;
            let args: ToolCallArgs =
                serde_json::from_value(arguments).context("failed to parse tool arguments")?;
            json!({
                "id": request.id,
                "result": call_tool(name, args)?
            })
        }
        _ => json!({
            "id": request.id,
            "error": {
                "message": format!("unsupported method: {}", request.method)
            }
        }),
    };

    serde_json::to_string_pretty(&response).context("failed to serialize response")
}

#[cfg(test)]
mod tests {
    use super::{handle_request, list_tools};
    use serde_json::Value;
    use std::error::Error;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use trendradar_domain::NewsItem;
    use trendradar_storage::{NewsRepository, SqliteNewsRepository};

    fn unique_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_nanos();
        std::env::temp_dir().join(format!("trendradar-{name}-{nanos}"))
    }

    fn temp_db_with_items() -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
        let dir = unique_test_dir("mcp");
        std::fs::create_dir_all(&dir)?;
        let db_path = dir.join("mcp.db");
        let mut repo = SqliteNewsRepository::open(&db_path)?;
        repo.save_news_batch(&[
            NewsItem {
                title: "Rust release".to_owned(),
                source_id: "weibo".to_owned(),
                rank: 1,
            },
            NewsItem {
                title: "AI chip rally".to_owned(),
                source_id: "zhihu".to_owned(),
                rank: 2,
            },
        ])?;
        Ok((dir, db_path))
    }

    #[test]
    fn tools_list_returns_query_tools() {
        let tools = list_tools();
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name, "storage.list_news");
    }

    #[test]
    fn tools_call_can_list_news() -> Result<(), Box<dyn Error>> {
        let (dir, db_path) = temp_db_with_items()?;
        let request = format!(
            r#"{{
                "id": 1,
                "method": "tools/call",
                "params": {{
                    "name": "storage.list_news",
                    "arguments": {{
                        "db_path": "{}"
                    }}
                }}
            }}"#,
            db_path.display()
        );

        let response = handle_request(&request)?;
        let value: Value = serde_json::from_str(&response)?;
        let items = value["result"]["items"]
            .as_array()
            .ok_or("missing result items")?;
        assert_eq!(items.len(), 2);
        let _ = std::fs::remove_dir_all(dir);
        Ok(())
    }

    #[test]
    fn tools_call_can_render_ai_analysis() -> Result<(), Box<dyn Error>> {
        let (dir, db_path) = temp_db_with_items()?;
        let request = format!(
            r#"{{
                "id": 2,
                "method": "tools/call",
                "params": {{
                    "name": "ai.analyze",
                    "arguments": {{
                        "db_path": "{}",
                        "provider": "mock",
                        "max_items": 1
                    }}
                }}
            }}"#,
            db_path.display()
        );

        let response = handle_request(&request)?;
        let value: Value = serde_json::from_str(&response)?;
        let markdown = value["result"]["markdown"]
            .as_str()
            .ok_or("missing markdown result")?;
        assert!(markdown.contains("## AI Analysis"));
        let _ = std::fs::remove_dir_all(dir);
        Ok(())
    }
}

//! 通知适配器：webhook 与 console 输出。

use std::fmt;

use tracing::info;

/// 通知结果类型。
pub type Result<T> = std::result::Result<T, NotificationError>;

/// 通知错误。
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// 网络请求失败。
    #[error("notification network error: {message}")]
    Network {
        /// 具体原因。
        message: String,
    },
    /// 配置无效。
    #[error("notification config error: {message}")]
    Config {
        /// 具体原因。
        message: String,
    },
}

/// 通知器接口。
pub trait Notifier {
    /// 发送通知消息。
    fn send(&self, subject: &str, body: &str) -> Result<()>;
}

/// 控制台通知器（用于调试）。
///
/// 将消息输出到 stdout。
pub struct ConsoleNotifier;

impl Notifier for ConsoleNotifier {
    fn send(&self, subject: &str, body: &str) -> Result<()> {
        info!("console notification: {subject}");
        println!("--- {subject} ---\n{body}");
        Ok(())
    }
}

/// Webhook 通知器。
///
/// 通过 HTTP POST 发送 JSON payload 到指定 URL。
pub struct WebhookNotifier {
    url: String,
    client: reqwest::blocking::Client,
}

impl WebhookNotifier {
    /// 创建 webhook 通知器。
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

/// 飞书通知器。
pub struct FeishuNotifier {
    url: String,
    client: reqwest::blocking::Client,
}

impl FeishuNotifier {
    /// 创建飞书通知器。
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

/// 钉钉通知器。
pub struct DingTalkNotifier {
    url: String,
    client: reqwest::blocking::Client,
}

impl DingTalkNotifier {
    /// 创建钉钉通知器。
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

/// 企业微信通知器。
pub struct WeComNotifier {
    url: String,
    client: reqwest::blocking::Client,
}

impl WeComNotifier {
    /// 创建企业微信通知器。
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Notifier for WebhookNotifier {
    fn send(&self, subject: &str, body: &str) -> Result<()> {
        let payload = serde_json::json!({
            "subject": subject,
            "body": body,
        });

        post_json(&self.client, &self.url, payload, "webhook")
    }
}

impl Notifier for FeishuNotifier {
    fn send(&self, subject: &str, body: &str) -> Result<()> {
        let payload = serde_json::json!({
            "msg_type": "text",
            "content": {
                "text": format!("{subject}\n{body}")
            }
        });

        post_json(&self.client, &self.url, payload, "feishu")
    }
}

impl Notifier for DingTalkNotifier {
    fn send(&self, subject: &str, body: &str) -> Result<()> {
        let payload = serde_json::json!({
            "msgtype": "text",
            "text": {
                "content": format!("{subject}\n{body}")
            }
        });

        post_json(&self.client, &self.url, payload, "dingtalk")
    }
}

impl Notifier for WeComNotifier {
    fn send(&self, subject: &str, body: &str) -> Result<()> {
        let payload = serde_json::json!({
            "msgtype": "text",
            "text": {
                "content": format!("{subject}\n{body}")
            }
        });

        post_json(&self.client, &self.url, payload, "wecom")
    }
}

impl fmt::Debug for ConsoleNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConsoleNotifier").finish()
    }
}

impl fmt::Debug for WebhookNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebhookNotifier")
            .field("url", &self.url)
            .finish()
    }
}

impl fmt::Debug for FeishuNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FeishuNotifier")
            .field("url", &self.url)
            .finish()
    }
}

impl fmt::Debug for DingTalkNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DingTalkNotifier")
            .field("url", &self.url)
            .finish()
    }
}

impl fmt::Debug for WeComNotifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeComNotifier")
            .field("url", &self.url)
            .finish()
    }
}

fn post_json(
    client: &reqwest::blocking::Client,
    url: &str,
    payload: serde_json::Value,
    channel: &str,
) -> Result<()> {
    let response =
        client
            .post(url)
            .json(&payload)
            .send()
            .map_err(|error| NotificationError::Network {
                message: error.to_string(),
            })?;

    let status = response.status().as_u16();
    if !(200..300).contains(&(status)) {
        return Err(NotificationError::Network {
            message: format!("{channel} webhook returned status {status}"),
        });
    }

    info!(status, channel, "notification sent");
    Ok(())
}

/// 从配置参数构建通知器列表。
///
/// 返回零个或多个通知器，调用方可逐个发送。
pub fn build_notifiers(
    enabled: bool,
    webhook_url: Option<&str>,
    feishu_webhook_url: Option<&str>,
    dingtalk_webhook_url: Option<&str>,
    wecom_webhook_url: Option<&str>,
) -> Vec<Box<dyn Notifier>> {
    let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();

    if !enabled {
        return notifiers;
    }

    if let Some(url) = webhook_url {
        if !url.is_empty() {
            notifiers.push(Box::new(WebhookNotifier::new(url)));
        }
    }

    if let Some(url) = feishu_webhook_url {
        if !url.is_empty() {
            notifiers.push(Box::new(FeishuNotifier::new(url)));
        }
    }

    if let Some(url) = dingtalk_webhook_url {
        if !url.is_empty() {
            notifiers.push(Box::new(DingTalkNotifier::new(url)));
        }
    }

    if let Some(url) = wecom_webhook_url {
        if !url.is_empty() {
            notifiers.push(Box::new(WeComNotifier::new(url)));
        }
    }

    // 如果没有配置任何外部通知器，默认添加 console
    if notifiers.is_empty() {
        notifiers.push(Box::new(ConsoleNotifier));
    }

    notifiers
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        ConsoleNotifier, DingTalkNotifier, FeishuNotifier, Notifier, WeComNotifier,
        WebhookNotifier, build_notifiers,
    };
    use std::error::Error;

    #[test]
    fn console_notifier_sends_without_error() -> Result<(), Box<dyn Error>> {
        let notifier = ConsoleNotifier;
        notifier.send("Test Subject", "Test Body")?;
        Ok(())
    }

    #[test]
    fn webhook_notifier_sends_to_mock_server() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/notify")
            .match_header("content-type", "application/json")
            .with_status(200)
            .create();

        let notifier = WebhookNotifier::new(format!("{}/notify", server.url()));
        notifier.send("Test", "Hello")?;

        mock.assert();
        Ok(())
    }

    #[test]
    fn webhook_notifier_reports_http_error() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        server.mock("POST", "/fail").with_status(500).create();

        let notifier = WebhookNotifier::new(format!("{}/fail", server.url()));
        let error = notifier
            .send("Test", "Hello")
            .expect_err("should fail on 500");

        assert!(
            error.to_string().contains("status 500"),
            "expected status 500 in: {error}"
        );
        Ok(())
    }

    #[test]
    fn build_notifiers_returns_empty_when_disabled() {
        let notifiers =
            build_notifiers(false, Some("http://example.com/webhook"), None, None, None);
        assert!(notifiers.is_empty());
    }

    #[test]
    fn build_notifiers_returns_webhook_when_configured() {
        let notifiers = build_notifiers(true, Some("http://example.com/webhook"), None, None, None);
        assert_eq!(notifiers.len(), 1);
    }

    #[test]
    fn build_notifiers_falls_back_to_console() {
        let notifiers = build_notifiers(true, None, None, None, None);
        assert_eq!(notifiers.len(), 1);
    }

    #[test]
    fn build_notifiers_ignores_empty_webhook_url() {
        let notifiers = build_notifiers(true, Some(""), None, None, None);
        assert_eq!(notifiers.len(), 1);
    }

    #[test]
    fn build_notifiers_returns_multiple_channel_notifiers() {
        let notifiers = build_notifiers(
            true,
            Some("http://example.com/webhook"),
            Some("http://example.com/feishu"),
            Some("http://example.com/dingtalk"),
            Some("http://example.com/wecom"),
        );
        assert_eq!(notifiers.len(), 4);
    }

    #[test]
    fn feishu_notifier_sends_expected_payload() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        let payload = serde_json::json!({
            "msg_type": "text",
            "content": { "text": "Test\nHello" }
        });
        let mock = server
            .mock("POST", "/feishu")
            .match_header("content-type", "application/json")
            .match_body(payload.to_string().into_bytes())
            .with_status(200)
            .create();

        let notifier = FeishuNotifier::new(format!("{}/feishu", server.url()));
        notifier.send("Test", "Hello")?;

        mock.assert();
        Ok(())
    }

    #[test]
    fn dingtalk_notifier_sends_expected_payload() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        let payload = serde_json::json!({
            "msgtype": "text",
            "text": { "content": "Test\nHello" }
        });
        let mock = server
            .mock("POST", "/dingtalk")
            .match_header("content-type", "application/json")
            .match_body(payload.to_string().into_bytes())
            .with_status(200)
            .create();

        let notifier = DingTalkNotifier::new(format!("{}/dingtalk", server.url()));
        notifier.send("Test", "Hello")?;

        mock.assert();
        Ok(())
    }

    #[test]
    fn wecom_notifier_sends_expected_payload() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new();
        let payload = serde_json::json!({
            "msgtype": "text",
            "text": { "content": "Test\nHello" }
        });
        let mock = server
            .mock("POST", "/wecom")
            .match_header("content-type", "application/json")
            .match_body(payload.to_string().into_bytes())
            .with_status(200)
            .create();

        let notifier = WeComNotifier::new(format!("{}/wecom", server.url()));
        notifier.send("Test", "Hello")?;

        mock.assert();
        Ok(())
    }
}

use crate::domain::services::discord_service::{DiscordMessage, DiscordService};
use reqwest::Client;
use serde_json::json;
use std::env;
use tracing::{error, info};

/// Discord webhook 实现
pub struct DiscordWebhookService {
    webhook_url: String,
    client: Client,
}

impl DiscordWebhookService {
    /// 创建新的 Discord 服务实例
    ///
    /// # 参数
    /// - `webhook_url`: Discord webhook URL
    ///
    /// # 环境变量
    /// 如果未提供 webhook_url，将尝试从 DISCORD_WEBHOOK_URL 环境变量读取
    pub fn new(webhook_url: Option<String>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let url = match webhook_url {
            Some(url) => url,
            None => {
                let env_url = env::var("DISCORD_WEBHOOK_URL").map_err(|_| {
                    "Discord webhook URL 未提供。请通过参数或 DISCORD_WEBHOOK_URL 环境变量设置。"
                })?;
                env_url
            }
        };

        // 验证 URL 格式
        if !url.starts_with("https://discord.com/api/webhooks/") && !url.starts_with("https://discordapp.com/api/webhooks/") {
            return Err("无效的 Discord webhook URL 格式".into());
        }

        Ok(Self {
            webhook_url: url,
            client: Client::new(),
        })
    }

    /// 从环境变量创建 Discord 服务（便捷方法）
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::new(None)
    }
}

#[async_trait::async_trait]
impl DiscordService for DiscordWebhookService {
    async fn send_message(&self, message: &DiscordMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("📤 发送消息到 Discord: {}", message.title);
        
        let embed = message.to_embed_json();
        
        let payload = json!({
            "embeds": [embed],
            "username": "TrendArc Bot",
            "avatar_url": "https://raw.githubusercontent.com/de-authority/trendarc/main/assets/logo.png"
        });

        match self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Discord 消息发送成功");
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    error!("❌ Discord 发送失败: {} - {}", status, error_text);
                    Err(format!("Discord 发送失败: {} - {}", status, error_text).into())
                }
            }
            Err(e) => {
                error!("❌ Discord 网络错误: {}", e);
                Err(e.into())
            }
        }
    }
    
    async fn send_batch(&self, messages: &[DiscordMessage]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if messages.is_empty() {
            info!("📤 没有消息需要发送到 Discord");
            return Ok(());
        }

        info!("📤 批量发送 {} 条消息到 Discord", messages.len());
        
        // Discord 限制：每个 webhook 调用最多 10 个 embeds
        const BATCH_SIZE: usize = 10;
        
        for chunk in messages.chunks(BATCH_SIZE) {
            let embeds: Vec<serde_json::Value> = chunk.iter().map(|msg| msg.to_embed_json()).collect();
            
            let payload = json!({
                "embeds": embeds,
                "username": "TrendArc Bot",
                "avatar_url": "https://raw.githubusercontent.com/de-authority/trendarc/main/assets/logo.png"
            });

            match self.client
                .post(&self.webhook_url)
                .json(&payload)
                .send()
                .await
            {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        error!("❌ Discord 批量发送失败: {} - {}", status, error_text);
                        return Err(format!("Discord 批量发送失败: {} - {}", status, error_text).into());
                    }
                }
                Err(e) => {
                    error!("❌ Discord 批量网络错误: {}", e);
                    return Err(e.into());
                }
            }
            
            // 避免速率限制
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        
        info!("✅ Discord 批量发送完成: {} 条消息", messages.len());
        Ok(())
    }
}

/// 创建 Discord 服务（工厂函数）
pub fn create_discord_service(webhook_url: Option<String>) -> Result<impl DiscordService, Box<dyn std::error::Error + Send + Sync>> {
    DiscordWebhookService::new(webhook_url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::services::discord_service::DiscordMessage;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_discord_service_send_message() {
        let mock_server = MockServer::start().await;
        
        // Mock Discord webhook endpoint
        Mock::given(method("POST"))
            .and(path("/api/webhooks/test"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;
        
        let webhook_url = format!("{}/api/webhooks/test", mock_server.uri());
        let service = DiscordWebhookService::new(Some(webhook_url)).unwrap();
        
        let message = DiscordMessage {
            title: "Test Title".to_string(),
            description: "Test Description".to_string(),
            url: "https://example.com".to_string(),
            source: "Test Source".to_string(),
            author: "Test Author".to_string(),
            published_at: "2024-01-01 00:00:00".to_string(),
            domain: Some("AI".to_string()),
            classification_reason: Some("Test reason".to_string()),
            classification_confidence: Some(0.8),
        };
        
        let result = service.send_message(&message).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_discord_service_creation() {
        // 测试有效的 URL
        let url = "https://discord.com/api/webhooks/123/abc";
        let service = DiscordWebhookService::new(Some(url.to_string()));
        assert!(service.is_ok());
        
        // 测试无效的 URL
        let invalid_url = "https://example.com";
        let service = DiscordWebhookService::new(Some(invalid_url.to_string()));
        assert!(service.is_err());
    }
}
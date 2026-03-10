use async_trait::async_trait;

/// Discord 消息结构体
#[derive(Debug, Clone)]
pub struct DiscordMessage {
    /// 消息标题
    pub title: String,
    /// 消息内容/描述
    pub description: String,
    /// 消息 URL
    pub url: String,
    /// 来源
    pub source: String,
    /// 作者
    pub author: String,
    /// 发布时间
    pub published_at: String,
    /// 领域分类
    pub domain: Option<String>,
    /// 分类依据
    pub classification_reason: Option<String>,
    /// 分类置信度
    pub classification_confidence: Option<f32>,
}

impl DiscordMessage {
    /// 从 NewsItem 创建 Discord 消息
    pub fn from_news_item(news: &crate::domain::NewsItem) -> Self {
        let domain_str = news.domain.map(|d| d.to_string());
        let confidence = news.classification_confidence;
        let reason = news.classification_reason.clone();

        // 格式化发布时间
        let published_at = news.published_at.format("%Y-%m-%d %H:%M:%S").to_string();

        Self {
            title: news.title.clone(),
            description: news.content.clone().unwrap_or_default(),
            url: news.url.clone(),
            source: news.source.clone(),
            author: news.author.clone(),
            published_at,
            domain: domain_str,
            classification_reason: reason,
            classification_confidence: confidence,
        }
    }

    /// 转换为 Discord webhook 的 embeds JSON 结构
    pub fn to_embed_json(&self) -> serde_json::Value {
        let mut embed = serde_json::json!({
            "title": self.title,
            "url": self.url,
            "color": 0x3498db, // Discord 蓝色
            "fields": [
                {
                    "name": "来源",
                    "value": format!("{} | {}", self.source, self.author),
                    "inline": true
                },
                {
                    "name": "发布时间",
                    "value": self.published_at,
                    "inline": true
                }
            ]
        });

        // 添加领域信息
        if let Some(domain) = &self.domain {
            let domain_emoji = match domain.as_str() {
                "AI" => "🤖",
                "Block" => "⛓️",
                "Social" => "📱",
                _ => "📰",
            };

            if let Some(embed_obj) = embed.as_object_mut() {
                embed_obj.insert("title".to_string(), serde_json::Value::String(format!("{} {}", domain_emoji, self.title)));
                
                let mut fields = embed_obj.get_mut("fields").unwrap().as_array_mut().unwrap();
                fields.push(serde_json::json!({
                    "name": "领域",
                    "value": domain,
                    "inline": true
                }));
            }
        }

        // 添加分类依据（如果存在）
        if let Some(reason) = &self.classification_reason {
            if let Some(embed_obj) = embed.as_object_mut() {
                let mut fields = embed_obj.get_mut("fields").unwrap().as_array_mut().unwrap();
                fields.push(serde_json::json!({
                    "name": "分类依据",
                    "value": reason,
                    "inline": false
                }));
            }
        }

        // 添加置信度（如果存在）
        if let Some(confidence) = self.classification_confidence {
            if let Some(embed_obj) = embed.as_object_mut() {
                let mut fields = embed_obj.get_mut("fields").unwrap().as_array_mut().unwrap();
                let confidence_percent = (confidence * 100.0).round();
                let confidence_bar = create_confidence_bar(confidence);
                
                fields.push(serde_json::json!({
                    "name": "分类置信度",
                    "value": format!("{}% {}", confidence_percent, confidence_bar),
                    "inline": false
                }));
            }
        }

        // 添加描述（如果有内容且不为空）
        if !self.description.is_empty() {
            let description_preview = if self.description.len() > 200 {
                format!("{}...", &self.description[..200])
            } else {
                self.description.clone()
            };

            if let Some(embed_obj) = embed.as_object_mut() {
                embed_obj.insert("description".to_string(), serde_json::Value::String(description_preview));
            }
        }

        embed
    }
}

/// 创建置信度进度条
fn create_confidence_bar(confidence: f32) -> String {
    let bars = 10;
    let filled = (confidence * bars as f32).round() as usize;
    let empty = bars - filled;
    
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Discord 服务接口
#[async_trait]
pub trait DiscordService: Send + Sync {
    /// 发送单条消息到 Discord
    async fn send_message(&self, message: &DiscordMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 批量发送消息到 Discord
    async fn send_batch(&self, messages: &[DiscordMessage]) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::NewsItem;
    use chrono::Utc;

    fn create_test_news_item() -> NewsItem {
        NewsItem::new(
            "test-id".to_string(),
            "Test Title".to_string(),
            "https://example.com/test".to_string(),
            "test-source".to_string(),
            "test-author".to_string(),
            Utc::now(),
        )
    }

    #[test]
    fn test_discord_message_from_news_item() {
        let news = create_test_news_item();
        let message = DiscordMessage::from_news_item(&news);
        
        assert_eq!(message.title, "Test Title");
        assert_eq!(message.url, "https://example.com/test");
        assert_eq!(message.source, "test-source");
        assert_eq!(message.author, "test-author");
    }

    #[test]
    fn test_discord_message_to_embed() {
        let news = create_test_news_item();
        let message = DiscordMessage::from_news_item(&news);
        let embed = message.to_embed_json();
        
        assert!(embed.is_object());
        let obj = embed.as_object().unwrap();
        assert!(obj.contains_key("title"));
        assert!(obj.contains_key("url"));
        assert!(obj.contains_key("color"));
        assert!(obj.contains_key("fields"));
    }

    #[test]
    fn test_confidence_bar() {
        assert_eq!(create_confidence_bar(0.0), "░░░░░░░░░░");
        assert_eq!(create_confidence_bar(1.0), "██████████");
        assert_eq!(create_confidence_bar(0.5), "█████░░░░░");
        assert_eq!(create_confidence_bar(0.75), "███████░░░");
    }
}

use crate::domain::services::{InferenceResult, NewsInferenceService};
use crate::domain::{Domain, NewsItem};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{debug, error, info};

/// OpenAI 请求结构
#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    response_format: ResponseFormat, // OpenAI使用response_format
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    type_: String,
}

impl ResponseFormat {
    fn json() -> Self {
        Self { type_: "json_object".to_string() }
    }
}

/// OpenAI 响应结构
#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

/// AI 返回的 JSON 格式定义 (与Ollama兼容)
#[derive(Deserialize)]
struct AIClassification {
    is_relevant: bool,
    domain: Option<String>,
    confidence: f32,
    reason: Option<String>,
    suggested_keywords: Vec<String>,
}

pub struct OpenAIInferenceService {
    api_key: String,
    model_name: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIInferenceService {
    /// 创建服务实例。
    ///
    /// 配置优先级（高→低）：
    /// 1. 环境变量 `OPENAI_API_KEY`（必需）
    /// 2. 环境变量 `OPENAI_MODEL`（默认：gpt-3.5-turbo）
    /// 3. 环境变量 `OPENAI_BASE_URL`（默认：https://api.openai.com/v1）
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable is required")?;
        
        let model = std::env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
            
        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        Ok(Self {
            api_key,
            model_name: model,
            base_url,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl NewsInferenceService for OpenAIInferenceService {
    async fn infer(
        &self,
        news: &NewsItem,
    ) -> Result<InferenceResult, Box<dyn Error + Send + Sync>> {
        let content_preview = news.content.as_deref().unwrap_or("No content provided.");
        let limit = 2000;
        let truncated_content = if content_preview.len() > limit {
            let mut end = limit;
            while !content_preview.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            &content_preview[..end]
        } else {
            content_preview
        };

        let sys_prompt = r#"You are a professional news classifier. 
Your goal is to determine if a news item is related to our target focus: AI, Blockchain, or Social Platforms.

AVAILABLE DOMAINS:
- AI: Artificial intelligence, LLMs, Neural Networks, Robotics, etc.
- Block: Cryptocurrency, Web3, DeFi, Smart Contracts, etc.
- Social: Social Media platforms (Twitter/X, Meta, Tiktok, etc.), tech platform news.

OUTPUT FORMAT (JSON):
{
  "is_relevant": boolean,
  "domain": "AI" | "Block" | "Social",
  "confidence": float (0.0-1.0),
  "reason": "short explanation in Chinese",
  "suggested_keywords": ["extracted_keyword1", "extracted_keyword2"]
}
"#;

        let user_input = format!(
            "Title: {}\nSource: {}\nContent Snippet: {}\n",
            news.title, news.source, truncated_content
        );

        let request = OpenAIChatRequest {
            model: self.model_name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: sys_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_input.clone(),
                },
            ],
            stream: false,
            response_format: ResponseFormat::json(),
        };

        debug!("Sending request to OpenAI with input:\n{}", user_input);
        
        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await?;
            error!("OpenAI API error {}: {}", status, err_body);
            
            // 处理速率限制和认证错误
            if status == 429 {
                return Err("OpenAI API rate limit exceeded".into());
            } else if status == 401 || status == 403 {
                return Err("OpenAI API authentication failed".into());
            }
            return Err(format!("OpenAI API returned error {}: {}", status, err_body).into());
        }

        let body: OpenAIChatResponse = response.json().await?;
        
        // OpenAI返回choices数组，取第一个
        if body.choices.is_empty() {
            return Err("OpenAI API returned empty choices array".into());
        }
        
        let content = &body.choices[0].message.content;
        info!("🤖 AI Raw Response: {}", content);
        let ai_result: AIClassification = serde_json::from_str(content)?;

        let final_domain = match ai_result.domain.as_deref() {
            Some("AI") => Some(Domain::AI),
            Some("Block") => Some(Domain::Block),
            Some("Social") => Some(Domain::Social),
            _ => None,
        };

        Ok(InferenceResult {
            is_relevant: ai_result.is_relevant && final_domain.is_some(),
            domain: final_domain,
            confidence: ai_result.confidence,
            reason: ai_result.reason.unwrap_or_else(|| "No reason provided".to_string()),
            suggested_keywords: ai_result.suggested_keywords,
        })
    }

    fn name(&self) -> &str {
        &self.model_name
    }
}
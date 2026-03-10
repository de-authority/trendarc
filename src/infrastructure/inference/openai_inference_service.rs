use crate::domain::services::{InferenceResult, NewsInferenceService};
use crate::domain::{Domain, NewsItem};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{error, info};

/// OpenAI 请求结构
#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
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
    #[serde(rename = "type")]
    type_: String,
}

impl ResponseFormat {
    fn json() -> Self {
        Self {
            type_: "json_object".to_string(),
        }
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
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
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
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or("fakekeys".to_string());

        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "qwen2.5:3b".to_string());

        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434/v1".to_string());

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
Your task is to analyze news items and determine if they belong to:
- AI: Artificial intelligence, LLMs, Neural Networks, Robotics, etc.
- Block: Cryptocurrency, Web3, DeFi, Smart Contracts, etc.
- Social: Social Media platforms (Twitter/X, Meta, Tiktok, etc.), tech platform news.

### RULES:
1. Always provide a "reason" in brief English, regardless of the "is_relevant" value.
2. If "is_relevant" is false:
   - Set "domain" to null.
   - The "reason" should explain why it does not fit the target domains.
3. If "is_relevant" is true:
   - "domain" MUST be one of ["AI", "Block", "Social"].
   - The "reason" should highlight the specific connection to the domain.
4. "suggested_keywords" should be an empty array [] if "is_relevant" is false.
5. Output strictly valid JSON. No conversational filler.
6. "is_relevant" MUST be strictly a boolean (true or false). NEVER use null or any other type.

### OUTPUT FORMAT:
{
  "is_relevant": boolean,
  "domain": "AI" | "Block" | "Social" | null,
  "confidence": float,
  "reason": "Short explanation in English",
  "suggested_keywords": ["keyword1", "keyword2"]
}
"#;

        let user_input = format!(
            "Title: {}\nContent Snippet: {}\n",
            news.title, truncated_content
        );
        info!("Sending request to OpenAI with input: {}", &user_input);
        let request = OpenAIChatRequest {
            model: self.model_name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: sys_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_input,
                },
            ],
            temperature: Some(0.0),
            stream: false,
            response_format: ResponseFormat::json(),
        };



        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let err_body = response.text().await?;
            error!("OpenAI API error {}: {}", status, err_body);

            // 处理速率限制和认证错误
            if status == 429 {
                return Err("OpenAI API rate limit exceeded".into());
            } else if status == 401 || status == 403 {
                return Err("OpenAI API authentication failed".into());
            } else if status == 405 {
                // Ollama可能返回405错误，这通常意味着端点不支持某些方法
                return Err(format!("Ollama/OpenAI API endpoint does not support this method (405). URL: {}, Supported: POST only", url).into());
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
            reason: ai_result
                .reason
                .unwrap_or_else(|| "No reason provided".to_string()),
            suggested_keywords: ai_result.suggested_keywords,
        })
    }

    fn name(&self) -> &str {
        &self.model_name
    }
}

mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_ai_infer() {
        let news = NewsItem::new(
            "test-id-123".to_string(),
            "LibreSprite – open-source pixel art editor".to_string(),
            "https://example.com/libresprite".to_string(),
            "hackernews".to_string(),
            "opensource_contributor".to_string(),
            Utc::now(),
        ).with_content(
            "This program is distributed under the GNU General Public License Version 2 | WebSite Designed With ❤️".to_string()
        );

        // This will panic if OPENAI_API_KEY is not set, but that's acceptable for a test
        let opai = OpenAIInferenceService::new().unwrap();

        println!("We start let ai infer now");
        let start_time = Instant::now();
        let result = opai.infer(&news).await;
        // Just verify the function doesn't panic for basic cases
        // We can't assert much without a real API key
        println!("Inference result: {:?}", result);
        let dur = start_time.elapsed();
        println!("infer takes {} seconds", dur.as_secs_f32());
    }

    #[tokio::test]
    async fn test_ai_infer_correctness_relevant() {
        use chrono::Utc;
        use std::time::Instant;

        // 构造一篇极其明显的 AI 相关新闻
        let news = NewsItem::new(
        "test-id-ai-001".to_string(),
        "OpenAI releases new GPT-5 model with advanced reasoning".to_string(),
        "https://example.com/gpt5".to_string(),
        "hackernews".to_string(),
        "ai_researcher".to_string(),
        Utc::now(),
    ).with_content(
        "The new large language model features advanced neural networks, better robotic control algorithms, and massively improved context windows."
            .to_string(),
    );

        let opai = OpenAIInferenceService::new().expect("Failed to init service");

        println!("🤖 正在测试正向用例 (预期为 AI 相关)...");
        let start_time = Instant::now();
        let result = opai.infer(&news).await.expect("Inference request failed");
        let dur = start_time.elapsed();

        println!("⏱️ 正向用例耗时: {:.2}s", dur.as_secs_f32());
        println!("返回结果: {:?}", result);

        // 断言验证 (根据你实际的 InferenceResult 结构体字段名调整)
        assert!(result.is_relevant, "模型未能识别出明显的 AI 新闻");

        // 如果你将 domain 定义为了 Option<String> 或者是 Enum，可以这样断言：
        // assert_eq!(result.domain.as_deref(), Some("AI"));

        assert!(!result.reason.is_empty(), "必须返回分类理由");
    }

    #[tokio::test]
    async fn test_ai_infer_correctness_irrelevant() {
        use chrono::Utc;
        use std::time::Instant;

        // 构造一篇与 AI/Block/Social 完全无关的美食新闻
        let news = NewsItem::new(
            "test-id-food-002".to_string(),
            "Delicious Chocolate Cake Recipe for Beginners".to_string(),
            "https://example.com/cake".to_string(),
            "reddit".to_string(),
            "chef_john".to_string(),
            Utc::now(),
        )
        .with_content(
            "Mix flour, sugar, and cocoa powder. Bake for 30 minutes at 180°C. Enjoy your dessert!"
                .to_string(),
        );

        let opai = OpenAIInferenceService::new().expect("Failed to init service");

        println!("🤖 正在测试反向用例 (预期为无关新闻)...");
        let start_time = Instant::now();
        let result = opai.infer(&news).await.expect("Inference request failed");
        let dur = start_time.elapsed();

        println!("⏱️ 反向用例耗时: {:.2}s", dur.as_secs_f32());
        println!("返回结果: {:?}", result);

        // 断言验证
        assert!(!result.is_relevant, "模型错误地将蛋糕食谱识别为相关新闻");
        assert!(
            result.domain.is_none(),
            "无关新闻的 domain 必须为 None (null)"
        );
        assert!(result.suggested_keywords.is_empty(), "无关新闻不应有关键词");
    }

    #[tokio::test]
    async fn test_ai_infer_concurrency() {
        use chrono::Utc;
        use std::time::Instant;

        let opai = OpenAIInferenceService::new().expect("Failed to init service");

        // 构造 3 篇不同的新闻
        let news1 = NewsItem::new(
            "test-concurrent-1".to_string(),
            "Nvidia announces new AI chips for data centers".to_string(),
            "url1".to_string(),
            "source".to_string(),
            "author".to_string(),
            Utc::now(),
        )
        .with_content("Hardware acceleration for LLMs.".to_string());

        let news2 = NewsItem::new(
            "test-concurrent-2".to_string(),
            "Ethereum gas fees hit all time low".to_string(),
            "url2".to_string(),
            "source".to_string(),
            "author".to_string(),
            Utc::now(),
        )
        .with_content(
            "Web3 and DeFi protocols are seeing cheaper smart contract executions.".to_string(),
        );

        let news3 = NewsItem::new(
            "test-concurrent-3".to_string(),
            "Top 10 gaming keyboards in 2026".to_string(),
            "url3".to_string(),
            "source".to_string(),
            "author".to_string(),
            Utc::now(),
        )
        .with_content("Mechanical switches and RGB lights.".to_string());

        println!("🚀 开始并发推理测试 (同时发起 3 个请求)...");
        let start_time = Instant::now();

        // 使用 tokio::join! 并发执行。注意：这要求你的 Ollama 配置允许并发，
        // 并且显存能够同时装下多个 Context Window。
        let (res1, res2, res3) =
            tokio::join!(opai.infer(&news1), opai.infer(&news2), opai.infer(&news3));

        let dur = start_time.elapsed();
        println!("⏱️ 3 条并发请求总耗时: {:.2}s", dur.as_secs_f32());

        // 打印平均耗时供参考
        println!("📊 平均每条耗时: {:.2}s", dur.as_secs_f32() / 3.0);

        // 基础验证，确保所有请求都成功返回了结果（没有因为超时或并发过高而报错）
        let r1 = res1.expect("News 1 failed");
        let r2 = res2.expect("News 2 failed");
        let r3 = res3.expect("News 3 failed");

        // 验证分类正确性
        assert!(r1.is_relevant, "新闻 1 应为 AI 相关");
        assert!(r2.is_relevant, "新闻 2 应为 Block 相关");
        assert!(!r3.is_relevant, "新闻 3 应为无关");
    }
}

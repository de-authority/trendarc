use crate::domain::config::ClassificationConfig;
use crate::domain::services::{ContentExtractor, DefaultContentExtractor, NewsInferenceService};
use crate::domain::{
    ClassificationStrategy, Domain, KeywordBasedStrategy, NewsItem, NewsItemStatus,
};
use futures::future::join_all;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

/// 分类结果（替代裸元组，提升可读性和可维护性）
struct ClassificationOutcome {
    domain: Option<Domain>,
    confidence: f32,
    reason: String,
    is_relevant: bool,
}

/// 分类服务：负责协调静态规则、正文抓取和 AI 仲裁
pub struct NewsClassificationService {
    /// 动态关键词配置 (共享读写)
    config: Arc<RwLock<ClassificationConfig>>,
    /// 配置文件路径
    config_path: PathBuf,
    /// 正文提取器
    extractor: Arc<dyn ContentExtractor>,
    /// AI 仲裁服务 (可选)
    inference_service: Option<Arc<dyn NewsInferenceService>>,
    /// 置信度阈值，低于此值将触发下一阶段
    confidence_threshold: f32,
}

impl NewsClassificationService {
    /// 创建新的分类服务
    pub fn new() -> Self {
        let config_path = PathBuf::from("config/classification.json");
        let config = ClassificationConfig::load_from_file(&config_path)
            .unwrap_or_else(|_| ClassificationConfig::default());

        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            extractor: Arc::new(DefaultContentExtractor::new()),
            inference_service: None,
            confidence_threshold: 0.7,
        }
    }

    /// 注入 AI 仲裁服务
    pub fn with_inference_service(mut self, service: Arc<dyn NewsInferenceService>) -> Self {
        self.inference_service = Some(service);
        self
    }

    /// 核心分类逻辑（改进型五阶漏斗方案）
    async fn classify_item(&self, news: &NewsItem) -> ClassificationOutcome {
        let mut best_domain = None;
        let mut best_confidence = 0.0_f32;

        // 获取当前快照配置
        let config = self.config.read().unwrap().clone();

        // --- 第一阶段：静态规则 (Title/URL 扫描) ---
        let strategy = KeywordBasedStrategy::from_config(config.clone());
        if let Some(result) = strategy.classify(news) {
            if result.confidence >= self.confidence_threshold {
                return ClassificationOutcome {
                    domain: Some(result.domain),
                    confidence: result.confidence,
                    reason: format!("[FastPass] {}", result.reason),
                    is_relevant: true,
                };
            }
            // 保存备选结果（第一阶段弱命中）
            best_domain = Some(result.domain);
            best_confidence = result.confidence;
        }

        // --- 第二阶段：正文抓取 (Content Enrichment) ---
        info!("🌐 抓取全文内容: {}", news.url);
        let mut augmented_news = news.clone();
        match self.extractor.extract(&news.url).await {
            Ok(content) => {
                let limit = 2000;
                let text_len = content.text.len();
                let end = if text_len > limit {
                    let mut e = limit;
                    while !content.text.is_char_boundary(e) && e > 0 {
                        e -= 1;
                    }
                    e
                } else {
                    text_len
                };

                let preview = if text_len > limit {
                    format!("{}...", &content.text[..end].replace('\n', " "))
                } else {
                    content.text.replace('\n', " ")
                };
                info!("📄 提取内容预览: {}", preview);
                augmented_news.content = Some(content.text);
            }
            Err(e) => {
                // 【修复】抓取失败不再直接丢弃——保留第一阶段弱命中结果，
                // 继续尝试 AI 仲裁（仅用标题/来源），否则进入兜底逻辑。
                warn!(
                    "⚠️ 无法提取全文，降级处理（保留已有弱命中）: {} | 错误: {}",
                    news.title, e
                );
                // augmented_news 此时无 content，后续阶段会感知到这一点
            }
        }

        // --- 第三阶段：基于全文的关键词扫描 ---
        if let Some(result) = strategy.classify(&augmented_news) {
            if result.confidence >= self.confidence_threshold {
                return ClassificationOutcome {
                    domain: Some(result.domain),
                    confidence: result.confidence,
                    reason: format!("[FullContentScan] {}", result.reason),
                    is_relevant: true,
                };
            }
            if result.confidence > best_confidence {
                best_domain = Some(result.domain);
                best_confidence = result.confidence;
            }
        }

        // --- 第四阶段：AI 仲裁 ---
        if let Some(ref ai) = self.inference_service {
            info!("🤖 触发 AI 深度推理: {}", news.title);
            match ai.infer(&augmented_news).await {
                Ok(result) => {
                    let ai_reason = format!("[AI:{}] {}", ai.name(), result.reason);

                    if !result.is_relevant || result.domain.is_none() {
                        return ClassificationOutcome {
                            domain: None,
                            confidence: 0.0,
                            reason: ai_reason,
                            is_relevant: false,
                        };
                    }

                    let domain = result.domain.unwrap();

                    // 【重要】暂时禁用自动学习关键词，因为AI关键词不准
                    // 分类置信度与关键词质量无关，需要人工审核
                    if !result.suggested_keywords.is_empty() {
                        info!("🔍 AI建议关键词（需人工审核）: {:?}", result.suggested_keywords);
                        // 暂不学习，记录日志供人工审核
                        // self.append_keywords(domain, result.suggested_keywords);
                    }

                    return ClassificationOutcome {
                        domain: Some(domain),
                        confidence: result.confidence,
                        reason: ai_reason,
                        is_relevant: true,
                    };
                }
                Err(e) => {
                    warn!("❌ AI 仲裁失败: {}", e);
                    // AI 失败时不直接丢弃，继续进入兜底逻辑
                }
            }
        }

        // --- 第五阶段：兜底 ---
        if let Some(domain) = best_domain {
            if best_confidence > 0.3 {
                return ClassificationOutcome {
                    domain: Some(domain),
                    confidence: best_confidence,
                    reason: "[WeakMatchFallback]".to_string(),
                    is_relevant: true,
                };
            }
        }

        ClassificationOutcome {
            domain: None,
            confidence: 0.0,
            reason: "Unclassifiable (No confident match and content unavailable/irrelevant)"
                .to_string(),
            is_relevant: false,
        }
    }

    /// 批量并发处理并过滤
    ///
    /// 并发策略：
    /// - 全文抓取（HTTP IO）对所有条目并发执行
    /// - Ollama 推理请求并发发出（Ollama 内部按 OLLAMA_NUM_PARALLEL 排队）
    /// - 相比串行，IO 等待时间大幅减少
    pub async fn classify_batch_and_filter(&self, items: &mut Vec<NewsItem>) {
        // 1. 取出所有条目所有权
        let all_items: Vec<NewsItem> = items.drain(..).collect();

        // 2. 为每条新闻创建分类 future，全部并发执行
        let futures: Vec<_> = all_items
            .iter()
            .map(|item| self.classify_item(item))
            .collect();
        let outcomes: Vec<ClassificationOutcome> = join_all(futures).await;

        // 3. 根据结果重建过滤后的列表
        let mut filtered_items = Vec::new();
        for (mut item, outcome) in all_items.into_iter().zip(outcomes) {
            if outcome.is_relevant {
                item.domain = outcome.domain;
                item.classification_confidence = Some(outcome.confidence);
                item.classification_reason = Some(outcome.reason);
                item.status = NewsItemStatus::Completed;
                filtered_items.push(item);
            } else {
                info!("🗑️ 丢弃无关项: {} | 依据: {}", item.title, outcome.reason);
            }
        }
        *items = filtered_items;
    }

    /// 暂时禁用的关键词学习方法
    #[allow(dead_code)]
    fn append_keywords(&self, domain: Domain, keywords: Vec<String>) {
        // 当前AI关键词不准，暂时禁用自动学习
        // 只记录日志供人工审核
        if !keywords.is_empty() {
            info!("🔍 AI建议关键词（暂不学习）: {:?}", keywords);
        }
    }

    pub fn group_by_domain(&self, news_items: &[NewsItem]) -> HashMap<Domain, Vec<NewsItem>> {
        let mut grouped = HashMap::new();
        grouped.insert(Domain::AI, Vec::new());
        grouped.insert(Domain::Block, Vec::new());
        grouped.insert(Domain::Social, Vec::new());

        for item in news_items {
            if let Some(domain) = item.domain {
                grouped
                    .entry(domain)
                    .or_insert_with(Vec::new)
                    .push(item.clone());
            }
        }
        grouped
    }
}

impl Default for NewsClassificationService {
    fn default() -> Self {
        Self::new()
    }
}

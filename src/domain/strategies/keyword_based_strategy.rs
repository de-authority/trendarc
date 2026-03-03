//! # Keyword-Based Classification Strategy
//!
//! Classifies news based on keyword matching with varying confidence levels.

use crate::domain::{NewsItem, Domain};
use crate::domain::config::ClassificationConfig;
use super::{ClassificationStrategy, ClassificationResult};

/// Strategy that classifies news based on keyword matching
pub struct KeywordBasedStrategy {
    /// Strong keywords (high confidence)
    strong_keywords: std::collections::HashMap<Domain, Vec<String>>,
    /// Weak keywords (low confidence)
    weak_keywords: std::collections::HashMap<Domain, Vec<String>>,
}

impl KeywordBasedStrategy {
    /// Create a new keyword-based strategy with default keywords
    pub fn new() -> Self {
        let config = ClassificationConfig::default();
        Self::from_config(config)
    }
    
    /// Create a new keyword-based strategy from configuration
    pub fn from_config(config: ClassificationConfig) -> Self {
        Self {
            strong_keywords: config.strong_keywords,
            weak_keywords: config.weak_keywords,
        }
    }
    
    /// Add a strong keyword for a domain
    pub fn add_strong_keyword(&mut self, domain: Domain, keyword: String) {
        self.strong_keywords
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(keyword);
    }
    
    /// Add a weak keyword for a domain
    pub fn add_weak_keyword(&mut self, domain: Domain, keyword: String) {
        self.weak_keywords
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(keyword);
    }
}

impl Default for KeywordBasedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassificationStrategy for KeywordBasedStrategy {
    fn classify(&self, news: &NewsItem) -> Option<ClassificationResult> {
        let title_lower = news.title.to_lowercase();
        let url_lower = news.url.to_lowercase();
        
        // First, check for strong keywords in title
        for (domain, keywords) in &self.strong_keywords {
            for keyword in keywords {
                if title_lower.contains(&keyword.to_lowercase()) {
                    return Some(ClassificationResult::high_confidence(
                        *domain,
                        format!("keyword-based (strong: {})", keyword),
                    ));
                }
            }
        }
        
        // Then, check for strong keywords in URL
        for (domain, keywords) in &self.strong_keywords {
            for keyword in keywords {
                if url_lower.contains(&keyword.to_lowercase()) {
                    return Some(ClassificationResult::high_confidence(
                        *domain,
                        format!("keyword-based (strong-url: {})", keyword),
                    ));
                }
            }
        }
        
        // Then, check for weak keywords in title
        for (domain, keywords) in &self.weak_keywords {
            for keyword in keywords {
                if title_lower.contains(&keyword.to_lowercase()) {
                    return Some(ClassificationResult::low_confidence(
                        *domain,
                        format!("keyword-based (weak: {})", keyword),
                    ));
                }
            }
        }
        
        // Finally, check for weak keywords in URL
        for (domain, keywords) in &self.weak_keywords {
            for keyword in keywords {
                if url_lower.contains(&keyword.to_lowercase()) {
                    return Some(ClassificationResult::low_confidence(
                        *domain,
                        format!("keyword-based (weak-url: {})", keyword),
                    ));
                }
            }
        }
        
        None
    }
    
    fn name(&self) -> &str {
        "keyword-based"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_news(title: &str, url: &str) -> NewsItem {
        NewsItem::new(
            "test-id".to_string(),
            title.to_string(),
            url.to_string(),
            "test-source".to_string(),
            "test-author".to_string(),
            Utc::now(),
        )
    }

    #[test]
    fn test_strong_keyword_in_title() {
        let strategy = KeywordBasedStrategy::new();
        let news = create_test_news("New GPT-4 model released", "https://example.com/article");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.confidence, 0.9);
        assert!(result.strategy_name.contains("strong"));
    }

    #[test]
    fn test_strong_keyword_in_url() {
        let strategy = KeywordBasedStrategy::new();
        let news = create_test_news("Major announcement", "https://example.com/bitcoin-update");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::Block);
        assert_eq!(result.confidence, 0.9);
        assert!(result.strategy_name.contains("strong-url"));
    }

    #[test]
    fn test_weak_keyword_in_title() {
        let strategy = KeywordBasedStrategy::new();
        let news = create_test_news("AI technology trends", "https://example.com/article");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.confidence, 0.3);
        assert!(result.strategy_name.contains("weak"));
    }

    #[test]
    fn test_weak_keyword_in_url() {
        let strategy = KeywordBasedStrategy::new();
        let news = create_test_news("Article about trends", "https://example.com/blockchain-news");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::Block);
        assert_eq!(result.confidence, 0.3);
        assert!(result.strategy_name.contains("weak-url"));
    }

    #[test]
    fn test_no_keywords() {
        let strategy = KeywordBasedStrategy::new();
        let news = create_test_news("General news article", "https://example.com/news");
        
        assert!(strategy.classify(&news).is_none());
    }

    #[test]
    fn test_case_insensitive() {
        let strategy = KeywordBasedStrategy::new();
        let news = create_test_news("BITCOIN price update", "https://example.com/article");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::Block);
    }

    #[test]
    fn test_strong_keyword_takes_precedence() {
        let strategy = KeywordBasedStrategy::new();
        let news = create_test_news("ChatGPT AI assistant", "https://example.com/article");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.confidence, 0.9); // High confidence from "chatgpt"
    }

    #[test]
    fn test_add_custom_strong_keyword() {
        let mut strategy = KeywordBasedStrategy::new();
        strategy.add_strong_keyword(Domain::AI, "custom-tech".to_string());
        
        let news = create_test_news("Custom-tech breakthrough", "https://example.com/article");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    fn test_add_custom_weak_keyword() {
        let mut strategy = KeywordBasedStrategy::new();
        strategy.add_weak_keyword(Domain::Social, "custom-social".to_string());
        
        let news = create_test_news("Custom-social platform", "https://example.com/article");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::Social);
        assert_eq!(result.confidence, 0.3);
    }
}
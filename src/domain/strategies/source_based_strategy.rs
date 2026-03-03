//! # Source-Based Classification Strategy
//!
//! Classifies news based on data source with high confidence.

use crate::domain::{NewsItem, Domain};
use crate::domain::config::ClassificationConfig;
use super::{ClassificationStrategy, ClassificationResult};

/// Strategy that classifies news based on the data source
pub struct SourceBasedStrategy {
    /// Mapping from source names to domains
    source_mapping: std::collections::HashMap<String, Domain>,
}

impl SourceBasedStrategy {
    /// Create a new source-based strategy with default mappings
    pub fn new() -> Self {
        let config = ClassificationConfig::default();
        Self::from_config(config)
    }
    
    /// Create a new source-based strategy from configuration
    pub fn from_config(config: ClassificationConfig) -> Self {
        Self {
            source_mapping: config.source_tendency,
        }
    }
    
    /// Add a custom source mapping
    pub fn add_mapping(&mut self, source: String, domain: Domain) {
        self.source_mapping.insert(source, domain);
    }
}

impl Default for SourceBasedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassificationStrategy for SourceBasedStrategy {
    fn classify(&self, news: &NewsItem) -> Option<ClassificationResult> {
        // Check if the source is mapped
        if let Some(&domain) = self.source_mapping.get(&news.source) {
            return Some(ClassificationResult::high_confidence(
                domain,
                "source-based".to_string(),
            ));
        }
        
        // Also check URL domain for mapped sources
        if let Ok(parsed_url) = url::Url::parse(&news.url) {
            if let Some(host) = parsed_url.host_str() {
                // Check if host is in source mapping
                if let Some(&domain) = self.source_mapping.get(host) {
                    return Some(ClassificationResult::high_confidence(
                        domain,
                        "source-based (url)".to_string(),
                    ));
                }
                
                // Check base domain
                let base_domain = host.split('.').rev().take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(".");
                if let Some(&domain) = self.source_mapping.get(&base_domain) {
                    return Some(ClassificationResult::high_confidence(
                        domain,
                        "source-based (base-domain)".to_string(),
                    ));
                }
            }
        }
        
        None
    }
    
    fn name(&self) -> &str {
        "source-based"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_news(source: &str, url: &str) -> NewsItem {
        NewsItem::new(
            "test-id".to_string(),
            "Test Title".to_string(),
            url.to_string(),
            source.to_string(),
            "test-author".to_string(),
            Utc::now(),
        )
    }

    #[test]
    fn test_source_mapping() {
        let mut strategy = SourceBasedStrategy::new();
        strategy.add_mapping("test-source".to_string(), Domain::AI);
        
        let news = create_test_news("test-source", "https://example.com/article");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.confidence, 0.9);
        assert_eq!(result.strategy_name, "source-based");
    }

    #[test]
    fn test_url_domain_mapping() {
        let mut strategy = SourceBasedStrategy::new();
        strategy.add_mapping("arxiv.org".to_string(), Domain::AI);
        
        let news = create_test_news("unknown", "https://arxiv.org/abs/1234");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.strategy_name, "source-based (url)");
    }

    #[test]
    fn test_base_domain_mapping() {
        let mut strategy = SourceBasedStrategy::new();
        strategy.add_mapping("github.com".to_string(), Domain::AI);
        
        let news = create_test_news("unknown", "https://blog.github.com/article");
        let result = strategy.classify(&news).unwrap();
        
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.strategy_name, "source-based (base-domain)");
    }

    #[test]
    fn test_no_mapping() {
        let strategy = SourceBasedStrategy::new();
        let news = create_test_news("unknown-source", "https://example.com/article");
        
        assert!(strategy.classify(&news).is_none());
    }
}
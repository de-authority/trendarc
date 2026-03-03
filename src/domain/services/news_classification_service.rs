//! # News Classification Service
//!
//! Provides domain classification logic for news items using multiple strategies
//! in a priority-based approach.

use crate::domain::{NewsItem, Domain};
use crate::domain::strategies::{
    ClassificationStrategy, ClassificationResult,
    SourceBasedStrategy, KeywordBasedStrategy
};

/// Service for classifying news items into domains
pub struct NewsClassificationService {
    /// Classification strategies in priority order
    strategies: Vec<Box<dyn ClassificationStrategy>>,
}

impl Default for NewsClassificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl NewsClassificationService {
    /// Create a new classification service with default strategies
    ///
    /// Priority order:
    /// 1. Source-based strategy (highest confidence)
    /// 2. Keyword-based strategy (medium/low confidence)
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(SourceBasedStrategy::new()),
                Box::new(KeywordBasedStrategy::new()),
            ],
        }
    }
    
    /// Create a new classification service with custom strategies
    pub fn with_strategies(strategies: Vec<Box<dyn ClassificationStrategy>>) -> Self {
        Self { strategies }
    }
    
    /// Classify a single news item
    /// 
    /// Uses a multi-strategy approach with priority-based fallback:
    /// 1. Source-based classification (highest confidence)
    /// 2. Keyword-based classification (strong keywords first, then weak)
    /// 3. Default to Uncategorized if no strategy matches
    /// 
    /// Returns the classification result with confidence score
    pub fn classify(&self, news: &NewsItem) -> ClassificationResult {
        // Try each strategy in priority order
        for strategy in &self.strategies {
            if let Some(result) = strategy.classify(news) {
                return result;
            }
        }
        
        // Default: Uncategorized with zero confidence
        ClassificationResult::new(Domain::Uncategorized, 0.0, "default".to_string())
    }
    
    /// Classify multiple news items
    pub fn classify_batch(&self, news_items: &mut [NewsItem]) -> Vec<ClassificationResult> {
        news_items
            .iter_mut()
            .map(|news| {
                let result = self.classify(news);
                news.domain = Some(result.domain);
                news.classification_confidence = Some(result.confidence);
                result
            })
            .collect()
    }
    
    /// Filter news items by domain
    pub fn filter_by_domain(&self, news_items: &[NewsItem], domain: Domain) -> Vec<NewsItem> {
        news_items
            .iter()
            .filter(|news| {
                news.domain.map_or(false, |d| d == domain)
            })
            .cloned()
            .collect()
    }
    
    /// Group news items by domain
    pub fn group_by_domain(&self, news_items: &[NewsItem]) -> std::collections::HashMap<Domain, Vec<NewsItem>> {
        let mut groups: std::collections::HashMap<Domain, Vec<NewsItem>> = std::collections::HashMap::new();
        
        for news in news_items {
            let domain = news.domain.unwrap_or(Domain::Uncategorized);
            groups.entry(domain).or_insert_with(Vec::new).push(news.clone());
        }

        // Ensure all domains are present in the result
        for domain in [Domain::AI, Domain::Block, Domain::Social, Domain::Uncategorized] {
            groups.entry(domain).or_insert_with(Vec::new);
        }

        groups
    }
    
    /// Get classification statistics
    pub fn get_stats(&self, news_items: &[NewsItem]) -> ClassificationStats {
        let mut stats = ClassificationStats::default();
        
        for news in news_items {
            match news.domain {
                Some(Domain::AI) => stats.ai += 1,
                Some(Domain::Block) => stats.block += 1,
                Some(Domain::Social) => stats.social += 1,
                Some(Domain::Uncategorized) => stats.uncategorized += 1,
                None => stats.uncategorized += 1,
            }
            stats.total += 1;
            
            if let Some(confidence) = news.classification_confidence {
                if confidence >= 0.8 {
                    stats.high_confidence += 1;
                } else if confidence >= 0.5 {
                    stats.medium_confidence += 1;
                } else if confidence > 0.0 {
                    stats.low_confidence += 1;
                }
            }
        }
        
        stats
    }
}

/// Classification statistics
#[derive(Debug, Default, Clone)]
pub struct ClassificationStats {
    /// Total number of news items
    pub total: usize,
    /// Number of AI-related news
    pub ai: usize,
    /// Number of blockchain-related news
    pub block: usize,
    /// Number of social media-related news
    pub social: usize,
    /// Number of uncategorized news
    pub uncategorized: usize,
    /// Number of high-confidence classifications (>= 0.8)
    pub high_confidence: usize,
    /// Number of medium-confidence classifications (>= 0.5)
    pub medium_confidence: usize,
    /// Number of low-confidence classifications (> 0.0)
    pub low_confidence: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_news(title: &str, url: &str, source: &str) -> NewsItem {
        NewsItem::new(
            "test-id".to_string(),
            title.to_string(),
            url.to_string(),
            source.to_string(),
            "test-author".to_string(),
            Utc::now(),
        )
    }

    #[test]
    fn test_classify_ai_news_by_keyword() {
        let service = NewsClassificationService::new();
        
        let ai_news = create_test_news(
            "New GPT-4 model released",
            "https://example.com/gpt-4",
            "tech-news",
        );
        
        let result = service.classify(&ai_news);
        assert_eq!(result.domain, Domain::AI);
        assert!(result.confidence >= 0.8);
    }

    #[test]
    fn test_classify_blockchain_news() {
        let service = NewsClassificationService::new();
        
        let crypto_news = create_test_news(
            "Bitcoin reaches new all-time high",
            "https://example.com/btc",
            "crypto-news",
        );
        
        let result = service.classify(&crypto_news);
        assert_eq!(result.domain, Domain::Block);
        assert!(result.confidence >= 0.8);
    }

    #[test]
    fn test_classify_social_news() {
        let service = NewsClassificationService::new();
        
        let social_news = create_test_news(
            "Twitter launches new feature",
            "https://example.com/twitter-feature",
            "social-news",
        );
        
        let result = service.classify(&social_news);
        assert_eq!(result.domain, Domain::Social);
        assert!(result.confidence >= 0.8);
    }

    #[test]
    fn test_classify_by_source() {
        let service = NewsClassificationService::new();
        
        // HackerNews should be classified as AI by source tendency
        let hn_news = create_test_news(
            "General tech article",
            "https://example.com/tech",
            "hackernews",
        );
        
        let result = service.classify(&hn_news);
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    fn test_classify_uncategorized_news() {
        let service = NewsClassificationService::new();
        
        let generic_news = create_test_news(
            "Weather forecast for tomorrow",
            "https://example.com/weather",
            "weather-news",
        );
        
        let result = service.classify(&generic_news);
        assert_eq!(result.domain, Domain::Uncategorized);
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_classify_batch() {
        let service = NewsClassificationService::new();
        
        let mut news_items = vec![
            create_test_news("AI breakthrough", "https://example.com/ai", "news"),
            create_test_news("Crypto news", "https://example.com/crypto", "news"),
            create_test_news("Twitter launches feature", "https://example.com/social", "news"),
            create_test_news("General news", "https://example.com/general", "news"),
        ];
        
        let results = service.classify_batch(&mut news_items);
        
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].domain, Domain::AI);
        assert_eq!(results[1].domain, Domain::Block);
        assert_eq!(results[2].domain, Domain::Social);
        assert_eq!(results[3].domain, Domain::Uncategorized);
        
        // Verify that news items are updated
        assert_eq!(news_items[0].domain, Some(Domain::AI));
        assert_eq!(news_items[1].domain, Some(Domain::Block));
    }

    #[test]
    fn test_filter_by_domain() {
        let service = NewsClassificationService::new();
        
        let mut news_items = vec![
            create_test_news("AI news", "https://example.com/ai", "news"),
            create_test_news("More AI news", "https://example.com/ai2", "news"),
            create_test_news("Crypto news", "https://example.com/crypto", "news"),
        ];
        
        service.classify_batch(&mut news_items);
        
        let ai_news = service.filter_by_domain(&news_items, Domain::AI);
        
        assert_eq!(ai_news.len(), 2);
        assert!(ai_news[0].title.contains("AI"));
        assert!(ai_news[1].title.contains("AI"));
    }

    #[test]
    fn test_group_by_domain() {
        let service = NewsClassificationService::new();
        
        let mut news_items = vec![
            create_test_news("AI news 1", "https://example.com/ai1", "news"),
            create_test_news("AI news 2", "https://example.com/ai2", "news"),
            create_test_news("Crypto news", "https://example.com/crypto", "news"),
            create_test_news("Twitter news", "https://example.com/social", "news"),
        ];
        
        service.classify_batch(&mut news_items);
        
        let groups = service.group_by_domain(&news_items);
        
        assert_eq!(groups.get(&Domain::AI).unwrap().len(), 2);
        assert_eq!(groups.get(&Domain::Block).unwrap().len(), 1);
        assert_eq!(groups.get(&Domain::Social).unwrap().len(), 1);
        assert_eq!(groups.get(&Domain::Uncategorized).unwrap().len(), 0);
    }

    #[test]
    fn test_get_stats() {
        let service = NewsClassificationService::new();
        
        let mut news_items = vec![
            create_test_news("GPT-4 release", "https://example.com/gpt4", "news"),
            create_test_news("AI trends", "https://example.com/ai", "news"),
            create_test_news("Bitcoin news", "https://example.com/btc", "news"),
            create_test_news("Twitter update", "https://example.com/tweet", "news"),
            create_test_news("General news", "https://example.com/general", "news"),
        ];
        
        service.classify_batch(&mut news_items);
        
        let stats = service.get_stats(&news_items);
        
        assert_eq!(stats.total, 5);
        assert_eq!(stats.ai, 2);
        assert_eq!(stats.block, 1);
        assert_eq!(stats.social, 1);
        assert_eq!(stats.uncategorized, 1);
        assert!(stats.high_confidence > 0);
    }
}
//! # News Filter Service
//!
//! Provides filtering functionality for news items based on various criteria.

use crate::domain::{NewsItem, Domain};

/// Configuration for news filtering
#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    /// Filter by specific sources (None = all sources)
    pub sources: Option<Vec<String>>,
    
    /// Filter by specific domains (None = all domains)
    pub domains: Option<Vec<Domain>>,
    
    /// Minimum confidence threshold (None = no threshold)
    pub min_confidence: Option<f32>,
    
    /// Maximum confidence threshold (None = no threshold)
    pub max_confidence: Option<f32>,
}

impl FilterConfig {
    /// Create a new empty filter configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set sources to filter
    pub fn with_sources(mut self, sources: Vec<String>) -> Self {
        self.sources = Some(sources);
        self
    }
    
    /// Set domains to filter
    pub fn with_domains(mut self, domains: Vec<Domain>) -> Self {
        self.domains = Some(domains);
        self
    }
    
    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, min_conf: f32) -> Self {
        self.min_confidence = Some(min_conf);
        self
    }
    
    /// Set maximum confidence threshold
    pub fn with_max_confidence(mut self, max_conf: f32) -> Self {
        self.max_confidence = Some(max_conf);
        self
    }
    
    /// Check if filter is empty (no filters applied)
    pub fn is_empty(&self) -> bool {
        self.sources.is_none() 
            && self.domains.is_none() 
            && self.min_confidence.is_none() 
            && self.max_confidence.is_none()
    }
}

/// Service for filtering news items
pub struct NewsFilterService;

impl NewsFilterService {
    /// Create a new filter service
    pub fn new() -> Self {
        Self
    }
    
    /// Filter news items by sources
    pub fn filter_by_sources(news_items: &[NewsItem], sources: &[&str]) -> Vec<NewsItem> {
        if sources.is_empty() {
            return news_items.to_vec();
        }
        
        news_items
            .iter()
            .filter(|news| sources.contains(&news.source.as_str()))
            .cloned()
            .collect()
    }
    
    /// Filter news items by domains
    pub fn filter_by_domains(news_items: &[NewsItem], domains: &[Domain]) -> Vec<NewsItem> {
        if domains.is_empty() {
            return news_items.to_vec();
        }
        
        news_items
            .iter()
            .filter(|news| {
                news.domain.map_or(false, |d| domains.contains(&d))
            })
            .cloned()
            .collect()
    }
    
    /// Filter news items by confidence range
    pub fn filter_by_confidence(
        news_items: &[NewsItem],
        min: Option<f32>,
        max: Option<f32>,
    ) -> Vec<NewsItem> {
        news_items
            .iter()
            .filter(|news| {
                if let Some(confidence) = news.classification_confidence {
                    let above_min = min.map_or(true, |m| confidence >= m);
                    let below_max = max.map_or(true, |m| confidence <= m);
                    above_min && below_max
                } else {
                    // Items without confidence are filtered out if min > 0
                    min.map_or(true, |m| 0.0 >= m)
                }
            })
            .cloned()
            .collect()
    }
    
    /// Apply a comprehensive filter configuration
    pub fn filter(news_items: &[NewsItem], config: &FilterConfig) -> Vec<NewsItem> {
        if config.is_empty() {
            return news_items.to_vec();
        }
        
        let mut filtered = news_items.to_vec();
        
        // Filter by sources
        if let Some(ref sources) = config.sources {
            let source_refs: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
            filtered = Self::filter_by_sources(&filtered, &source_refs);
        }
        
        // Filter by domains
        if let Some(ref domains) = config.domains {
            filtered = Self::filter_by_domains(&filtered, domains);
        }
        
        // Filter by confidence
        if config.min_confidence.is_some() || config.max_confidence.is_some() {
            filtered = Self::filter_by_confidence(
                &filtered,
                config.min_confidence,
                config.max_confidence,
            );
        }
        
        filtered
    }
    
    /// Filter and keep only high-confidence items (>= 0.8)
    pub fn filter_high_confidence(news_items: &[NewsItem]) -> Vec<NewsItem> {
        Self::filter_by_confidence(news_items, Some(0.8), None)
    }
    
    /// Filter and keep only medium-confidence items (0.5 - 0.8)
    pub fn filter_medium_confidence(news_items: &[NewsItem]) -> Vec<NewsItem> {
        Self::filter_by_confidence(news_items, Some(0.5), Some(0.8))
    }
    
    /// Filter and keep only low-confidence items (< 0.5)
    pub fn filter_low_confidence(news_items: &[NewsItem]) -> Vec<NewsItem> {
        Self::filter_by_confidence(news_items, Some(0.0), Some(0.5))
    }
    
    /// Filter and exclude uncategorized items
    pub fn exclude_uncategorized(news_items: &[NewsItem]) -> Vec<NewsItem> {
        news_items
            .iter()
            .filter(|news| {
                news.domain.map_or(false, |d| d != Domain::Uncategorized)
            })
            .cloned()
            .collect()
    }
}

impl Default for NewsFilterService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_news(
        title: &str,
        url: &str,
        source: &str,
        domain: Option<Domain>,
        confidence: Option<f32>,
    ) -> NewsItem {
        let mut news = NewsItem::new(
            "test-id".to_string(),
            title.to_string(),
            url.to_string(),
            source.to_string(),
            "test-author".to_string(),
            Utc::now(),
        );
        news.domain = domain;
        news.classification_confidence = confidence;
        news
    }

    #[test]
    fn test_filter_by_sources() {
        let news_items = vec![
            create_test_news("News 1", "url1", "hackernews", None, None),
            create_test_news("News 2", "url2", "twitter", None, None),
            create_test_news("News 3", "url3", "hackernews", None, None),
        ];
        
        let filtered = NewsFilterService::filter_by_sources(&news_items, &["hackernews"]);
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|n| n.source == "hackernews"));
    }

    #[test]
    fn test_filter_by_domains() {
        let news_items = vec![
            create_test_news("AI news", "url1", "source1", Some(Domain::AI), Some(0.9)),
            create_test_news("Crypto news", "url2", "source2", Some(Domain::Block), Some(0.9)),
            create_test_news("Social news", "url3", "source3", Some(Domain::Social), Some(0.9)),
        ];
        
        let filtered = NewsFilterService::filter_by_domains(&news_items, &[Domain::AI, Domain::Block]);
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|n| {
            n.domain.map_or(false, |d| d == Domain::AI || d == Domain::Block)
        }));
    }

    #[test]
    fn test_filter_by_confidence() {
        let news_items = vec![
            create_test_news("High conf", "url1", "s1", Some(Domain::AI), Some(0.9)),
            create_test_news("Med conf", "url2", "s2", Some(Domain::AI), Some(0.6)),
            create_test_news("Low conf", "url3", "s3", Some(Domain::AI), Some(0.3)),
        ];
        
        let filtered = NewsFilterService::filter_by_confidence(&news_items, Some(0.7), None);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].classification_confidence, Some(0.9));
    }

    #[test]
    fn test_filter_with_config() {
        let news_items = vec![
            create_test_news("HN AI", "url1", "hackernews", Some(Domain::AI), Some(0.9)),
            create_test_news("Twitter AI", "url2", "twitter", Some(Domain::AI), Some(0.6)),
            create_test_news("HN Block", "url3", "hackernews", Some(Domain::Block), Some(0.9)),
        ];
        
        let config = FilterConfig::new()
            .with_sources(vec!["hackernews".to_string()])
            .with_domains(vec![Domain::AI]);
        
        let filtered = NewsFilterService::filter(&news_items, &config);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source, "hackernews");
        assert_eq!(filtered[0].domain, Some(Domain::AI));
    }

    #[test]
    fn test_filter_empty_config() {
        let news_items = vec![
            create_test_news("News 1", "url1", "s1", Some(Domain::AI), Some(0.9)),
            create_test_news("News 2", "url2", "s2", Some(Domain::Block), Some(0.6)),
        ];
        
        let filtered = NewsFilterService::filter(&news_items, &FilterConfig::new());
        
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_high_confidence() {
        let news_items = vec![
            create_test_news("High", "url1", "s1", Some(Domain::AI), Some(0.9)),
            create_test_news("Med", "url2", "s2", Some(Domain::AI), Some(0.6)),
            create_test_news("Low", "url3", "s3", Some(Domain::AI), Some(0.3)),
        ];
        
        let filtered = NewsFilterService::filter_high_confidence(&news_items);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].classification_confidence, Some(0.9));
    }

    #[test]
    fn test_filter_medium_confidence() {
        let news_items = vec![
            create_test_news("High", "url1", "s1", Some(Domain::AI), Some(0.9)),
            create_test_news("Med", "url2", "s2", Some(Domain::AI), Some(0.6)),
            create_test_news("Low", "url3", "s3", Some(Domain::AI), Some(0.3)),
        ];
        
        let filtered = NewsFilterService::filter_medium_confidence(&news_items);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].classification_confidence, Some(0.6));
    }

    #[test]
    fn test_filter_low_confidence() {
        let news_items = vec![
            create_test_news("High", "url1", "s1", Some(Domain::AI), Some(0.9)),
            create_test_news("Med", "url2", "s2", Some(Domain::AI), Some(0.6)),
            create_test_news("Low", "url3", "s3", Some(Domain::AI), Some(0.3)),
        ];
        
        let filtered = NewsFilterService::filter_low_confidence(&news_items);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].classification_confidence, Some(0.3));
    }

    #[test]
    fn test_exclude_uncategorized() {
        let news_items = vec![
            create_test_news("AI", "url1", "s1", Some(Domain::AI), Some(0.9)),
            create_test_news("Uncategorized", "url2", "s2", Some(Domain::Uncategorized), None),
            create_test_news("Block", "url3", "s3", Some(Domain::Block), Some(0.9)),
        ];
        
        let filtered = NewsFilterService::exclude_uncategorized(&news_items);
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|n| n.domain != Some(Domain::Uncategorized)));
    }

    #[test]
    fn test_filter_config_builder() {
        let config = FilterConfig::new()
            .with_sources(vec!["hackernews".to_string()])
            .with_domains(vec![Domain::AI])
            .with_min_confidence(0.8)
            .with_max_confidence(1.0);
        
        assert_eq!(config.sources, Some(vec!["hackernews".to_string()]));
        assert_eq!(config.domains, Some(vec![Domain::AI]));
        assert_eq!(config.min_confidence, Some(0.8));
        assert_eq!(config.max_confidence, Some(1.0));
        assert!(!config.is_empty());
    }

    #[test]
    fn test_filter_config_is_empty() {
        let config = FilterConfig::new();
        assert!(config.is_empty());
    }
}
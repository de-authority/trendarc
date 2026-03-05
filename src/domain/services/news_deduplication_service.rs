//! # News Deduplication Service
//!
//! Provides deduplication logic for news items based on various criteria.

use crate::domain::NewsItem;
use std::collections::HashSet;

/// Service for deduplicating news items
pub struct NewsDeduplicationService;

impl NewsDeduplicationService {
    /// Deduplicate news items by URL
    ///
    /// Keeps only the first occurrence of each unique URL.
    pub fn deduplicate_by_url(news: Vec<NewsItem>) -> Vec<NewsItem> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut result = Vec::new();

        for item in news {
            if seen.insert(item.url.clone()) {
                result.push(item);
            }
        }

        result
    }

    /// Deduplicate news items by title
    ///
    /// Keeps only the first occurrence of each unique title.
    #[allow(dead_code)]
    pub fn deduplicate_by_title(news: Vec<NewsItem>) -> Vec<NewsItem> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut result = Vec::new();

        for item in news {
            if seen.insert(item.title.clone()) {
                result.push(item);
            }
        }

        result
    }

    /// Deduplicate news items by both URL and title
    ///
    /// Keeps only items that have unique combinations of URL and title.
    #[allow(dead_code)]
    pub fn deduplicate_by_url_and_title(news: Vec<NewsItem>) -> Vec<NewsItem> {
        let mut seen: HashSet<(String, String)> = HashSet::new();
        let mut result = Vec::new();

        for item in news {
            let key = (item.url.clone(), item.title.clone());
            if seen.insert(key) {
                result.push(item);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    fn create_test_news_item(id: &str, url: &str, title: &str) -> NewsItem {
        NewsItem::new(
            id.to_string(),
            title.to_string(),
            url.to_string(),
            "test_source".to_string(),
            "test_author".to_string(),
            Utc::now(),
        )
    }
    #[test]
    fn test_deduplicate_by_url_removes_duplicates() {
        let news = vec![
            create_test_news_item("1", "http://example.com/1", "Title 1"),
            create_test_news_item("2", "http://example.com/1", "Title 2"), // Same URL
            create_test_news_item("3", "http://example.com/2", "Title 3"),
        ];

        let result = NewsDeduplicationService::deduplicate_by_url(news);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].url, "http://example.com/1");
        assert_eq!(result[1].url, "http://example.com/2");
    }

    #[test]
    fn test_deduplicate_by_url_keeps_first_occurrence() {
        let news = vec![
            create_test_news_item("1", "http://example.com/1", "Title 1"),
            create_test_news_item("2", "http://example.com/1", "Title 2"), // Duplicate
        ];

        let result = NewsDeduplicationService::deduplicate_by_url(news);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Title 1"); // Should keep the first one
    }

    #[test]
    fn test_deduplicate_by_title_removes_duplicates() {
        let news = vec![
            create_test_news_item("1", "http://example.com/1", "Same Title"),
            create_test_news_item("2", "http://example.com/2", "Same Title"), // Same title
            create_test_news_item("3", "http://example.com/3", "Different Title"),
        ];

        let result = NewsDeduplicationService::deduplicate_by_title(news);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Same Title");
        assert_eq!(result[1].title, "Different Title");
    }

    #[test]
    fn test_deduplicate_by_url_and_title() {
        let news = vec![
            create_test_news_item("1", "http://example.com/1", "Title A"),
            create_test_news_item("2", "http://example.com/1", "Title A"), // Same URL and title
            create_test_news_item("3", "http://example.com/1", "Title B"), // Same URL, different title
            create_test_news_item("4", "http://example.com/2", "Title A"), // Different URL, same title
            create_test_news_item("5", "http://example.com/2", "Title B"), // Same as above
        ];

        let result = NewsDeduplicationService::deduplicate_by_url_and_title(news);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].title, "Title A");
        assert_eq!(result[1].title, "Title B");
        assert_eq!(result[2].title, "Title A");
        assert_eq!(result[3].title, "Title B");
    }

    #[test]
    fn test_deduplicate_empty_vector() {
        let news: Vec<NewsItem> = vec![];
        let result = NewsDeduplicationService::deduplicate_by_url(news);
        assert!(result.is_empty());
    }

    #[test]
    fn test_deduplicate_single_item() {
        let news = vec![create_test_news_item(
            "1",
            "http://example.com/1",
            "Title 1",
        )];
        let result = NewsDeduplicationService::deduplicate_by_url(news);
        assert_eq!(result.len(), 1);
    }
}

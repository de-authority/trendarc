//! # Classification Strategy Trait
//!
//! Defines the interface for all classification strategies.

use crate::domain::{NewsItem, Domain};

/// Result of a classification attempt
#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationResult {
    /// The classified domain
    pub domain: Domain,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// The strategy that produced this result
    pub strategy_name: String,
}

impl ClassificationResult {
    /// Create a new classification result
    pub fn new(domain: Domain, confidence: f32, strategy_name: String) -> Self {
        Self {
            domain,
            confidence,
            strategy_name,
        }
    }
    
    /// Create a high-confidence result
    pub fn high_confidence(domain: Domain, strategy_name: String) -> Self {
        Self {
            domain,
            confidence: 0.9,
            strategy_name,
        }
    }
    
    /// Create a medium-confidence result
    pub fn medium_confidence(domain: Domain, strategy_name: String) -> Self {
        Self {
            domain,
            confidence: 0.6,
            strategy_name,
        }
    }
    
    /// Create a low-confidence result
    pub fn low_confidence(domain: Domain, strategy_name: String) -> Self {
        Self {
            domain,
            confidence: 0.3,
            strategy_name,
        }
    }
}

/// Trait for news classification strategies
///
/// Each strategy should implement this trait to provide
/// its own classification logic.
pub trait ClassificationStrategy: Send + Sync {
    /// Attempt to classify a news item
    ///
    /// Returns:
    /// - `Some(result)` if the strategy can classify the item
    /// - `None` if the strategy cannot classify the item
    fn classify(&self, news: &NewsItem) -> Option<ClassificationResult>;
    
    /// Get the name of this strategy
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_result_creation() {
        let result = ClassificationResult::new(Domain::AI, 0.85, "test".to_string());
        assert_eq!(result.domain, Domain::AI);
        assert_eq!(result.confidence, 0.85);
        assert_eq!(result.strategy_name, "test");
    }

    #[test]
    fn test_high_confidence_result() {
        let result = ClassificationResult::high_confidence(Domain::AI, "test".to_string());
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    fn test_medium_confidence_result() {
        let result = ClassificationResult::medium_confidence(Domain::Block, "test".to_string());
        assert_eq!(result.confidence, 0.6);
    }

    #[test]
    fn test_low_confidence_result() {
        let result = ClassificationResult::low_confidence(Domain::Social, "test".to_string());
        assert_eq!(result.confidence, 0.3);
    }
}
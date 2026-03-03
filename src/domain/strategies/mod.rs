//! # Classification Strategies
//!
//! This module contains various classification strategies for news items.
//! Each strategy implements the `ClassificationStrategy` trait.

pub mod classification_strategy;
pub mod source_based_strategy;
pub mod keyword_based_strategy;

pub use classification_strategy::{
    ClassificationStrategy,
    ClassificationResult,
};

pub use source_based_strategy::SourceBasedStrategy;
pub use keyword_based_strategy::KeywordBasedStrategy;
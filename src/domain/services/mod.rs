pub mod content_extractor;
pub mod discord_service;
pub mod news_classification_service;
pub mod news_deduplication_service;
pub mod news_inference_service;
pub mod news_sorting_service;

pub use content_extractor::{ContentExtractor, DefaultContentExtractor};
pub use discord_service::{DiscordMessage, DiscordService};
pub use news_classification_service::NewsClassificationService;
pub use news_deduplication_service::NewsDeduplicationService;
pub use news_inference_service::{InferenceResult, NewsInferenceService};
pub use news_sorting_service::NewsSortingService;

#[cfg(test)]
mod classification_redesign_tests;

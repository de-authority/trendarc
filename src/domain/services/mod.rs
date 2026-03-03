pub mod news_deduplication_service;
pub mod news_sorting_service;
pub mod news_classification_service;
pub mod news_filter_service;

pub use news_deduplication_service::NewsDeduplicationService;
pub use news_sorting_service::NewsSortingService;
pub use news_classification_service::{NewsClassificationService, ClassificationStats};
pub use news_filter_service::{NewsFilterService, FilterConfig};

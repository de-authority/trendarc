pub mod entities;
pub mod fetchers;
pub mod repositories;
pub mod services;
pub mod strategies;
pub mod config;

// 重新导出常用的类型，方便使用
pub use entities::{NewsItem, Domain};
pub use fetchers::NewsFetcher;
pub use repositories::NewsRepository;
pub use services::{
    NewsDeduplicationService, 
    NewsSortingService, 
    NewsClassificationService,
    ClassificationStats,
    NewsFilterService,
    FilterConfig,
};
pub use strategies::{ClassificationStrategy, ClassificationResult};
pub use config::ClassificationConfig;

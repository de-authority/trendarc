pub mod config;
pub mod entities;
pub mod fetchers;
pub mod repositories;
pub mod services;
pub mod strategies;

// 重新导出常用的类型，方便使用
pub use entities::{Domain, NewsItem};
pub use fetchers::NewsFetcher;
pub use repositories::NewsRepository;
pub use services::{NewsClassificationService, NewsDeduplicationService, NewsSortingService};

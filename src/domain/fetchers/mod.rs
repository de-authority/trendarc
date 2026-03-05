pub mod news_fetcher;
pub mod source_factory;

pub use news_fetcher::NewsFetcher;
pub use source_factory::{NewsSourceFactory, CompositeNewsFetcher};
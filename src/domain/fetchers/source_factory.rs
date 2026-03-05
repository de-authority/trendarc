use crate::cli::DataSource;
use crate::infrastructure::news_sources::HackerNewsSource;
use crate::domain::NewsFetcher;
use std::sync::Arc;

/// 数据源工厂
pub struct NewsSourceFactory;

impl NewsSourceFactory {
    /// 根据 DataSource 枚举创建对应的 NewsFetcher
    pub fn create(source: DataSource) -> Arc<dyn NewsFetcher> {
        match source {
            DataSource::All => {
                // 当前只有 HackerNews，为未来扩展预留
                let fetchers = vec![
                    Arc::new(HackerNewsSource::new()) as Arc<dyn NewsFetcher>
                ];
                Arc::new(CompositeNewsFetcher::new(fetchers))
            }
            DataSource::HackerNews => {
                Arc::new(HackerNewsSource::new())
            }
        }
    }
}

/// 组合数据源，支持从多个数据源并发抓取
pub struct CompositeNewsFetcher {
    fetchers: Vec<Arc<dyn NewsFetcher>>,
}

impl CompositeNewsFetcher {
    /// 创建新的组合数据源
    pub fn new(fetchers: Vec<Arc<dyn NewsFetcher>>) -> Self {
        Self { fetchers }
    }
}

use async_trait::async_trait;
use crate::domain::NewsItem;
use futures::future::join_all;

#[async_trait]
impl NewsFetcher for CompositeNewsFetcher {
    async fn fetch(
        &self,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        // 如果只有一个数据源，直接使用它
        if self.fetchers.len() == 1 {
            return self.fetchers[0].fetch(limit).await;
        }

        // 多数据源智能分配算法
        let mut all_results = Vec::new();
        let mut remaining_limit = limit;
        
        // 基础分配：每个数据源分配基础配额
        let base_per_source = if !self.fetchers.is_empty() {
            (limit as f32 / self.fetchers.len() as f32).ceil() as usize
        } else {
            0
        };
        
        // 第一轮抓取
        let mut futures = Vec::new();
        for fetcher in &self.fetchers {
            let fetch_limit = base_per_source.min(remaining_limit);
            if fetch_limit > 0 {
                futures.push(fetcher.fetch(fetch_limit));
            }
        }
        
        let results = join_all(futures).await;
        for result in results {
            if let Ok(items) = result {
                all_results.extend(items);
            }
        }
        
        // 去重和排序
        use crate::domain::{NewsDeduplicationService, NewsSortingService};
        all_results = NewsDeduplicationService::deduplicate_by_url(all_results);
        all_results = NewsSortingService::sort_by_published_at_desc(all_results);
        
        // 如果数量不足，尝试补充抓取
        if all_results.len() < limit && !self.fetchers.is_empty() {
            let mut remaining_sources = self.fetchers.clone();
            let mut iteration = 0;
            const MAX_ITERATIONS: usize = 3; // 最多尝试3轮
            
            while all_results.len() < limit && !remaining_sources.is_empty() && iteration < MAX_ITERATIONS {
                let needed = limit - all_results.len();
                let per_source = (needed as f32 / remaining_sources.len() as f32).ceil() as usize;
                
                let mut futures = Vec::new();
                for fetcher in &remaining_sources {
                    futures.push(fetcher.fetch(per_source));
                }
                
                let batch_results = join_all(futures).await;
                let mut batch_items = Vec::new();
                for result in batch_results {
                    if let Ok(items) = result {
                        batch_items.extend(items);
                    }
                }
                
                // 合并并去重
                all_results.extend(batch_items);
                all_results = NewsDeduplicationService::deduplicate_by_url(all_results);
                all_results = NewsSortingService::sort_by_published_at_desc(all_results);
                
                iteration += 1;
            }
        }
        
        // 确保不超过 limit
        if all_results.len() > limit {
            all_results.truncate(limit);
        }
        
        Ok(all_results)
    }

    fn source_name(&self) -> &str {
        "composite"
    }
}
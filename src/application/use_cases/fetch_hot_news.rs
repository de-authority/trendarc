use crate::domain::{
    NewsClassificationService, NewsDeduplicationService, NewsFetcher, NewsItem, NewsRepository,
    NewsSortingService,
};
use async_trait::async_trait;
use std::sync::Arc;

/// 获取热点新闻用例
///
/// **职责**：
/// - 编排"获取热点新闻"这个业务流程
/// - 依赖 `NewsFetcher` 接口，不关心具体实现
/// - 对获取的新闻进行去重、排序
/// - 可选地保存到数据库（通过 Repository）
///
/// **为什么在 Application 层而不是 Domain 层？**
/// - 这是一个"用例"，是应用级别的流程编排
/// - 不涉及核心业务规则（那是 Domain 层的事）
///
/// **关于分类**：
/// - 分类功能通过依赖注入的 `NewsClassificationService` 提供
/// - 如果提供了 Repository，分类结果会一并保存到数据库
///
/// **关于持久化**：
/// - Repository 是可选的，通过 `with_repository()` 方法注入
/// - 如果提供了 Repository，新闻和领域分类会自动保存到数据库
/// - 如果没有提供 Repository，则只抓取不保存（保持向后兼容）
#[async_trait]
pub trait FetchHotNewsUseCase: Send + Sync {
    async fn execute(
        &self,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认实现
pub struct FetchHotNewsService<'a> {
    fetcher: &'a dyn NewsFetcher,
    classifier: Arc<NewsClassificationService>,
    repository: Option<Arc<dyn NewsRepository>>,
}

impl<'a> FetchHotNewsService<'a> {
    pub fn new(fetcher: &'a dyn NewsFetcher, classifier: Arc<NewsClassificationService>) -> Self {
        Self {
            fetcher,
            classifier,
            repository: None,
        }
    }

    /// 设置 Repository（可选）
    ///
    /// # 示例
    /// ```ignore
    /// let use_case = FetchHotNewsService::new(&fetcher, classifier)
    ///     .with_repository(Arc::new(SqliteNewsRepository::new(pool)));
    /// ```
    pub fn with_repository(mut self, repository: Arc<dyn NewsRepository>) -> Self {
        self.repository = Some(repository);
        self
    }
}

use tracing::info;

#[async_trait]
impl<'a> FetchHotNewsUseCase for FetchHotNewsService<'a> {
    async fn execute(
        &self,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        // 向后兼容：默认执行分类
        self.execute_with_classification(limit).await
    }
}

impl<'a> FetchHotNewsService<'a> {
    /// 执行抓取并分类（优化版本）
    pub async fn execute_with_classification(
        &self,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "📡 从 {} 获取热点新闻（执行分类）...",
            self.fetcher.source_name()
        );

        // 1. 获取数据
        let news = self.fetcher.fetch(limit).await?;
        info!("📰 抓取到 {} 条原始新闻", news.len());

        // 2. 过滤掉数据库中已存在的新闻（优化：批量检查）
        let (filtered_news, skipped_count) = if let Some(ref repo) = self.repository {
            // 提取所有 URL
            let urls: Vec<String> = news.iter().map(|n| n.url.clone()).collect();

            // 批量查询已存在的 URL
            let existing_urls = repo.find_existing_urls(&urls).await?;

            // 过滤掉已存在的新闻
            let filtered: Vec<NewsItem> = news
                .into_iter()
                .filter(|n| !existing_urls.contains(&n.url))
                .collect();

            let skipped = urls.len() - filtered.len();

            if skipped > 0 {
                info!("⏭️  忽略 {} 条已存在于数据库的新闻", skipped);
            }

            (filtered, skipped)
        } else {
            (news, 0)
        };

        if filtered_news.is_empty() {
            info!("✅ 没有新新闻需要处理");
            return Ok(Vec::new());
        }

        info!("🔍 需要处理 {} 条新新闻", filtered_news.len());

        // 3. 去重（内存中的去重，以防抓取到重复的 URL）
        let filtered_len = filtered_news.len();
        let unique_news = NewsDeduplicationService::deduplicate_by_url(filtered_news);
        if unique_news.len() != filtered_len {
            info!(
                "🧹 内存中去重，过滤掉 {} 条重复项",
                filtered_len - unique_news.len()
            );
        }

        // 4. 排序（按时间，最新的在前）
        let sorted_news = NewsSortingService::sort_by_published_at_desc(unique_news);

        // 5. 分类新闻并过滤掉无关项
        let mut news_items = sorted_news;
        self.classifier
            .classify_batch_and_filter(&mut news_items)
            .await;

        // 6. 保存到数据库（如果提供了 Repository）
        if let Some(ref repo) = self.repository
            && !news_items.is_empty()
        {
            info!("💾 保存 {} 条新新闻到数据库...", news_items.len());
            repo.save_batch(&news_items).await?;
            info!("✅ 保存完成！");
        }

        info!(
            "✅ 获取完成！共处理 {} 条新新闻（忽略 {} 条已存在）",
            news_items.len(),
            skipped_count
        );

        Ok(news_items)
    }

    /// 执行抓取但不分类（优化版本）
    pub async fn execute_without_classification(
        &self,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "📡 从 {} 获取热点新闻（不执行分类）...",
            self.fetcher.source_name()
        );

        // 1. 获取数据
        let news = self.fetcher.fetch(limit).await?;
        info!("📰 抓取到 {} 条原始新闻", news.len());

        // 2. 过滤掉数据库中已存在的新闻（优化：批量检查）
        let (filtered_news, skipped_count) = if let Some(ref repo) = self.repository {
            // 提取所有 URL
            let urls: Vec<String> = news.iter().map(|n| n.url.clone()).collect();

            // 批量查询已存在的 URL
            let existing_urls = repo.find_existing_urls(&urls).await?;

            // 过滤掉已存在的新闻
            let filtered: Vec<NewsItem> = news
                .into_iter()
                .filter(|n| !existing_urls.contains(&n.url))
                .collect();

            let skipped = urls.len() - filtered.len();

            if skipped > 0 {
                info!("⏭️  忽略 {} 条已存在于数据库的新闻", skipped);
            }

            (filtered, skipped)
        } else {
            (news, 0)
        };

        if filtered_news.is_empty() {
            info!("✅ 没有新新闻需要处理");
            return Ok(Vec::new());
        }

        info!("🔍 需要处理 {} 条新新闻", filtered_news.len());

        // 3. 去重（内存中的去重，以防抓取到重复的 URL）
        let filtered_len = filtered_news.len();
        let unique_news = NewsDeduplicationService::deduplicate_by_url(filtered_news);
        if unique_news.len() != filtered_len {
            info!(
                "🧹 内存中去重，过滤掉 {} 条重复项",
                filtered_len - unique_news.len()
            );
        }

        // 4. 排序（按时间，最新的在前）
        let sorted_news = NewsSortingService::sort_by_published_at_desc(unique_news);

        // 5. 不执行分类，所有新闻都保留
        let news_items = sorted_news;

        // 6. 保存到数据库（如果提供了 Repository）
        if let Some(ref repo) = self.repository
            && !news_items.is_empty()
        {
            info!("💾 保存 {} 条新新闻到数据库...", news_items.len());
            repo.save_batch(&news_items).await?;
            info!("✅ 保存完成！");
        }

        info!(
            "✅ 获取完成！共处理 {} 条新新闻（忽略 {} 条已存在）",
            news_items.len(),
            skipped_count
        );

        Ok(news_items)
    }
}

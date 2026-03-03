use crate::domain::{NewsFetcher, NewsItem, NewsDeduplicationService, NewsSortingService, NewsRepository, NewsClassificationService};
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
    async fn execute(&self, limit: usize) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认实现
pub struct FetchHotNewsService<'a> {
    fetcher: &'a dyn NewsFetcher,
    classifier: Arc<NewsClassificationService>,
    repository: Option<Arc<dyn NewsRepository>>,
}

impl<'a> FetchHotNewsService<'a> {
    pub fn new(
        fetcher: &'a dyn NewsFetcher,
        classifier: Arc<NewsClassificationService>,
    ) -> Self {
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

#[async_trait]
impl<'a> FetchHotNewsUseCase for FetchHotNewsService<'a> {
    async fn execute(&self, limit: usize) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        println!("📡 从 {} 获取热点新闻...\n", self.fetcher.source_name());

        // 1. 获取数据
        let news = self.fetcher.fetch(limit).await?;

        // 2. 去重（按 URL）
        let unique_news = NewsDeduplicationService::deduplicate_by_url(news);

        // 3. 排序（按时间，最新的在前）
        let sorted_news = NewsSortingService::sort_by_published_at_desc(unique_news);

        // 4. 保存到数据库（如果提供了 Repository）
        if let Some(ref repo) = self.repository {
            println!("💾 保存新闻到数据库...");
            // 对新闻进行分类并保存
            let mut news_to_save = sorted_news.clone();
            self.classifier.classify_batch(&mut news_to_save);
            repo.save_batch(&news_to_save).await?;
            println!("✅ 保存完成！\n");
        }

        // 5. 分类（可选：在展示时使用分类器）
        // 注意：分类不修改 NewsItem，只是在展示时使用
        
        println!("✅ 获取完成！共 {} 条新闻（已去重）\n", sorted_news.len());

        Ok(sorted_news)
    }
}
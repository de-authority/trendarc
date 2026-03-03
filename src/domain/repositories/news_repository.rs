use crate::domain::{Domain, NewsItem};
use async_trait::async_trait;

/// 新闻仓库接口
///
/// 定义在 Domain 层，因为数据持久化是业务需求的一部分
/// Infrastructure 层提供具体实现（如 SQLite、PostgreSQL）
#[async_trait]
pub trait NewsRepository: Send + Sync {
    /// 保存单条新闻（包含领域分类）
    /// 如果 URL 已存在则跳过（静默失败）
    async fn save(&self, news: &NewsItem) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 批量保存新闻（包含领域分类）
    async fn save_batch(
        &self,
        news_items: &[NewsItem],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 根据 ID 查询新闻
    async fn find_by_id(
        &self,
        id: &str,
    ) -> Result<Option<NewsItem>, Box<dyn std::error::Error + Send + Sync>>;

    /// 根据领域查询新闻
    async fn find_by_domain(
        &self,
        domain: Domain,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>>;

    /// 根据多个领域查询新闻
    async fn find_by_domains(
        &self,
        domains: &[Domain],
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>>;

    /// 查询最近的新闻（按发布时间降序）
    async fn find_recent(
        &self,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>>;

    /// 根据 URL 查询新闻（用于去重）
    async fn find_by_url(
        &self,
        url: &str,
    ) -> Result<Option<NewsItem>, Box<dyn std::error::Error + Send + Sync>>;

    /// 统计新闻总数
    async fn count(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;

    /// 统计各领域的新闻数量
    async fn count_by_domain(
        &self,
    ) -> Result<Vec<(Domain, usize)>, Box<dyn std::error::Error + Send + Sync>>;
}

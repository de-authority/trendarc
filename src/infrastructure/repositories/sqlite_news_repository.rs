use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::domain::{Domain, NewsItem, NewsRepository};

/// SQLite 实现的新闻仓库
/// 
/// Infrastructure 层的具体实现，使用 SQLx 进行异步数据库操作
/// 只负责数据持久化，不包含业务逻辑
pub struct SqliteNewsRepository {
    pool: SqlitePool,
}

impl SqliteNewsRepository {
    /// 创建新的 SQLite 仓库实例
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NewsRepository for SqliteNewsRepository {
    async fn save(&self, news: &NewsItem) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let domain_str = news.domain.map(|d| d.to_string());
        let confidence = news.classification_confidence;
        
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO news_items (id, title, url, source, author, published_at, domain, classification_confidence)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#
        )
        .bind(&news.id)
        .bind(&news.title)
        .bind(&news.url)
        .bind(&news.source)
        .bind(&news.author)
        .bind(news.published_at.to_rfc3339())
        .bind(&domain_str)
        .bind(confidence)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_batch(&self, news_items: &[NewsItem]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = self.pool.begin().await?;
        
        for news in news_items {
            let domain_str = news.domain.map(|d| d.to_string());
            let confidence = news.classification_confidence;
            
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO news_items (id, title, url, source, author, published_at, domain, classification_confidence)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#
            )
            .bind(&news.id)
            .bind(&news.title)
            .bind(&news.url)
            .bind(&news.source)
            .bind(&news.author)
            .bind(news.published_at.to_rfc3339())
            .bind(&domain_str)
            .bind(confidence)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<f32>)>(
            "SELECT id, title, url, source, author, published_at, domain, classification_confidence FROM news_items WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((id, title, url, source, author, published_at, domain_str, confidence)) => {
                let published_at = chrono::DateTime::parse_from_rfc3339(&published_at)?
                    .with_timezone(&chrono::Utc);
                let domain = domain_str.and_then(|s| parse_domain(&s));
                
                Ok(Some(NewsItem::new_with_classification(
                    id, title, url, source, author, published_at,
                    domain.unwrap_or(Domain::Uncategorized),
                    confidence.unwrap_or(0.0),
                )))
            }
            None => Ok(None),
        }
    }

    async fn find_by_domain(&self, domain: Domain, limit: usize) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        let domain_str = domain.to_string();
        
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<f32>)>(
            r#"
            SELECT id, title, url, source, author, published_at, domain, classification_confidence
            FROM news_items
            WHERE domain = ?1
            ORDER BY published_at DESC
            LIMIT ?2
            "#
        )
        .bind(&domain_str)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut news_items = Vec::new();
        for (id, title, url, source, author, published_at, domain_str, confidence) in rows {
            let published_at = chrono::DateTime::parse_from_rfc3339(&published_at)?
                .with_timezone(&chrono::Utc);
            let parsed_domain = domain_str.and_then(|s| parse_domain(&s));
            
            news_items.push(NewsItem::new_with_classification(
                id, title, url, source, author, published_at,
                parsed_domain.unwrap_or(Domain::Uncategorized),
                confidence.unwrap_or(0.0),
            ));
        }

        Ok(news_items)
    }

    async fn find_recent(&self, limit: usize) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<f32>)>(
            "SELECT id, title, url, source, author, published_at, domain, classification_confidence FROM news_items ORDER BY published_at DESC LIMIT ?1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut news_items = Vec::new();
        for (id, title, url, source, author, published_at, domain_str, confidence) in rows {
            let published_at = chrono::DateTime::parse_from_rfc3339(&published_at)?
                .with_timezone(&chrono::Utc);
            let domain = domain_str.and_then(|s| parse_domain(&s));
            
            news_items.push(NewsItem::new_with_classification(
                id, title, url, source, author, published_at,
                domain.unwrap_or(Domain::Uncategorized),
                confidence.unwrap_or(0.0),
            ));
        }

        Ok(news_items)
    }

    async fn find_by_url(&self, url: &str) -> Result<Option<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, Option<f32>)>(
            "SELECT id, title, url, source, author, published_at, domain, classification_confidence FROM news_items WHERE url = ?1"
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((id, title, url, source, author, published_at, domain_str, confidence)) => {
                let published_at = chrono::DateTime::parse_from_rfc3339(&published_at)?
                    .with_timezone(&chrono::Utc);
                let domain = domain_str.and_then(|s| parse_domain(&s));
                
                Ok(Some(NewsItem::new_with_classification(
                    id, title, url, source, author, published_at,
                    domain.unwrap_or(Domain::Uncategorized),
                    confidence.unwrap_or(0.0),
                )))
            }
            None => Ok(None),
        }
    }

    async fn count(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM news_items")
            .fetch_one(&self.pool)
            .await?;
        Ok(count as usize)
    }

    async fn count_by_domain(&self) -> Result<Vec<(Domain, usize)>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query_as::<_, (Option<String>, i64)>(
            r#"
            SELECT domain, COUNT(*) as count
            FROM news_items
            GROUP BY domain
            ORDER BY count DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for (domain_str, count) in rows {
            let domain = domain_str
                .and_then(|s| parse_domain(&s))
                .unwrap_or(Domain::Uncategorized);
            result.push((domain, count as usize));
        }

        Ok(result)
    }
}

/// 解析域名字符串为 Domain 枚举
fn parse_domain(s: &str) -> Option<Domain> {
    match s {
        "AI" => Some(Domain::AI),
        "Block" => Some(Domain::Block),
        "Social" => Some(Domain::Social),
        "Uncategorized" => Some(Domain::Uncategorized),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::create_pool;
    use chrono::Utc;

    fn create_test_news(id: &str, title: &str, url: &str, domain: Option<Domain>) -> NewsItem {
        match domain {
            Some(d) => NewsItem::new_with_classification(
                id.to_string(),
                title.to_string(),
                url.to_string(),
                String::from("test-source"),
                String::from("test-author"),
                Utc::now(),
                d,
                0.9,
            ),
            None => NewsItem::new(
                id.to_string(),
                title.to_string(),
                url.to_string(),
                String::from("test-source"),
                String::from("test-author"),
                Utc::now(),
            ),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        let news = create_test_news("1", "Test News", "https://example.com/1", Some(Domain::AI));
        repo.save(&news).await.unwrap();
        
        let found = repo.find_by_id("1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test News");
    }

    #[tokio::test]
    async fn test_save_batch() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        let news_items = vec![
            create_test_news("1", "News 1", "https://example.com/1", Some(Domain::AI)),
            create_test_news("2", "News 2", "https://example.com/2", Some(Domain::Block)),
            create_test_news("3", "News 3", "https://example.com/3", Some(Domain::Social)),
        ];
        
        repo.save_batch(&news_items).await.unwrap();
        
        let count = repo.count().await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_save_duplicate_url() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        let news1 = create_test_news("1", "First", "https://example.com/dup", Some(Domain::AI));
        let news2 = create_test_news("2", "Second", "https://example.com/dup", Some(Domain::Block));
        
        repo.save(&news1).await.unwrap();
        repo.save(&news2).await.unwrap();
        
        let count = repo.count().await.unwrap();
        assert_eq!(count, 1);
        
        let found = repo.find_by_url("https://example.com/dup").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "First");
    }

    #[tokio::test]
    async fn test_find_recent() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        let base_time = Utc::now();
        let news_items = vec![
            NewsItem::new_with_classification(
                String::from("1"), String::from("Oldest"), String::from("https://example.com/1"),
                String::from("source"), String::from("author"), base_time - chrono::Duration::hours(2),
                Domain::AI, 0.8,
            ),
            NewsItem::new_with_classification(
                String::from("2"), String::from("Middle"), String::from("https://example.com/2"),
                String::from("source"), String::from("author"), base_time - chrono::Duration::hours(1),
                Domain::Block, 0.9,
            ),
            NewsItem::new_with_classification(
                String::from("3"), String::from("Newest"), String::from("https://example.com/3"),
                String::from("source"), String::from("author"), base_time,
                Domain::Social, 0.7,
            ),
        ];
        
        repo.save_batch(&news_items).await.unwrap();
        
        let recent = repo.find_recent(10).await.unwrap();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].title, "Newest");
        assert_eq!(recent[2].title, "Oldest");
    }

    #[tokio::test]
    async fn test_find_by_url() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        let news = create_test_news("1", "Test", "https://example.com/test", Some(Domain::AI));
        repo.save(&news).await.unwrap();
        
        let found = repo.find_by_url("https://example.com/test").await.unwrap();
        assert!(found.is_some());
        
        let not_found = repo.find_by_url("https://example.com/notfound").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_count() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        assert_eq!(repo.count().await.unwrap(), 0);
        
        repo.save(&create_test_news("1", "News 1", "https://example.com/1", Some(Domain::AI))).await.unwrap();
        repo.save(&create_test_news("2", "News 2", "https://example.com/2", Some(Domain::Block))).await.unwrap();
        
        assert_eq!(repo.count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_count_by_domain() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        let news_items = vec![
            create_test_news("1", "AI 1", "https://example.com/1", Some(Domain::AI)),
            create_test_news("2", "AI 2", "https://example.com/2", Some(Domain::AI)),
            create_test_news("3", "Block 1", "https://example.com/3", Some(Domain::Block)),
            create_test_news("4", "Social 1", "https://example.com/4", Some(Domain::Social)),
        ];
        
        repo.save_batch(&news_items).await.unwrap();
        
        let counts = repo.count_by_domain().await.unwrap();
        assert_eq!(counts.len(), 3);
        
        let ai_count = counts.iter().find(|(d, _)| *d == Domain::AI).map(|(_, c)| *c).unwrap_or(0);
        assert_eq!(ai_count, 2);
    }

    #[tokio::test]
    async fn test_find_by_domain() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        let news_items = vec![
            create_test_news("1", "AI News", "https://example.com/1", Some(Domain::AI)),
            create_test_news("2", "Block News", "https://example.com/2", Some(Domain::Block)),
            create_test_news("3", "Another AI", "https://example.com/3", Some(Domain::AI)),
        ];
        
        repo.save_batch(&news_items).await.unwrap();
        
        let ai_news = repo.find_by_domain(Domain::AI, 10).await.unwrap();
        assert_eq!(ai_news.len(), 2);
        assert!(ai_news[0].title.contains("AI"));
    }

    #[tokio::test]
    async fn test_domain_persistence() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repo = SqliteNewsRepository::new(pool);
        
        let original = create_test_news("1", "Test", "https://example.com/1", Some(Domain::AI));
        repo.save(&original).await.unwrap();
        
        let loaded = repo.find_by_id("1").await.unwrap().unwrap();
        assert_eq!(loaded.domain, Some(Domain::AI));
        assert_eq!(loaded.classification_confidence, Some(0.9));
    }
}
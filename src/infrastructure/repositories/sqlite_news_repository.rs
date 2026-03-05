use crate::domain::{Domain, NewsItem, NewsItemStatus, NewsRepository};
use async_trait::async_trait;
use sqlx::SqlitePool;

/// SQLite 实现的新闻仓库
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
        let status_str = format!("{:?}", news.status);

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO news_items (id, title, url, source, author, content, published_at, status, domain, classification_confidence, classification_reason)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#
        )
        .bind(&news.id)
        .bind(&news.title)
        .bind(&news.url)
        .bind(&news.source)
        .bind(&news.author)
        .bind(&news.content)
        .bind(news.published_at.to_rfc3339())
        .bind(&status_str)
        .bind(&domain_str)
        .bind(confidence)
        .bind(&news.classification_reason)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_batch(
        &self,
        news_items: &[NewsItem],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = self.pool.begin().await?;

        for news in news_items {
            let domain_str = news.domain.map(|d| d.to_string());
            let confidence = news.classification_confidence;
            let status_str = format!("{:?}", news.status);

            sqlx::query(
                r#"
                INSERT OR IGNORE INTO news_items (id, title, url, source, author, content, published_at, status, domain, classification_confidence, classification_reason)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#
            )
            .bind(&news.id)
            .bind(&news.title)
            .bind(&news.url)
            .bind(&news.source)
            .bind(&news.author)
            .bind(&news.content)
            .bind(news.published_at.to_rfc3339())
            .bind(&status_str)
            .bind(&domain_str)
            .bind(confidence)
            .bind(&news.classification_reason)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &str,
    ) -> Result<Option<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, String, String, Option<String>, Option<f32>, Option<String>)>(
            "SELECT id, title, url, source, author, content, published_at, status, domain, classification_confidence, classification_reason FROM news_items WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(row_to_news_item).transpose()?)
    }

    async fn find_by_domain(
        &self,
        domain: Domain,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        self.find_by_domains(&[domain], limit).await
    }

    async fn find_by_domains(
        &self,
        domains: &[Domain],
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        if domains.is_empty() {
            return Ok(Vec::new());
        }

        let domain_strs: Vec<String> = domains.iter().map(|d| d.to_string()).collect();
        let placeholders = domains
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let query_str = format!(
            r#"
            SELECT id, title, url, source, author, content, published_at, status, domain, classification_confidence, classification_reason
            FROM news_items
            WHERE domain IN ({})
            ORDER BY published_at DESC
            LIMIT ?{}
            "#,
            placeholders,
            domains.len() + 1
        );

        let mut query = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                Option<String>,
                String,
                String,
                Option<String>,
                Option<f32>,
                Option<String>,
            ),
        >(&query_str);

        for domain_str in &domain_strs {
            query = query.bind(domain_str);
        }
        query = query.bind(limit as i64);

        let rows = query.fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_news_item).collect()
    }

    async fn find_recent(
        &self,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, String, String, Option<String>, Option<f32>, Option<String>)>(
            "SELECT id, title, url, source, author, content, published_at, status, domain, classification_confidence, classification_reason FROM news_items ORDER BY published_at DESC LIMIT ?1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_news_item).collect()
    }

    async fn find_by_url(
        &self,
        url: &str,
    ) -> Result<Option<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, String, String, Option<String>, Option<f32>, Option<String>)>(
            "SELECT id, title, url, source, author, content, published_at, status, domain, classification_confidence, classification_reason FROM news_items WHERE url = ?1"
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(row_to_news_item).transpose()?)
    }

    async fn find_existing_urls(
        &self,
        urls: &[String],
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        if urls.is_empty() {
            return Ok(Vec::new());
        }

        // SQLite 参数限制为 999，分批处理
        const BATCH_SIZE: usize = 999;
        let mut existing_urls = Vec::new();

        for chunk in urls.chunks(BATCH_SIZE) {
            if chunk.is_empty() {
                continue;
            }

            // 构建占位符 (?1, ?2, ...)
            let placeholders = chunk
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");

            let query_str = format!(
                "SELECT url FROM news_items WHERE url IN ({})",
                placeholders
            );

            let mut query = sqlx::query_scalar(&query_str);
            for url in chunk {
                query = query.bind(url);
            }

            let chunk_results: Vec<String> = query.fetch_all(&self.pool).await?;
            existing_urls.extend(chunk_results);
        }

        Ok(existing_urls)
    }

    async fn count(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM news_items")
            .fetch_one(&self.pool)
            .await?;
        Ok(count as usize)
    }

    async fn count_by_domain(
        &self,
    ) -> Result<Vec<(Domain, usize)>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query_as::<_, (Option<String>, i64)>(
            r#"
            SELECT domain, COUNT(*) as count
            FROM news_items
            GROUP BY domain
            ORDER BY count DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for (domain_str, count) in rows {
            if let Some(domain) = domain_str.and_then(|s| parse_domain(&s)) {
                result.push((domain, count as usize));
            }
        }

        Ok(result)
    }
}

fn row_to_news_item(
    row: (
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<f32>,
        Option<String>,
    ),
) -> Result<NewsItem, Box<dyn std::error::Error + Send + Sync>> {
    let (
        id,
        title,
        url,
        source,
        author,
        content,
        published_at_str,
        status_str,
        domain_str,
        confidence,
        reason,
    ) = row;

    let published_at =
        chrono::DateTime::parse_from_rfc3339(&published_at_str)?.with_timezone(&chrono::Utc);
    let domain = domain_str.and_then(|s| parse_domain(&s));
    let status = parse_status(&status_str);

    Ok(NewsItem {
        id,
        title,
        url,
        source,
        author,
        content,
        published_at,
        status,
        domain,
        classification_confidence: confidence,
        classification_reason: reason,
    })
}

fn parse_domain(s: &str) -> Option<Domain> {
    match s {
        "AI" => Some(Domain::AI),
        "Block" => Some(Domain::Block),
        "Social" => Some(Domain::Social),
        _ => None,
    }
}

fn parse_status(s: &str) -> NewsItemStatus {
    match s {
        "Pending" => NewsItemStatus::Pending,
        "Classifying" => NewsItemStatus::Classifying,
        "NeedsReview" => NewsItemStatus::NeedsReview,
        "Completed" => NewsItemStatus::Completed,
        "Failed" => NewsItemStatus::Failed,
        _ => NewsItemStatus::Pending,
    }
}
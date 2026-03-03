mod application;
mod cli;
mod domain;
mod infrastructure;

use crate::application::orchestration;
use crate::domain::NewsClassificationService;
use crate::infrastructure::database::create_pool;
use crate::infrastructure::news_sources::HackerNewsSource;
use crate::infrastructure::repositories::SqliteNewsRepository;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // 初始化日志
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // 解析命令行参数
    let cli = cli::Cli::parse_args();

    info!("🚀 TrendArc - 热点新闻聚合器");

    let db_path = cli.database.clone();

    match cli.command {
        cli::Commands::Fetch {
            save,
            limit,
            domain,
        } => {
            let repository = if save {
                info!("📊 初始化数据库: {}", db_path);
                let pool = create_pool(&db_path).await?;
                let repo =
                    Arc::new(SqliteNewsRepository::new(pool)) as Arc<dyn domain::NewsRepository>;
                info!("✅ 数据库初始化完成");
                Some(repo)
            } else {
                None
            };

            let hn_fetcher = Arc::new(HackerNewsSource::new());
            let classifier = Arc::new(NewsClassificationService::new());

            info!("🌐 正在从 Hacker News 抓取数据...");
            let news_items =
                orchestration::fetch_from_source(hn_fetcher, classifier.clone(), limit, repository)
                    .await?;

            let filtered_news = if let Some(ref domains) = domain {
                info!(
                    "🔍 过滤领域: {}",
                    domains
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                news_items
                    .into_iter()
                    .filter(|n| n.domain.map_or(false, |d| domains.contains(&d)))
                    .collect()
            } else {
                news_items
            };

            orchestration::display_news(&filtered_news).await;
            info!("✅ 完成！共展示 {} 条新闻", filtered_news.len());
        }
        cli::Commands::List { limit, domain } => {
            info!("📊 初始化数据库: {}", db_path);
            let pool = create_pool(&db_path).await?;
            let repository =
                Arc::new(SqliteNewsRepository::new(pool)) as Arc<dyn domain::NewsRepository>;
            info!("✅ 数据库连接成功");

            let news_items =
                orchestration::load_from_database(&repository, domain.as_deref(), limit).await?;

            orchestration::display_news(&news_items).await;
            info!("═════════════════════════════════════════════");
            info!("✅ 完成！共展示 {} 条新闻", news_items.len());
        }
        cli::Commands::Stats => {
            let pool = create_pool(&db_path).await?;
            let repository =
                Arc::new(SqliteNewsRepository::new(pool)) as Arc<dyn domain::NewsRepository>;
            orchestration::show_stats(&repository).await?;
        }
    }

    Ok(())
}

// ========== 集成测试 ==========
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::application::use_cases::fetch_hot_news::{FetchHotNewsService, FetchHotNewsUseCase};
    use crate::domain::{NewsFetcher, NewsItem};
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::sync::Arc;

    // Mock NewsFetcher for testing
    struct MockNewsFetcher {
        data: Vec<NewsItem>,
    }

    impl MockNewsFetcher {
        fn with_data(data: Vec<NewsItem>) -> Self {
            Self { data }
        }
    }

    #[async_trait]
    impl NewsFetcher for MockNewsFetcher {
        async fn fetch(
            &self,
            _limit: usize,
        ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(self.data.clone())
        }

        fn source_name(&self) -> &str {
            "mock-source"
        }
    }

    // Helper function to create test news items
    fn create_test_news(id: &str, title: &str, url: &str, time: chrono::DateTime<Utc>) -> NewsItem {
        NewsItem::new(
            id.to_string(),
            title.to_string(),
            url.to_string(),
            String::from("test-source"),
            String::from("test-author"),
            time,
        )
    }

    #[tokio::test]
    async fn test_fetch_and_save_workflow() {
        // 测试抓取→保存→加载的完整流程
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repository: Arc<dyn domain::NewsRepository> = Arc::new(SqliteNewsRepository::new(pool));

        let base_time = Utc::now();
        let test_news = vec![
            create_test_news("1", "Latest News", "url1", base_time + Duration::hours(1)),
            create_test_news("2", "Duplicate Title", "url2", base_time),
            create_test_news("3", "Another News", "url3", base_time - Duration::hours(1)),
        ];

        // 模拟抓取并保存
        let mock_fetcher = MockNewsFetcher::with_data(test_news);
        let classifier = Arc::new(NewsClassificationService::new());
        let use_case = FetchHotNewsService::new(&mock_fetcher, classifier)
            .with_repository(Arc::clone(&repository));
        let _ = use_case.execute(10).await;

        // 验证数据库中有数据
        let count = repository.count().await.unwrap();
        assert_eq!(count, 3);

        // 验证可以加载数据
        let loaded = repository.find_recent(10).await.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].title, "Latest News"); // 最新在前
    }

    #[tokio::test]
    async fn test_duplicate_url_handling() {
        // 测试 URL 去重
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let repository: Arc<dyn domain::NewsRepository> = Arc::new(SqliteNewsRepository::new(pool));

        let base_time = Utc::now();
        let news_with_duplicates = vec![
            create_test_news("1", "First", "same-url", base_time),
            create_test_news("2", "Second", "same-url", base_time - Duration::minutes(10)),
        ];

        let mock_fetcher = MockNewsFetcher::with_data(news_with_duplicates);
        let classifier = Arc::new(NewsClassificationService::new());
        let use_case = FetchHotNewsService::new(&mock_fetcher, classifier)
            .with_repository(Arc::clone(&repository));
        let _ = use_case.execute(10).await;

        // 验证只有一条被保存
        let count = repository.count().await.unwrap();
        assert_eq!(count, 1);

        // 验证是第一条
        let loaded = repository.find_recent(10).await.unwrap();
        assert_eq!(loaded[0].title, "First");
    }
}

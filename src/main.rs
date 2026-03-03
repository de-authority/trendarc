mod application;
mod cli;
mod domain;
mod infrastructure;

use crate::application::orchestration;
use crate::domain::{Domain, NewsClassificationService};
use crate::infrastructure::database::create_pool;
use crate::infrastructure::news_sources::HackerNewsSource;
use crate::infrastructure::repositories::SqliteNewsRepository;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 解析命令行参数
    let cli = cli::Cli::parse_args();
    
    // 验证参数
    if let Err(e) = cli.validate() {
        eprintln!("❌ 参数错误: {}", e);
        std::process::exit(1);
    }

    // 显示警告信息
    for warning in cli.get_warnings() {
        eprintln!("⚠️  警告: {}", warning);
    }

    println!("🚀 TrendArc - 热点新闻聚合器\n");

    // 初始化数据库连接池（如果需要）
    let repository = if cli.save || cli.load || cli.stats {
        println!("📊 初始化数据库: {}", cli.database);
        let pool = create_pool(&cli.database).await?;
        let repo = Arc::new(SqliteNewsRepository::new(pool)) as Arc<dyn domain::NewsRepository>;
        println!("✅ 数据库初始化完成\n");
        Some(repo)
    } else {
        None
    };

    // 显示统计信息
    if cli.stats {
        return orchestration::show_stats(repository.as_ref().unwrap()).await;
    }

    // 解析并验证所有域名参数（提前失败）
    let target_domains: Option<Vec<Domain>> = cli.domain.as_ref().map(|domains| {
        domains.iter()
            .map(|d| Domain::from_str(d))
            .collect::<Result<Vec<_>, _>>()
    }).transpose()?;

    // 获取新闻数据
    let news_items = if cli.load {
        // 从数据库加载
        println!("📂 从数据库加载新闻...\n");
        
        // 数据库查询时的域名过滤：单域名在SQL层面过滤，多域名加载全部后在内存过滤
        let db_filter_domain = target_domains.as_ref().and_then(|domains| {
            if domains.len() == 1 {
                Some(domains[0])
            } else {
                None
            }
        });
        
        let news = orchestration::load_from_database(repository.as_ref().unwrap(), db_filter_domain, cli.limit).await?;
        println!("✅ 加载完成！共 {} 条新闻\n", news.len());
        news
    } else {
        // 从数据源抓取
        let hn_fetcher = Arc::new(HackerNewsSource::new());
        let classifier = Arc::new(NewsClassificationService::new());
        orchestration::fetch_from_source(hn_fetcher, classifier, cli.limit, repository).await?
    };

    // 过滤领域（如果指定）
    let filtered_news = if let Some(ref domains) = target_domains {
        let classifier = NewsClassificationService::new();
        
        if domains.len() == 1 {
            // 单个域名：使用统一的过滤函数
            println!("🔍 过滤领域: {}\n", domains[0]);
            filter_by_multiple_domains(&news_items, domains, &classifier)
        } else {
            // 多个域名：单次遍历，同时匹配所有域名
            println!("🔍 过滤领域: {}\n", cli.domain.as_ref().unwrap().join(", "));
            filter_by_multiple_domains(&news_items, domains, &classifier)
        }
    } else {
        news_items
    };

    // 展示新闻
    orchestration::display_news(&filtered_news).await;

    println!("═════════════════════════════════════════════");
    println!("✅ 完成！共展示 {} 条新闻", filtered_news.len());

    Ok(())
}

/// 高效地按多个领域过滤新闻
/// 单次遍历，同时匹配所有目标领域，自动去重并按时间排序
fn filter_by_multiple_domains(
    news_items: &[crate::domain::NewsItem],
    target_domains: &[Domain],
    classifier: &NewsClassificationService,
) -> Vec<crate::domain::NewsItem> {
    use std::collections::HashSet;
    
    let target_set: HashSet<Domain> = target_domains.iter().copied().collect();
    let mut seen_urls = HashSet::new();
    let mut filtered = Vec::new();

    for news in news_items {
        // 去重：同一 URL 只保留一次
        if !seen_urls.contains(&news.url) {
            seen_urls.insert(&news.url);
        } else {
            continue;
        }

        // 检查新闻是否属于任一目标领域
        let result = classifier.classify(news);
        if target_set.contains(&result.domain) {
            // 更新新闻的 domain 字段，确保 display_news 能正确分组
            let mut classified_news = news.clone();
            classified_news.domain = Some(result.domain);
            classified_news.classification_confidence = Some(result.confidence);
            filtered.push(classified_news);
        }
    }

    // 按发布时间降序排序
    filtered.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    filtered
}

// ========== 集成测试 ==========
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::domain::{NewsFetcher, NewsItem};
    use crate::application::use_cases::fetch_hot_news::{FetchHotNewsService, FetchHotNewsUseCase};
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
        async fn fetch(&self, _limit: usize) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
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
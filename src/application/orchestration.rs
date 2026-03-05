use crate::application::use_cases::fetch_hot_news::{FetchHotNewsService, FetchHotNewsUseCase};
use crate::domain::{Domain, NewsClassificationService, NewsFetcher};
use std::sync::Arc;
use tracing::info;

/// 应用层编排模块
///
/// **职责**：
/// - 编排业务流程
/// - 协调多个 UseCase 和组件
/// - 处理数据转换和展示
///
/// **为什么单独一个模块？**
/// - main.rs 应该只负责程序的启动和退出
/// - 业务流程编排属于 Application 层的职责
/// - 这样可以更好地测试和复用

/// 从数据源抓取新闻（支持分类控制）
pub async fn fetch_from_source_with_classification(
    fetcher: Arc<dyn NewsFetcher>,
    classifier: Arc<NewsClassificationService>,
    limit: usize,
    repository: Option<Arc<dyn crate::domain::NewsRepository>>,
    should_classify: bool,
) -> Result<Vec<crate::domain::NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
    let mut use_case = FetchHotNewsService::new(&*fetcher, classifier);

    // 如果需要保存，注入 Repository
    if let Some(ref repo) = repository {
        use_case = use_case.with_repository(Arc::clone(repo));
    }

    // 根据参数决定是否执行分类
    if should_classify {
        use_case.execute_with_classification(limit).await
    } else {
        use_case.execute_without_classification(limit).await
    }
}

/// 从数据源抓取新闻（向后兼容版本，默认执行分类）
pub async fn fetch_from_source(
    fetcher: Arc<dyn NewsFetcher>,
    classifier: Arc<NewsClassificationService>,
    limit: usize,
    repository: Option<Arc<dyn crate::domain::NewsRepository>>,
) -> Result<Vec<crate::domain::NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
    fetch_from_source_with_classification(fetcher, classifier, limit, repository, true).await
}

/// 从数据库加载新闻
pub async fn load_from_database(
    repository: &Arc<dyn crate::domain::NewsRepository>,
    domains: Option<&[Domain]>,
    limit: usize,
) -> Result<Vec<crate::domain::NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
    match domains {
        Some(domains) if !domains.is_empty() => {
            let news = repository.find_by_domains(domains, limit).await?;
            Ok(news)
        }
        _ => {
            let news = repository.find_recent(limit).await?;
            Ok(news)
        }
    }
}

/// 显示新闻
pub async fn display_news(news_items: &[crate::domain::NewsItem]) {
    let classifier = NewsClassificationService::new();
    let grouped = classifier.group_by_domain(news_items);

    info!("═════════════════════════════════════════════");

    // 展示 AI 领域新闻
    let ai_news = grouped.get(&Domain::AI).unwrap();
    if !ai_news.is_empty() {
        info!("🤖 AI 领域 ({} 条)", ai_news.len());
        info!("───────────────────────────────────────────");
        for (i, news) in ai_news.iter().enumerate() {
            print_news_item(i + 1, news);
        }
    }

    // 展示 Block 领域新闻
    let block_news = grouped.get(&Domain::Block).unwrap();
    if !block_news.is_empty() {
        info!("⛓️  Block 领域 ({} 条)", block_news.len());
        info!("───────────────────────────────────────────");
        for (i, news) in block_news.iter().enumerate() {
            print_news_item(i + 1, news);
        }
    }

    // 展示 Social 领域新闻
    let social_news = grouped.get(&Domain::Social).unwrap();
    if !social_news.is_empty() {
        info!("📱 Social 领域 ({} 条)", social_news.len());
        info!("───────────────────────────────────────────");
        for (i, news) in social_news.iter().enumerate() {
            print_news_item(i + 1, news);
        }
    }
}

/// 显示统计信息
pub async fn show_stats(
    repository: &Arc<dyn crate::domain::NewsRepository>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("📈 数据库统计信息");
    info!("───────────────────────────────────────────");

    let total = repository.count().await?;
    info!("📰 总新闻数: {}", total);

    let by_domain = repository.count_by_domain().await?;
    info!("按领域分布:");
    for (domain, count) in by_domain {
        info!("  {:?}: {} 条", domain, count);
    }

    Ok(())
}

/// 打印单条新闻
fn print_news_item(index: usize, news: &crate::domain::NewsItem) {
    info!("  【{}】{}", index, news.title);
    info!("      来源: {} | 作者: {}", news.source, news.author);
    info!("      链接: {}", news.url);
    if let Some(ref reason) = news.classification_reason {
        info!("      依据: {}", reason);
    }
}
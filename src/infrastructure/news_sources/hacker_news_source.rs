use crate::domain::NewsFetcher;
use crate::domain::NewsItem;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::task::JoinSet;

/// Raw HackerNews item from API
#[derive(Debug, Deserialize)]
struct RawHNItem {
    id: u32,
    title: String,
    url: Option<String>,
    by: String,
    score: i32,
    time: u64,
}

/// HackerNews source implementation
pub struct HackerNewsSource {
    client: Client,
    api_base: String,
}

impl HackerNewsSource {
    /// Create a new HackerNewsSource
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(2)) // 整体请求超时
                .connect_timeout(Duration::from_secs(2)) // 连接阶段超时
                .build()
                .unwrap(),
            api_base: "https://hacker-news.firebaseio.com/v0".to_string(),
        }
    }

    /// Convert raw HN item to domain NewsItem
    fn convert_to_domain(&self, raw: RawHNItem) -> NewsItem {
        NewsItem::new(
            raw.id.to_string(),                        // id
            raw.title,                                 // title
            raw.url.unwrap_or_else(|| "".to_string()), // url
            "hackernews".to_string(),                  // source
            raw.by,                                    // author
            Utc.timestamp_opt(raw.time as i64, 0)
                .single()
                .unwrap_or(Utc::now()), // published_at
        )
    }
}

#[async_trait]
impl NewsFetcher for HackerNewsSource {
    async fn fetch(
        &self,
        limit: usize,
    ) -> Result<Vec<NewsItem>, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Get top story IDs
        let ids_url = format!("{}/topstories.json", self.api_base);
        let ids: Vec<u32> = self.client.get(&ids_url).send().await?.json().await?;

        let mut tasks = JoinSet::new();

        for id in ids.into_iter().take(limit) {
            let item_url = format!("{}/item/{}.json", self.api_base, id);
            let client = self.client.clone();

            tasks.spawn(async move {
                match client
                    .get(&item_url)
                    .timeout(Duration::from_secs(1))
                    .send()
                    .await
                {
                    Ok(response) => {
                        if let Ok(raw_item) = response.json::<RawHNItem>().await {
                            if raw_item.url.is_some() {
                                Some(raw_item)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch item {}: {}", id, e);
                        None
                    }
                }
            });
        }

        // Step 3: Collect results
        let mut news_items = Vec::new();
        while let Some(result) = tasks.join_next().await {
            if let Ok(Some(raw_item)) = result {
                news_items.push(self.convert_to_domain(raw_item));
            }
        }

        Ok(news_items)
    }

    fn source_name(&self) -> &str {
        "hackernews"
    }
}

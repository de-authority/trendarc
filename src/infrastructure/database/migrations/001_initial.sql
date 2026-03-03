-- 新闻表
CREATE TABLE IF NOT EXISTS news_items (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT UNIQUE NOT NULL,
    source TEXT NOT NULL,
    author TEXT NOT NULL,
    published_at TEXT NOT NULL,
    domain TEXT,
    classification_confidence REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 索引：优化查询性能
CREATE INDEX IF NOT EXISTS idx_news_items_published_at ON news_items(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_news_items_url ON news_items(url);
CREATE INDEX IF NOT EXISTS idx_news_items_domain ON news_items(domain);
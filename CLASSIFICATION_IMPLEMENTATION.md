# 新闻分类和过滤功能实现总结

## 概述

本次实现了一个灵活的新闻分类和过滤系统，支持多级策略分类和多种过滤条件。

## 核心改进

### 1. 多级分类策略

采用策略模式实现多级分类，优先级从高到低：

1. **基于数据源的分类** (SourceBasedStrategy)
   - 置信度: 0.9 (高)
   - 根据数据源名称或 URL 域名进行分类
   - 例如: HackerNews → AI, Arxiv → AI
   - 优点: 快速、准确、不依赖内容

2. **基于关键词的分类** (KeywordBasedStrategy)
   - 置信度: 强关键词 0.9 (高), 弱关键词 0.3 (低)
   - 在标题和 URL 中匹配关键词
   - 例如: "GPT-4", "chatgpt" → AI (0.9)
   - 优点: 细粒度分类, 可覆盖更多场景

3. **默认分类**
   - 置信度: 0.0
   - 未分类项目归类为 Uncategorized

### 2. 过滤服务

新增 `NewsFilterService`，支持多种过滤条件：

- **按数据源过滤**: `--source hackernews`
- **按领域过滤**: `--domain ai,block,social`
- **按置信度过滤**: `--min-confidence 0.8`
- **组合过滤**: 可以同时使用多个条件

### 3. 分类配置

`ClassificationConfig` 提供可配置的分类规则：

```rust
// 默认配置
- source_tendency: HashMap<数据源, 领域>
- strong_keywords: HashMap<领域, Vec<强关键词>>
- weak_keywords: HashMap<领域, Vec<弱关键词>>
```

## 使用示例

### 命令行使用

```bash
# 1. 抓取所有新闻（不过滤）
cargo run

# 2. 抓取并过滤 AI 领域新闻
cargo run -- --domain ai

# 3. 抓取并过滤多个领域
cargo run -- --domain ai,block

# 4. 抓取特定数据源的新闻
cargo run -- --source hackernews

# 5. 抓取高置信度的新闻
cargo run -- --min-confidence 0.8

# 6. 组合过滤：AI 领域且置信度 > 0.7
cargo run -- --domain ai --min-confidence 0.7

# 7. 抓取并保存到数据库
cargo run -- --save

# 8. 从数据库加载 AI 领域新闻
cargo run -- --load --domain ai
```

### 代码使用

#### 1. 基本分类

```rust
use crate::domain::{NewsClassificationService, Domain};

let classifier = NewsClassificationService::new();
let result = classifier.classify(&news_item);

println!("分类结果: {:?}", result.domain);
println!("置信度: {}", result.confidence);
println!("使用的策略: {}", result.strategy_name);
```

#### 2. 批量分类

```rust
let classifier = NewsClassificationService::new();
let mut news_items = vec![...];
let results = classifier.classify_batch(&mut news_items);

// news_items 的 domain 和 classification_confidence 字段会被自动更新
```

#### 3. 按领域过滤

```rust
let classifier = NewsClassificationService::new();
let ai_news = classifier.filter_by_domain(&all_news, Domain::AI);
```

#### 4. 高级过滤

```rust
use crate::domain::{FilterConfig, NewsFilterService, Domain};

let config = FilterConfig::new()
    .with_sources(vec!["hackernews".to_string()])
    .with_domains(vec![Domain::AI])
    .with_min_confidence(0.8);

let filtered = NewsFilterService::filter(&all_news, &config);
```

## 架构设计

### 分层决策

1. **Domain 层** (领域层)
   - `ClassificationStrategy`: 分类策略 trait
   - `SourceBasedStrategy`: 基于数据源的分类
   - `KeywordBasedStrategy`: 基于关键词的分类
   - `NewsClassificationService`: 分类服务（编排多个策略）
   - `NewsFilterService`: 过滤服务
   - `FilterConfig`: 过滤配置
   - `ClassificationConfig`: 分类配置

2. **Application 层** (应用层)
   - `FetchHotNewsService`: 集成分类功能
   - `AggregateNewsService`: 支持带过滤的聚合

3. **Infrastructure 层** (基础设施层)
   - 数据库支持保存分类结果

### 设计原则

1. **开闭原则**: 可以轻松添加新的分类策略
2. **单一职责**: 每个服务只负责一个功能
3. **依赖注入**: 通过构造函数注入依赖，便于测试
4. **策略模式**: 算法可替换，运行时选择

## 扩展性

### 添加新的分类策略

```rust
use crate::domain::{ClassificationStrategy, NewsItem, ClassificationResult};

pub struct AIBasedStrategy;

impl ClassificationStrategy for AIBasedStrategy {
    fn classify(&self, news: &NewsItem) -> Option<ClassificationResult> {
        // 调用 AI API 进行分类
        Some(ClassificationResult::new(Domain::AI, 0.95, "ai-based".to_string()))
    }
    
    fn name(&self) -> &str {
        "ai-based"
    }
}
```

### 自定义关键词

```rust
use crate::domain::strategies::KeywordBasedStrategy;

let mut strategy = KeywordBasedStrategy::new();
strategy.add_strong_keyword(Domain::AI, "llm".to_string());
strategy.add_weak_keyword(Domain::Block, "crypto".to_string());
```

### 自定义数据源映射

```rust
use crate::domain::strategies::SourceBasedStrategy;

let mut strategy = SourceBasedStrategy::new();
strategy.add_mapping("my-custom-source".to_string(), Domain::Social);
```

## 性能考虑

1. **分类速度**: 基于数据源的分类 O(1)，基于关键词的分类 O(n*m)
   - n: 新闻数量
   - m: 关键词数量

2. **并发抓取**: 支持多个数据源并发抓取

3. **去重效率**: 使用 HashMap，O(1) 查找

## 测试

所有新组件都包含完整的单元测试：

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test domain::strategies
cargo test domain::services::news_classification_service
cargo test domain::services::news_filter_service
```

## 未来改进方向

1. **AI 分类集成**
   - 集成 OpenAI、Claude 等 AI API
   - 实现本地轻量级模型（如使用 TensorFlow）
   - 缓存 AI 分类结果以提高性能

2. **机器学习**
   - 基于历史数据训练分类器
   - 自动更新关键词库
   - 学习数据源-领域关联

3. **实时更新**
   - 支持配置文件热重载
   - 动态更新关键词库
   - 实时调整策略优先级

4. **性能优化**
   - 实现分类结果缓存
   - 并行分类（多线程）
   - 异步 AI API 调用

5. **增强过滤**
   - 支持正则表达式过滤
   - 支持时间范围过滤
   - 支持组合逻辑（AND/OR）

## 总结

本次实现提供了一个灵活、可扩展的新闻分类和过滤系统：

- ✅ 支持多级分类策略
- ✅ 支持多种过滤条件
- ✅ 命令行界面友好
- ✅ 代码结构清晰，易于维护
- ✅ 完整的单元测试
- ✅ 为未来 AI 分类集成预留接口

系统既支持快速、准确的数据源分类，也支持细粒度的关键词分类，可以根据不同场景灵活配置。
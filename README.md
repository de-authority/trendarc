# TrendArc - 智能热点新闻聚合器

TrendArc 是一个用 Rust 编写的智能热点新闻聚合器，专注于 AI、区块链和社交媒体三大领域的新闻分类与聚合。项目采用领域驱动设计（DDD）架构，结合规则引擎和 AI 推理，实现高效、准确的新闻分类。

## ✨ 核心特性

- **多源新闻聚合**：支持从 HackerNews 等平台抓取热点新闻
- **智能分类系统**：采用五阶漏斗分类策略，结合规则匹配与 AI 推理
- **领域聚焦**：专注于 AI、区块链、社交媒体三大技术领域
- **去重与排序**：自动去重相似新闻，按时间排序展示
- **Discord 集成**：支持将分类结果发送到 Discord 频道
- **SQLite 存储**：持久化存储新闻数据，支持历史查询
- **命令行界面**：提供直观的 CLI 工具，便于使用和集成

## 🏗️ 系统架构

### 分层架构设计

```
TrendArc/
├── src/
│   ├── domain/           # 领域层 - 核心业务逻辑
│   │   ├── entities/     # 领域实体（NewsItem, Domain）
│   │   ├── services/     # 领域服务（分类、去重、排序）
│   │   ├── strategies/   # 分类策略（关键词、来源）
│   │   ├── fetchers/     # 新闻抓取抽象
│   │   └── repositories/ # 仓储接口
│   ├── application/      # 应用层 - 用例编排
│   │   ├── use_cases/    # 具体用例实现
│   │   └── orchestration # 业务流程编排
│   └── infrastructure/   # 基础设施层
│       ├── news_sources/ # 具体新闻源实现
│       ├── inference/    # AI 推理服务（OpenAI）
│       ├── discord/      # Discord 客户端
│       ├── database/     # 数据库连接
│       └── repositories/ # 仓储实现（SQLite）
├── config/               # 配置文件
└── migrations/          # 数据库迁移
```

### 五阶漏斗分类策略

1. **静态规则匹配**：基于标题和 URL 的关键词快速匹配
2. **正文内容抓取**：抓取新闻全文内容进行增强分析
3. **全文关键词扫描**：基于完整内容进行深度关键词匹配
4. **AI 仲裁推理**：使用 OpenAI 进行语义理解和分类
5. **兜底处理**：弱匹配保留或标记为不可分类

## 🚀 快速开始

### 环境要求

- Rust 1.70+ 和 Cargo
- SQLite3（用于数据存储）
- OpenAI API 密钥（可选，用于 AI 分类）

### 安装与运行

```bash
# 克隆项目
git clone <repository-url>
cd trendarc

# 构建项目
cargo build --release

# 运行帮助查看可用命令
cargo run -- --help
```

### 基本使用示例

```bash
# 从 HackerNews 抓取新闻并显示
cargo run -- fetch --source hackernews --limit 10

# 抓取并保存到数据库
cargo run -- fetch --source hackernews --save --limit 20

# 抓取并发送到 Discord
cargo run -- fetch --source hackernews --discord --discord-webhook <webhook-url>

# 仅抓取特定领域的新闻（AI、Block、Social）
cargo run -- fetch --source hackernews --domain AI --domain Block

# 查看数据库中的新闻
cargo run -- list --limit 10

# 查看统计信息
cargo run -- stats
```

## 📊 命令行参数

### 主要命令

```
fetch   从指定数据源抓取新闻
list    从数据库列出已保存的新闻
stats   显示数据库统计信息
```

### Fetch 命令选项

```
--source <SOURCE>        数据源（目前支持：hackernews）
--save                   保存到数据库
--discord                发送到 Discord
--discord-webhook <URL>  Discord Webhook URL
--limit <NUMBER>         抓取数量限制（默认：20）
--domain <DOMAIN>        过滤特定领域（可多次指定：AI、Block、Social）
```

## 🔧 配置说明

### 分类配置文件

项目使用 `config/classification.json` 配置分类关键词，支持强关键词和弱关键词区分：

```json
{
  "strong_keywords": {
    "AI": ["gpt-4", "chatgpt", "openai", "llm", "generative ai", "deep learning"],
    "Block": ["bitcoin", "ethereum", "web3", "defi", "nft", "blockchain"],
    "Social": ["twitter", "tiktok", "instagram", "meta", "youtube", "social platform"]
  },
  "weak_keywords": {
    "AI": ["machine learning", "ai", "model", "inference", "training"],
    "Block": ["crypto", "cryptocurrency", "token", "on-chain", "transaction"],
    "Social": ["social media", "influencer", "viral", "trending", "content algorithm"]
  },
  "source_tendency": {}
}
```

**关键词类型说明**：
- **强关键词**：高置信度匹配（置信度 ≥ 0.9），直接确定分类
- **弱关键词**：低置信度匹配（置信度 ≈ 0.3），需要结合其他策略确认

### 环境变量

```bash
# OpenAI API 配置（用于 AI 分类）
export OPENAI_API_KEY=your-api-key
export OPENAI_MODEL=gpt-3.5-turbo

# Discord Webhook（默认配置）
export DISCORD_WEBHOOK_URL=your-webhook-url

# 数据库路径（默认：news.db）
export DATABASE_PATH=path/to/database.db
```

## 🧪 测试

项目包含完整的单元测试和集成测试：

```bash
# 运行所有测试
cargo test

# 运行特定测试模块
cargo test --test integration_tests

# 生成测试覆盖率报告
cargo tarpaulin --out Html
```

## 📁 项目结构详解

### 领域层（Domain）

- **entities/**: 核心领域实体定义
  - `NewsItem`: 新闻项实体，包含标题、URL、来源、发布时间等
  - `Domain`: 新闻领域枚举（AI、Block、Social）
- **services/**: 领域服务
  - `NewsClassificationService`: 新闻分类服务（五阶漏斗策略）
  - `NewsDeduplicationService`: 新闻去重服务
  - `NewsSortingService`: 新闻排序服务
- **strategies/**: 分类策略
  - `KeywordBasedStrategy`: 基于关键词的分类策略
  - `ClassificationStrategy`: 分类策略接口

### 应用层（Application）

- **use_cases/**: 具体业务用例
  - `FetchHotNewsUseCase`: 抓取热点新闻用例
- **orchestration.rs**: 业务流程编排器

### 基础设施层（Infrastructure）

- **news_sources/**: 新闻源实现
  - `HackerNewsSource`: HackerNews API 客户端
- **inference/**: AI 推理服务
  - `OpenAIInferenceService`: OpenAI API 集成
- **discord/**: Discord 客户端
- **repositories/**: 数据仓储
  - `SqliteNewsRepository`: SQLite 实现

## 🔄 工作流程

1. **新闻抓取**：从配置的数据源获取原始新闻数据
2. **预处理**：转换为领域实体，提取关键信息
3. **分类处理**：通过五阶漏斗策略进行分类
4. **去重排序**：去除重复新闻，按时间排序
5. **持久化**：保存到 SQLite 数据库
6. **输出**：控制台显示或发送到 Discord

## 🛠️ 开发指南

### 添加新的新闻源

1. 在 `src/domain/fetchers/` 中实现 `NewsFetcher` trait
2. 在 `src/infrastructure/news_sources/` 中创建具体实现
3. 在 `NewsSourceFactory` 中注册新源
4. 更新 CLI 参数解析支持新源

### 扩展分类领域

1. 在 `src/domain/entities/mod.rs` 中扩展 `Domain` 枚举
2. 更新 `config/classification.json` 添加新领域关键词
3. 在分类策略中实现新领域的识别逻辑

### 性能优化

- 使用异步并发处理提高抓取效率
- 实现请求缓存减少重复抓取
- 优化数据库查询索引

## 📈 性能指标

- 单次抓取处理时间：< 5秒（20条新闻）
- 分类准确率：> 85%（结合 AI 推理）
- 内存使用：< 50MB
- 数据库查询响应：< 100ms

## 🤝 贡献指南

1. Fork 项目仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 🙏 致谢

- 感谢 HackerNews 提供开放的 API
- 感谢 OpenAI 提供的 AI 推理能力
- 感谢 Rust 社区提供的优秀库和工具

## 📞 支持与反馈

如有问题或建议，请通过以下方式联系：

- 提交 GitHub Issue
- 查看项目文档
- 参与社区讨论

---

**TrendArc** - 让热点新闻分类更智能、更高效！
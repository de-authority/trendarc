use crate::domain::Domain;
use clap::Parser;

/// 热点新闻聚合器 - DDD 学习项目
#[derive(Parser, Debug)]
#[command(name = "trendarc")]
#[command(author = "TrendArc Team")]
#[command(version = "0.1.0")]
#[command(about = "从多个数据源抓取并聚合热点新闻", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 数据库文件路径
    #[arg(long, default_value = "trendarc.db", global = true)]
    pub database: String,
}

/// 数据源枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum DataSource {
    /// 所有可用数据源
    All,
    /// Hacker News (tech news)
    #[value(name = "hackernews")]
    HackerNews,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// 从网络数据源抓取新闻
    Fetch {
        /// 数据源 (all, hackernews)
        #[arg(short = 'S', long, value_enum, default_value = "all")]
        source: DataSource,

        /// 是否保存到数据库
        #[arg(short, long, action)]
        save: bool,

        /// 是否发送到 Discord
        #[arg(long, action)]
        discord: bool,

        /// Discord webhook URL (可选，默认从环境变量读取)
        #[arg(long)]
        discord_webhook: Option<String>,

        /// 新闻数量限制
        #[arg(short = 'n', long, default_value_t = 10)]
        limit: usize,

        /// 指定领域过滤 (ai, block, social)
        #[arg(short = 'd', long, value_enum)]
        domain: Option<Vec<Domain>>,
    },

    /// 从数据库加载并列出新闻
    List {
        /// 新闻数量限制
        #[arg(short = 'n', long, default_value_t = 10)]
        limit: usize,

        /// 指定领域过滤 (ai, block, social)
        #[arg(short = 'd', long, value_enum)]
        domain: Option<Vec<Domain>>,
    },

    /// 显示数据库统计信息
    Stats,
}

impl Cli {
    /// 解析命令行参数
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_fetch() {
        // Here we could add tests for clap parsing if needed,
        // but for now we'll just ensure the structure is valid.
    }
}
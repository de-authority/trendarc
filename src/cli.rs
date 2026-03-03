use clap::Parser;
use crate::domain::Domain;

/// 热点新闻聚合器 - DDD 学习项目
#[derive(Parser, Debug)]
#[command(name = "trendarc")]
#[command(author = "TrendArc Team")]
#[command(version = "0.1.0")]
#[command(about = "从多个数据源抓取并聚合热点新闻", long_about = None)]
pub struct Cli {
    /// 是否保存到数据库
    #[arg(short, long, action)]
    pub save: bool,

    /// 是否从数据库加载（而不是抓取）
    #[arg(short = 'L', long, action)]
    pub load: bool,

    /// 新闻数量限制
    #[arg(short = 'n', long, default_value_t = 10)]
    pub limit: usize,

    /// 数据库文件路径
    #[arg(long, default_value = "trendarc.db")]
    pub database: String,

    /// 指定领域过滤（ai, block, social），可多个，用逗号分隔
    #[arg(short = 'd', long, value_delimiter = ',')]
    pub domain: Option<Vec<String>>,

    /// 指定数据源过滤（如 hackernews），可多个，用逗号分隔
    #[arg(long, value_delimiter = ',')]
    pub source: Option<Vec<String>>,

    /// 最小置信度阈值（0.0 - 1.0）
    #[arg(long, value_name = "FLOAT")]
    pub min_confidence: Option<f32>,

    /// 显示统计信息
    #[arg(long, action)]
    pub stats: bool,
}

impl Cli {
    /// 解析命令行参数
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// 验证参数
    pub fn validate(&self) -> Result<(), String> {
        // 不能同时使用 --save 和 --load
        if self.save && self.load {
            return Err("不能同时使用 --save 和 --load".to_string());
        }

        // --stats 不能与 --save 或 --load 一起使用
        if self.stats && (self.save || self.load) {
            return Err("--stats 不能与 --save 或 --load 一起使用".to_string());
        }

        // 验证 domain 参数 - 使用统一的 Domain::is_valid
        if let Some(ref domains) = self.domain {
            for domain in domains {
                if !Domain::is_valid(domain) {
                    return Err(format!(
                        "无效的领域: {}. 有效值: {}",
                        domain,
                        Domain::valid_domains_str()
                    ));
                }
            }
        }

        // 验证 min_confidence 参数
        if let Some(conf) = self.min_confidence {
            if conf < 0.0 || conf > 1.0 {
                return Err(format!(
                    "无效的置信度: {}. 必须在 0.0 到 1.0 之间",
                    conf
                ));
            }
        }

        Ok(())
    }

    /// 获取警告信息（用于未使用或不推荐的参数组合）
    pub fn get_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // 未实现的参数警告
        if self.source.is_some() {
            warnings.push("--source 参数当前未被使用，暂不支持多数据源".to_string());
        }
        if self.min_confidence.is_some() {
            warnings.push("--min-confidence 参数当前未被使用，暂不支持置信度过滤".to_string());
        }

        // stats 与 domain 组合警告
        if self.stats && self.domain.is_some() {
            warnings.push("--stats 模式下 --domain 参数将被忽略".to_string());
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_success() {
        let cli = Cli {
            save: false,
            load: false,
            limit: 10,
            database: "test.db".to_string(),
            domain: None,
            source: None,
            min_confidence: None,
            stats: false,
        };
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn test_validate_save_and_load_conflict() {
        let cli = Cli {
            save: true,
            load: true,
            limit: 10,
            database: "test.db".to_string(),
            domain: None,
            source: None,
            min_confidence: None,
            stats: false,
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_domain() {
        let cli = Cli {
            save: false,
            load: false,
            limit: 10,
            database: "test.db".to_string(),
            domain: Some(vec!["invalid".to_string()]),
            source: None,
            min_confidence: None,
            stats: false,
        };
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_validate_valid_domain() {
        let cli = Cli {
            save: false,
            load: false,
            limit: 10,
            database: "test.db".to_string(),
            domain: Some(vec!["ai".to_string()]),
            source: None,
            min_confidence: None,
            stats: false,
        };
        assert!(cli.validate().is_ok());
    }
}
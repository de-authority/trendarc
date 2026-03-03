pub mod news_item;

pub use news_item::NewsItem;

/// News domain/category
/// 
/// Represents the three main domains this project focuses on:
/// - AI: Artificial Intelligence and machine learning news
/// - Block: Blockchain and cryptocurrency news
/// - Social: Social media and social platform news
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    /// AI-related news (ChatGPT, GPT, LLM, ML, etc.)
    AI,
    /// Blockchain-related news (Bitcoin, Ethereum, Web3, Crypto, etc.)
    Block,
    /// Social media news (Twitter, Facebook, Instagram, etc.)
    Social,
    /// Uncategorized or unknown domain
    Uncategorized,
}

impl Domain {
    /// All valid domain names for parsing
    pub const VALID_DOMAINS: [&'static str; 3] = ["ai", "block", "social"];

    /// Get display name for the domain
    pub fn display_name(&self) -> &str {
        match self {
            Domain::AI => "AI",
            Domain::Block => "Block",
            Domain::Social => "Social",
            Domain::Uncategorized => "Uncategorized",
        }
    }

    /// Parse a string to Domain
    /// Returns Ok(Domain) for valid domain strings (case-insensitive)
    /// Returns Err with error message for invalid strings
    pub fn from_str(s: &str) -> Result<Domain, String> {
        match s.to_lowercase().as_str() {
            "ai" => Ok(Domain::AI),
            "block" => Ok(Domain::Block),
            "social" => Ok(Domain::Social),
            _ => Err(format!("无效的领域: {}. 有效值: {}", s, Self::valid_domains_str())),
        }
    }

    /// Get comma-separated list of valid domains
    pub fn valid_domains_str() -> String {
        Self::VALID_DOMAINS.join(", ")
    }

    /// Check if a string is a valid domain name
    pub fn is_valid(s: &str) -> bool {
        Self::VALID_DOMAINS.contains(&s.to_lowercase().as_str())
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_display_names() {
        assert_eq!(Domain::AI.display_name(), "AI");
        assert_eq!(Domain::Block.display_name(), "Block");
        assert_eq!(Domain::Social.display_name(), "Social");
        assert_eq!(Domain::Uncategorized.display_name(), "Uncategorized");
    }

    #[test]
    fn test_domain_display_trait() {
        assert_eq!(format!("{}", Domain::AI), "AI");
        assert_eq!(format!("{}", Domain::Block), "Block");
        assert_eq!(format!("{}", Domain::Social), "Social");
    }

    #[test]
    fn test_domain_equality() {
        assert_eq!(Domain::AI, Domain::AI);
        assert_ne!(Domain::AI, Domain::Block);
    }

    #[test]
    fn test_domain_copy() {
        let domain = Domain::AI;
        let copied = domain;
        assert_eq!(domain, copied);
    }
}

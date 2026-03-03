//! # Classification Configuration
//!
//! Contains configuration data for classification strategies including
/// keyword mappings and source tendencies.

use crate::domain::Domain;
use std::collections::HashMap;

/// Configuration for news classification
#[derive(Debug, Clone)]
pub struct ClassificationConfig {
    /// Strong keywords (high confidence)
    pub strong_keywords: HashMap<Domain, Vec<String>>,
    
    /// Weak keywords (low confidence)
    pub weak_keywords: HashMap<Domain, Vec<String>>,
    
    /// Source tendency mapping (default domain for each source)
    pub source_tendency: HashMap<String, Domain>,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        let mut strong_keywords: HashMap<Domain, Vec<String>> = HashMap::new();
        let mut weak_keywords: HashMap<Domain, Vec<String>> = HashMap::new();
        let mut source_tendency: HashMap<String, Domain> = HashMap::new();

        // ===== AI Keywords =====
        
        // Strong AI keywords (high confidence)
        strong_keywords.insert(Domain::AI, vec![
            "gpt-4".to_string(),
            "gpt4".to_string(),
            "chatgpt".to_string(),
            "openai".to_string(),
            "claude".to_string(),
            "gemini".to_string(),
            "llm".to_string(),
            "large language model".to_string(),
            "hugging face".to_string(),
            "stable diffusion".to_string(),
            "midjourney".to_string(),
            "dall-e".to_string(),
            "dalle".to_string(),
            "transformer".to_string(),
            "attention mechanism".to_string(),
            "bert".to_string(),
            "diffusion model".to_string(),
            "generative ai".to_string(),
            "neural network".to_string(),
            "deep learning".to_string(),
        ]);

        // Weak AI keywords (low confidence)
        weak_keywords.insert(Domain::AI, vec![
            "ai".to_string(),
            "artificial intelligence".to_string(),
            "machine learning".to_string(),
            "ml".to_string(),
            "nlp".to_string(),
            "natural language processing".to_string(),
            "computer vision".to_string(),
            "model".to_string(),
            "training".to_string(),
            "inference".to_string(),
            "prompt".to_string(),
            "autonomous".to_string(),
            "robotics".to_string(),
            "tensorflow".to_string(),
            "pytorch".to_string(),
            "copilot".to_string(),
            "github copilot".to_string(),
        ]);

        // ===== Blockchain Keywords =====
        
        // Strong blockchain keywords (high confidence)
        strong_keywords.insert(Domain::Block, vec![
            "bitcoin".to_string(),
            "btc".to_string(),
            "ethereum".to_string(),
            "eth".to_string(),
            "solana".to_string(),
            "cardano".to_string(),
            "polkadot".to_string(),
            "web3".to_string(),
            "defi".to_string(),
            "nft".to_string(),
            "dao".to_string(),
            "smart contract".to_string(),
            "layer 2".to_string(),
            "l2".to_string(),
            "bridge".to_string(),
        ]);

        // Weak blockchain keywords (low confidence)
        weak_keywords.insert(Domain::Block, vec![
            "crypto".to_string(),
            "cryptocurrency".to_string(),
            "blockchain".to_string(),
            "wallet".to_string(),
            "metamask".to_string(),
            "miner".to_string(),
            "mining".to_string(),
            "hash".to_string(),
            "token".to_string(),
            "coinbase".to_string(),
            "binance".to_string(),
            "consensus".to_string(),
            "fork".to_string(),
            "gas".to_string(),
        ]);

        // ===== Social Keywords =====
        
        // Strong social keywords (high confidence)
        strong_keywords.insert(Domain::Social, vec![
            "twitter".to_string(),
            "x.com".to_string(),
            "tiktok".to_string(),
            "instagram".to_string(),
            "facebook".to_string(),
            "meta".to_string(),
            "linkedin".to_string(),
            "youtube".to_string(),
            "discord".to_string(),
            "telegram".to_string(),
        ]);

        // Weak social keywords (low confidence)
        weak_keywords.insert(Domain::Social, vec![
            "social media".to_string(),
            "influencer".to_string(),
            "viral".to_string(),
            "trending".to_string(),
            "hashtag".to_string(),
            "thread".to_string(),
            "follow".to_string(),
            "like".to_string(),
            "share".to_string(),
            "subscribe".to_string(),
            "stream".to_string(),
            "podcast".to_string(),
            "community".to_string(),
            "whatsapp".to_string(),
            "snapchat".to_string(),
        ]);

        // ===== Source Tendency =====
        
        // AI-focused sources
        source_tendency.insert("arxiv.org".to_string(), Domain::AI);
        source_tendency.insert("paperswithcode.com".to_string(), Domain::AI);
        source_tendency.insert("huggingface.co".to_string(), Domain::AI);
        source_tendency.insert("huggingface".to_string(), Domain::AI);
        source_tendency.insert("openai.com".to_string(), Domain::AI);
        source_tendency.insert("github.com".to_string(), Domain::AI);
        
        // Blockchain-focused sources
        source_tendency.insert("coindesk.com".to_string(), Domain::Block);
        source_tendency.insert("coinbase.com".to_string(), Domain::Block);
        source_tendency.insert("binance.com".to_string(), Domain::Block);
        source_tendency.insert("ethereum.org".to_string(), Domain::Block);
        
        // Social-focused sources
        source_tendency.insert("twitter.com".to_string(), Domain::Social);
        source_tendency.insert("x.com".to_string(), Domain::Social);
        source_tendency.insert("tiktok.com".to_string(), Domain::Social);
        source_tendency.insert("instagram.com".to_string(), Domain::Social);
        source_tendency.insert("linkedin.com".to_string(), Domain::Social);
        source_tendency.insert("youtube.com".to_string(), Domain::Social);
        
        // Tech-focused sources (tend toward AI)
        source_tendency.insert("hackernews".to_string(), Domain::AI);
        source_tendency.insert("ycombinator.com".to_string(), Domain::AI);
        source_tendency.insert("techcrunch.com".to_string(), Domain::AI);
        source_tendency.insert("theverge.com".to_string(), Domain::AI);
        source_tendency.insert("arstechnica.com".to_string(), Domain::AI);

        Self {
            strong_keywords,
            weak_keywords,
            source_tendency,
        }
    }
}

impl ClassificationConfig {
    /// Create a new empty configuration
    pub fn empty() -> Self {
        Self {
            strong_keywords: HashMap::new(),
            weak_keywords: HashMap::new(),
            source_tendency: HashMap::new(),
        }
    }
    
    /// Add a strong keyword for a domain
    pub fn add_strong_keyword(&mut self, domain: Domain, keyword: String) {
        self.strong_keywords
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(keyword);
    }
    
    /// Add a weak keyword for a domain
    pub fn add_weak_keyword(&mut self, domain: Domain, keyword: String) {
        self.weak_keywords
            .entry(domain)
            .or_insert_with(Vec::new)
            .push(keyword);
    }
    
    /// Set source tendency for a source
    pub fn set_source_tendency(&mut self, source: String, domain: Domain) {
        self.source_tendency.insert(source, domain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_ai_keywords() {
        let config = ClassificationConfig::default();
        
        let ai_strong = config.strong_keywords.get(&Domain::AI);
        assert!(ai_strong.is_some());
        assert!(ai_strong.unwrap().contains(&"gpt-4".to_string()));
        
        let ai_weak = config.weak_keywords.get(&Domain::AI);
        assert!(ai_weak.is_some());
        assert!(ai_weak.unwrap().contains(&"ai".to_string()));
    }

    #[test]
    fn test_default_config_has_blockchain_keywords() {
        let config = ClassificationConfig::default();
        
        let block_strong = config.strong_keywords.get(&Domain::Block);
        assert!(block_strong.is_some());
        assert!(block_strong.unwrap().contains(&"bitcoin".to_string()));
        
        let block_weak = config.weak_keywords.get(&Domain::Block);
        assert!(block_weak.is_some());
        assert!(block_weak.unwrap().contains(&"crypto".to_string()));
    }

    #[test]
    fn test_default_config_has_social_keywords() {
        let config = ClassificationConfig::default();
        
        let social_strong = config.strong_keywords.get(&Domain::Social);
        assert!(social_strong.is_some());
        assert!(social_strong.unwrap().contains(&"twitter".to_string()));
        
        let social_weak = config.weak_keywords.get(&Domain::Social);
        assert!(social_weak.is_some());
        assert!(social_weak.unwrap().contains(&"social media".to_string()));
    }

    #[test]
    fn test_default_config_has_source_tendency() {
        let config = ClassificationConfig::default();
        
        assert_eq!(config.source_tendency.get("hackernews"), Some(&Domain::AI));
        assert_eq!(config.source_tendency.get("arxiv.org"), Some(&Domain::AI));
        assert_eq!(config.source_tendency.get("twitter.com"), Some(&Domain::Social));
        assert_eq!(config.source_tendency.get("coindesk.com"), Some(&Domain::Block));
    }

    #[test]
    fn test_empty_config() {
        let config = ClassificationConfig::empty();
        
        assert!(config.strong_keywords.is_empty());
        assert!(config.weak_keywords.is_empty());
        assert!(config.source_tendency.is_empty());
    }

    #[test]
    fn test_add_strong_keyword() {
        let mut config = ClassificationConfig::empty();
        config.add_strong_keyword(Domain::AI, "custom-keyword".to_string());
        
        let ai_keywords = config.strong_keywords.get(&Domain::AI).unwrap();
        assert_eq!(ai_keywords.len(), 1);
        assert!(ai_keywords.contains(&"custom-keyword".to_string()));
    }

    #[test]
    fn test_add_weak_keyword() {
        let mut config = ClassificationConfig::empty();
        config.add_weak_keyword(Domain::Block, "custom-crypto".to_string());
        
        let block_keywords = config.weak_keywords.get(&Domain::Block).unwrap();
        assert_eq!(block_keywords.len(), 1);
        assert!(block_keywords.contains(&"custom-crypto".to_string()));
    }

    #[test]
    fn test_set_source_tendency() {
        let mut config = ClassificationConfig::empty();
        config.set_source_tendency("custom-source.com".to_string(), Domain::AI);
        
        assert_eq!(
            config.source_tendency.get("custom-source.com"),
            Some(&Domain::AI)
        );
    }
}
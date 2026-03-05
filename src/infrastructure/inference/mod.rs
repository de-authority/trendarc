pub mod openai_inference_service;

use crate::domain::services::NewsInferenceService;
use std::sync::Arc;
use tracing::warn;

/// 创建AI推理服务实例
/// 
/// 如果配置了OPENAI_API_KEY环境变量，则创建OpenAI服务
/// 否则返回None，表示禁用AI分类
pub fn create_inference_service() -> Option<Arc<dyn NewsInferenceService>> {
    match openai_inference_service::OpenAIInferenceService::new() {
        Ok(service) => {
            Some(Arc::new(service))
        }
        Err(e) => {
            warn!("⚠️ 无法创建AI推理服务: {}. AI分类将禁用", e);
            None
        }
    }
}

pub use openai_inference_service::OpenAIInferenceService;

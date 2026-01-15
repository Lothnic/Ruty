//! Shared types for backend communication

use serde::{Deserialize, Serialize};

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: String,
    pub model: String,
    pub has_api_key: bool,
}

/// Search result from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendSearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub category: String,
}

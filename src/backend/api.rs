//! HTTP client for Python backend API

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::sidecar::backend_url;

/// Client for communicating with Python FastAPI backend
#[derive(Clone)]
pub struct BackendClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_keys: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    #[serde(default)]
    pub tools_used: Vec<String>,
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub provider: String,
    pub model: String,
    pub sessions_active: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Provider {
    pub name: String,
    pub display_name: String,
    pub models: Vec<String>,
    pub requires_api_key: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProvidersResponse {
    pub providers: Vec<Provider>,
    pub current_provider: String,
    pub current_model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRequest {
    pub session_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextResponse {
    pub success: bool,
    pub files_loaded: usize,
    pub message: String,
}

impl BackendClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: backend_url(),
        }
    }
    
    pub fn with_url(url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: url.to_string(),
        }
    }

    /// Check if backend is healthy
    pub async fn health_check(&self) -> Result<HealthResponse, String> {
        let url = format!("{}/health", self.base_url);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    /// Send a chat message to the AI (blocking, full response)
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        let url = format!("{}/chat", self.base_url);
        self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Load local files as context
    pub async fn load_context(&self, session_id: &str, path: &str) -> Result<ContextResponse, String> {
        let url = format!("{}/context/load", self.base_url);
        let request = ContextRequest {
            session_id: session_id.to_string(),
            path: path.to_string(),
        };
        self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Clear context for session
    pub async fn clear_context(&self, session_id: &str) -> Result<(), String> {
        let url = format!("{}/context/clear/{}", self.base_url, session_id);
        self.client
            .delete(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    /// Get available providers
    pub async fn get_providers(&self) -> Result<ProvidersResponse, String> {
        let url = format!("{}/providers", self.base_url);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Update provider configuration
    pub async fn update_provider(&self, request: ProviderUpdateRequest) -> Result<(), String> {
        let url = format!("{}/providers/update", self.base_url);
        let resp = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Provider update failed: {}", resp.status()))
        }
    }
}

impl Default for BackendClient {
    fn default() -> Self {
        Self::new()
    }
}

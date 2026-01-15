//! HTTP client for Python backend API

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Client for communicating with Python FastAPI backend
pub struct BackendClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: String,
    pub local_context: Option<String>,
    pub api_keys: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub tools_used: Vec<String>,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub provider: String,
    pub model: String,
    pub sessions_active: u32,
}

impl BackendClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
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

    /// Send a chat message to the AI
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
}

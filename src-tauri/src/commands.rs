//! Tauri v2 commands for frontend-backend IPC

use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;

const API_BASE: &str = "http://127.0.0.1:3847";

#[derive(Serialize, Deserialize)]
pub struct ChatRequest {
    message: String,
    session_id: String,
    local_context: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ChatResponse {
    response: String,
    tools_used: Vec<String>,
    session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ContextRequest {
    path: String,
    session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ContextResponse {
    success: bool,
    loaded: Option<String>,
    #[serde(rename = "type")]
    context_type: Option<String>,
    error: Option<String>,
}

/// Send a chat message to the Python backend
#[tauri::command]
pub async fn send_message(message: String, session_id: String) -> Result<ChatResponse, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/chat", API_BASE))
        .json(&ChatRequest {
            message,
            session_id,
            local_context: None,
        })
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    response
        .json::<ChatResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Toggle the main window visibility
#[tauri::command]
pub fn toggle_window_cmd(window: WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.center();
        let _ = window.set_focus();
    }
}

/// Load local context from a file or directory
#[tauri::command]
pub async fn load_context(path: String, session_id: String) -> Result<ContextResponse, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/context/load", API_BASE))
        .json(&ContextRequest { path, session_id })
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    response
        .json::<ContextResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Clear local context for a session
#[tauri::command]
pub async fn clear_context(session_id: String) -> Result<bool, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/context/clear?session_id={}", API_BASE, session_id))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    Ok(response.status().is_success())
}

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

// ==================== Application Launcher ====================

use super::apps::{AppIndexer, Application};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global app indexer (singleton, lazily initialized)
static APP_INDEXER: Lazy<Mutex<AppIndexer>> = Lazy::new(|| {
    Mutex::new(AppIndexer::new())
});

/// Application result for frontend
#[derive(Serialize)]
pub struct AppResult {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub icon: Option<String>,
}

impl From<&Application> for AppResult {
    fn from(app: &Application) -> Self {
        Self {
            id: app.id.clone(),
            name: app.name.clone(),
            subtitle: app.generic_name.clone()
                .or(app.comment.clone())
                .unwrap_or_else(|| app.exec.split_whitespace().next().unwrap_or("").to_string()),
            icon: app.icon.clone(),
        }
    }
}

/// Search for applications
#[tauri::command]
pub fn search_apps(query: String) -> Vec<AppResult> {
    let indexer = APP_INDEXER.lock().unwrap();
    indexer.search(&query)
        .into_iter()
        .map(AppResult::from)
        .collect()
}

/// Launch an application by ID
#[tauri::command]
pub fn launch_app(app_id: String) -> Result<String, String> {
    let indexer = APP_INDEXER.lock().unwrap();
    
    // Find app by ID
    let app = indexer.all()
        .iter()
        .find(|a| a.id == app_id)
        .ok_or_else(|| format!("App not found: {}", app_id))?;
    
    app.launch()?;
    
    Ok(format!("Launched: {}", app.name))
}

/// Refresh the application index
#[tauri::command]
pub fn refresh_apps() -> usize {
    let mut indexer = APP_INDEXER.lock().unwrap();
    *indexer = AppIndexer::new();
    indexer.all().len()
}

// ==================== File Search ====================

use super::files::{FileSearcher, FileResult};

/// Global file searcher (lazily initialized)
static FILE_SEARCHER: Lazy<Mutex<FileSearcher>> = Lazy::new(|| {
    Mutex::new(FileSearcher::new())
});

/// Search for files
#[tauri::command]
pub fn search_files(query: String, max_results: Option<usize>, folders_only: Option<bool>) -> Vec<FileResult> {
    let searcher = FILE_SEARCHER.lock().unwrap();
    searcher.search(&query, max_results.unwrap_or(15), folders_only.unwrap_or(false))
}

/// Open a file with default application
#[tauri::command]
pub fn open_file(path: String) -> Result<String, String> {
    let searcher = FILE_SEARCHER.lock().unwrap();
    searcher.open(&path)?;
    Ok(format!("Opened: {}", path))
}

/// Reveal file in file manager
#[tauri::command]
pub fn reveal_file(path: String) -> Result<String, String> {
    let searcher = FILE_SEARCHER.lock().unwrap();
    searcher.reveal(&path)?;
    Ok(format!("Revealed: {}", path))
}

// ==================== Clipboard Manager ====================

use super::clipboard::{ClipboardManager, ClipboardItem};

/// Global clipboard manager (lazily initialized)
static CLIPBOARD_MANAGER: Lazy<Mutex<ClipboardManager>> = Lazy::new(|| {
    Mutex::new(ClipboardManager::new())
});

/// Start clipboard monitor
#[tauri::command]
pub fn init_clipboard() -> Result<String, String> {
    let manager = CLIPBOARD_MANAGER.lock().unwrap();
    manager.start();
    Ok("Clipboard monitor started".to_string())
}

/// Get clipboard history
#[tauri::command]
pub fn get_clipboard_history() -> Vec<ClipboardItem> {
    let manager = CLIPBOARD_MANAGER.lock().unwrap();
    manager.get_history()
}

/// Copy text to clipboard (moves to top of history)
#[tauri::command]
pub fn copy_to_clipboard(content: String) -> Result<String, String> {
    // Only works if arboard is used or we shell out to wl-copy/xclip
    // Because we are using shell-out for reading, let's use it for writing too
    // to keep it consistent.
    
    // Try wl-copy first
    use std::process::{Command, Stdio};
    use std::io::Write;
    
    let child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn();
        
    if let Ok(mut child) = child {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(content.as_bytes());
        }
        let _ = child.wait();
        return Ok("Copied via wl-copy".to_string());
    }
    
    // Try xclip
    let child = Command::new("xclip")
        .args(["-selection", "clipboard", "-i"])
        .stdin(Stdio::piped())
        .spawn();
        
    if let Ok(mut child) = child {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(content.as_bytes());
        }
        let _ = child.wait();
        return Ok("Copied via xclip".to_string());
    }

    Err("Failed to copy: no clipboard tool found".to_string())
}

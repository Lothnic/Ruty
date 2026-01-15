//! gRPC client for CLI commands
//!
//! Sends commands to the running Ruty daemon.

use super::proto::ruty_service_client::RutyServiceClient;
use super::proto::Empty;
use super::daemon_addr;

/// Check if daemon is running
pub async fn is_daemon_running() -> bool {
    match RutyServiceClient::connect(daemon_addr()).await {
        Ok(mut client) => client.ping(Empty {}).await.is_ok(),
        Err(_) => false,
    }
}

/// Toggle window visibility (main command for keybind)
pub async fn toggle_window() -> Result<bool, String> {
    let mut client = RutyServiceClient::connect(daemon_addr())
        .await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    let response = client
        .toggle_window(Empty {})
        .await
        .map_err(|e| format!("Toggle failed: {}", e))?;

    Ok(response.into_inner().visible)
}

/// Show window
pub async fn show_window() -> Result<(), String> {
    let mut client = RutyServiceClient::connect(daemon_addr())
        .await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    client
        .show_window(Empty {})
        .await
        .map_err(|e| format!("Show failed: {}", e))?;

    Ok(())
}

/// Hide window
pub async fn hide_window() -> Result<(), String> {
    let mut client = RutyServiceClient::connect(daemon_addr())
        .await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    client
        .hide_window(Empty {})
        .await
        .map_err(|e| format!("Hide failed: {}", e))?;

    Ok(())
}

/// Quit daemon
pub async fn quit_daemon() -> Result<(), String> {
    let mut client = RutyServiceClient::connect(daemon_addr())
        .await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    client
        .quit(Empty {})
        .await
        .map_err(|e| format!("Quit failed: {}", e))?;

    Ok(())
}

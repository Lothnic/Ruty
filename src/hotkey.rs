//! Global hotkey handling using `global-hotkey` crate
//!
//! Listens for Super+Space to toggle window visibility.

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::{Code, HotKey, Modifiers}};
use iced::Subscription;
use iced::time;
use std::sync::OnceLock;
use std::time::Duration;

/// Static hotkey manager (must persist for lifetime of app)
static HOTKEY_MANAGER: OnceLock<GlobalHotKeyManager> = OnceLock::new();

/// Hotkey ID for Super+Space
static HOTKEY_ID: OnceLock<u32> = OnceLock::new();

/// Initialize the global hotkey system
pub fn init_hotkeys() -> Result<(), String> {
    let manager = GlobalHotKeyManager::new()
        .map_err(|e| format!("Failed to create hotkey manager: {}", e))?;
    
    // Super+Space (Super = Meta on Linux)
    let hotkey = HotKey::new(Some(Modifiers::SUPER), Code::Space);
    
    manager.register(hotkey)
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;
    
    HOTKEY_MANAGER.set(manager)
        .map_err(|_| "Hotkey manager already initialized".to_string())?;
    HOTKEY_ID.set(hotkey.id())
        .map_err(|_| "Hotkey ID already set".to_string())?;
    
    tracing::info!("Global hotkey registered: Super+Space");
    Ok(())
}

/// Check if hotkey was pressed (called from time subscription)
pub fn check_hotkey_pressed() -> bool {
    let receiver = GlobalHotKeyEvent::receiver();
    if let Ok(event) = receiver.try_recv() {
        if event.state == HotKeyState::Pressed {
            if let Some(id) = HOTKEY_ID.get() {
                return event.id == *id;
            }
        }
    }
    false
}

/// Time tick event for polling
#[derive(Debug, Clone)]
pub struct HotkeyTick;

/// Create a time-based subscription that fires tick events for hotkey polling
pub fn hotkey_tick_subscription() -> Subscription<HotkeyTick> {
    time::every(Duration::from_millis(50)).map(|_| HotkeyTick)
}

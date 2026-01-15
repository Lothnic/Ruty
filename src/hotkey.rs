//! Global hotkey handling using `global-hotkey` crate + Unix signals
//!
//! On X11: Uses global-hotkey for Super+Space
//! On Wayland: Uses SIGUSR1 signal for system keybind integration

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::{Code, HotKey, Modifiers}};
use iced::Subscription;
use iced::time;
use signal_hook::consts::SIGUSR1;
use signal_hook::iterator::Signals;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

/// Static hotkey manager (must persist for lifetime of app)
static HOTKEY_MANAGER: OnceLock<GlobalHotKeyManager> = OnceLock::new();

/// Hotkey ID for Super+Space
static HOTKEY_ID: OnceLock<u32> = OnceLock::new();

/// Atomic flag for SIGUSR1 signal received
static SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);

/// Initialize the global hotkey system (X11) and signal handler (Wayland)
pub fn init_hotkeys() -> Result<(), String> {
    // Try X11 global hotkey first
    match GlobalHotKeyManager::new() {
        Ok(manager) => {
            let hotkey = HotKey::new(Some(Modifiers::SUPER), Code::Space);
            if let Err(e) = manager.register(hotkey) {
                tracing::warn!("Failed to register X11 hotkey: {}", e);
            } else {
                HOTKEY_MANAGER.set(manager).ok();
                HOTKEY_ID.set(hotkey.id()).ok();
                tracing::info!("Global hotkey registered: Super+Space (X11)");
            }
        }
        Err(e) => {
            tracing::warn!("X11 hotkey manager unavailable: {}", e);
        }
    }
    
    // Also set up SIGUSR1 handler for Wayland compatibility
    std::thread::spawn(|| {
        if let Ok(mut signals) = Signals::new([SIGUSR1]) {
            tracing::info!("SIGUSR1 signal handler ready (for Wayland keybind)");
            for _ in signals.forever() {
                tracing::info!("SIGUSR1 received - toggling window");
                SIGNAL_RECEIVED.store(true, Ordering::SeqCst);
            }
        }
    });
    
    Ok(())
}

/// Check if hotkey was pressed (via X11 or SIGUSR1 signal)
pub fn check_hotkey_pressed() -> bool {
    // Check SIGUSR1 signal first (Wayland)
    if SIGNAL_RECEIVED.swap(false, Ordering::SeqCst) {
        return true;
    }
    
    // Then check X11 global hotkey
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

//! Ruty - AI Assistant with Supermemory
//!
//! Main entry point for the Tauri v2 application.
//! Handles window management, system tray, and global shortcuts.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(debug_assertions)]
use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, WebviewWindow,
};

mod commands;

#[cfg(debug_assertions)]
struct PythonBackend(Mutex<Option<Child>>);

#[cfg(not(debug_assertions))]
struct PythonBackend(Mutex<Option<tauri_plugin_shell::process::CommandChild>>);

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(PythonBackend(Mutex::new(None)))
        .setup(|app| {
            // Start Python backend
            spawn_python_backend(app.handle())?;

            // Create system tray
            create_tray(app.handle())?;

            // Register global shortcut: Super+Space (non-fatal if fails)
            if let Err(e) = register_global_shortcut(app.handle()) {
                eprintln!("⚠️  Hotkey registration failed: {}", e);
                eprintln!("   Use system tray to open Ruty instead");
            }

            println!("✅ Ruty started!");
            println!("   Press Super+Space to toggle window (or use tray)");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::send_message,
            commands::toggle_window_cmd,
            commands::load_context,
            commands::clear_context,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Create system tray with menu
fn create_tray(app: &AppHandle) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let toggle = MenuItem::with_id(app, "toggle", "Toggle Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle, &quit])?;

    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Ruty - AI Assistant")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => {
                if let Some(window) = app.get_webview_window("main") {
                    toggle_window(&window);
                }
            }
            "quit" => {
                // Stop Python backend
                if let Some(state) = app.try_state::<PythonBackend>() {
                    if let Ok(mut guard) = state.0.lock() {
                        if let Some(child) = guard.take() {
                            let _ = child.kill();
                        }
                    }
                }
                std::process::exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    toggle_window(&window);
                }
            }
        })
        .build(app)?;

    Ok(tray)
}

/// Register Super+Space global shortcut
fn register_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    let shortcut: Shortcut = "Super+Space".parse()?;

    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
        if let Some(window) = app_handle.get_webview_window("main") {
            toggle_window(&window);
        }
    })?;

    // Try to register, but don't fail if already registered
    match app.global_shortcut().register(shortcut) {
        Ok(_) => println!("✓ Registered Super+Space hotkey"),
        Err(e) => eprintln!("⚠️ Could not register Super+Space (may be used by another app): {}", e),
    }

    Ok(())
}

/// Toggle window visibility
fn toggle_window(window: &WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.center();
        let _ = window.set_focus();
    }
}

/// Spawn the Python FastAPI backend as a subprocess
fn spawn_python_backend(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<PythonBackend>();

    #[cfg(debug_assertions)]
    {
        // DEVELOPMENT: Use 'uv run' to ensure correct venv activation
        let project_root = std::env::current_dir()
            .map(|p| p.parent().map(|pp| pp.to_path_buf()).unwrap_or(p))
            .unwrap_or_else(|_| std::path::PathBuf::from(".."));

        println!("📁 Running Python from: {:?}", project_root);

        let child = Command::new("uv")
            .args(["run", "python", "-m", "ruty.server"])
            .current_dir(&project_root)
            .spawn();

        match child {
            Ok(process) => {
                println!("🐍 Python backend started (PID: {})", process.id());
                if let Ok(mut guard) = state.0.lock() {
                    *guard = Some(process);
                }
            }
            Err(e) => {
                eprintln!("⚠️  Could not start Python backend: {}", e);
                eprintln!("   Start manually: cd {:?} && uv run python -m ruty.server", project_root);
            }
        }
    }

    #[cfg(not(debug_assertions))]
    {
        // RELEASE: Use bundled sidecar
        use tauri_plugin_shell::ShellExt;
        
        println!("🚀 Starting bundled Python backend...");
        
        let sidecar = app.shell().sidecar("ruty-backend")?;
        let (mut _rx, child) = sidecar.spawn()?;
        
        println!("🐍 Python backend started (Sidecar)");
        if let Ok(mut guard) = state.0.lock() {
            *guard = Some(child);
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    main();
}

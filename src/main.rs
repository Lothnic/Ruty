//! Ruty: AI-powered productivity launcher
//!
//! Gauntlet-style daemon architecture with gRPC IPC.
//!
//! Usage:
//!   ruty           - Start daemon (or connect to existing)
//!   ruty open      - Show window (toggle if visible)
//!   ruty close     - Hide window
//!   ruty quit      - Stop daemon
//!   ruty help      - Show help

mod app;
mod ui;
mod backend;
mod native;
mod hotkey;
mod ipc;
mod rpc;
mod commands;

use std::sync::Arc;
use app::Ruty;
use iced::{window, Size};
use rpc::server::WindowController;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Global window controller shared between RPC server and Iced app
static WINDOW_CONTROLLER: std::sync::OnceLock<Arc<WindowController>> = std::sync::OnceLock::new();

fn main() -> iced::Result {
    // Parse CLI arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        return handle_cli_command(&args[1]);
    }
    
    // No args = start daemon mode
    start_daemon()
}

fn handle_cli_command(cmd: &str) -> iced::Result {
    // Initialize minimal logging for CLI
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
    
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    
    match cmd {
        "open" | "toggle" => {
            let is_running = rt.block_on(rpc::client::is_daemon_running());
            
            if is_running {
                rt.block_on(async {
                    match rpc::client::toggle_window().await {
                        Ok(visible) => {
                            println!("Window is now {}", if visible { "visible" } else { "hidden" });
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                });
                Ok(())
            } else {
                println!("Daemon not running. Starting daemon...");
                drop(rt);
                start_daemon()
            }
        }
        "close" | "hide" => {
            rt.block_on(async {
                if rpc::client::is_daemon_running().await {
                    match rpc::client::hide_window().await {
                        Ok(_) => println!("Window hidden"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    println!("Daemon is not running");
                }
            });
            Ok(())
        }
        "quit" | "exit" | "stop" => {
            rt.block_on(async {
                if rpc::client::is_daemon_running().await {
                    match rpc::client::quit_daemon().await {
                        Ok(_) => println!("Daemon stopped"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    println!("Daemon is not running");
                }
            });
            Ok(())
        }
        "status" => {
            rt.block_on(async {
                if rpc::client::is_daemon_running().await {
                    println!("Daemon is running");
                } else {
                    println!("Daemon is not running");
                }
            });
            Ok(())
        }
        "help" | "--help" | "-h" => {
            println!("Ruty - AI-powered productivity launcher\n");
            println!("Usage: ruty [command]\n");
            println!("Commands:");
            println!("  (none)        Start daemon (or show window if already running)");
            println!("  open, toggle  Toggle window visibility");
            println!("  close, hide   Hide window");
            println!("  quit, stop    Stop daemon");
            println!("  status        Check if daemon is running");
            println!("  help          Show this help message");
            println!("\nSet Super+Space keybind to: ruty open");
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("Run 'ruty help' for usage");
            Ok(())
        }
    }
}

fn start_daemon() -> iced::Result {
    // Initialize logging (use try_init to avoid panic if already initialized by CLI)
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    tracing::info!("Starting Ruty daemon...");

    // Create shared window controller
    let controller = Arc::new(WindowController::new());
    WINDOW_CONTROLLER.set(controller.clone()).expect("Controller already set");

    // Start gRPC server in background
    let server_controller = controller.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = rpc::server::start_server(server_controller).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
    });

    // Initialize global hotkey (works on X11)
    if let Err(e) = hotkey::init_hotkeys() {
        tracing::warn!("Could not register global hotkey: {} (use 'ruty open' instead)", e);
    }

    tracing::info!("Ruty daemon started. Use 'ruty open' to toggle window.");

    // Start Iced application
    iced::application("Ruty", Ruty::update, Ruty::view)
        .subscription(Ruty::subscription)
        .theme(Ruty::theme)
        .window(window::Settings {
            size: Size::new(700.0, 400.0),
            position: window::Position::Centered,
            decorations: false,
            transparent: true,
            level: window::Level::AlwaysOnTop,
            resizable: true,
            ..Default::default()
        })
        .antialiasing(true)
        .run()
}

/// Get the global window controller
pub fn get_window_controller() -> Option<Arc<WindowController>> {
    WINDOW_CONTROLLER.get().cloned()
}

//! Ruty: AI-powered productivity launcher
//!
//! A Raycast-inspired application launcher with:
//! - Application launcher
//! - File search
//! - Clipboard history
//! - AI chat with memory (via Python backend)

mod app;
mod ui;
mod backend;
mod native;
mod hotkey;
mod ipc;

use app::Ruty;
use iced::{window, Size};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> iced::Result {
    // Check for CLI commands first
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "toggle" => {
                // Try to toggle existing instance, or start new one
                if ipc::try_toggle_existing() {
                    println!("Toggled existing Ruty instance");
                    std::process::exit(0);
                } else {
                    println!("No running instance, starting new one...");
                    // Fall through to start the app
                }
            }
            "close" | "quit" => {
                if ipc::try_close_existing() {
                    println!("Closed Ruty");
                } else {
                    println!("No running instance found");
                }
                std::process::exit(0);
            }
            "help" | "--help" | "-h" => {
                println!("Ruty - AI-powered productivity launcher\n");
                println!("Usage: ruty [command]\n");
                println!("Commands:");
                println!("  toggle    Toggle window visibility (or start if not running)");
                println!("  close     Close running instance");
                println!("  help      Show this help message");
                println!("\nWithout arguments, starts the launcher.");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Run 'ruty help' for usage");
                std::process::exit(1);
            }
        }
    }

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Ruty...");

    // Start IPC server for receiving toggle commands
    ipc::start_server();

    // Initialize global hotkey (works on X11, fallback for Wayland)
    if let Err(e) = hotkey::init_hotkeys() {
        tracing::warn!("Could not register global hotkey: {} (use 'ruty toggle' instead)", e);
    }

    // Use iced::application() builder pattern for 0.13
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

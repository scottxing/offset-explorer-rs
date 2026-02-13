// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Main entry point - Tauri desktop application

mod config;
mod ui_events;
mod kafka;
mod zookeeper;
mod schema_registry;
mod decoders;
mod models;
mod async_ops;
mod acls;
mod zk_browser;
mod tauri_commands;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Offset Explorer Rust v3.0.3");
    info!("Configuration directory: ~/.offsetexplorer3/");

    // Create application state
    let app_state = Arc::new(tauri_commands::AppState::new());

    // Run Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            info!("Tauri application initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

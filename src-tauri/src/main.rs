// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Main entry point - Tauri desktop application

mod acls;
mod async_ops;
mod config;
mod decoders;
mod kafka;
mod models;
mod schema_registry;
mod tauri_commands;
mod ui_events;
mod zk_browser;
mod zookeeper;

use std::sync::Arc;
use tauri_commands::AppState;
use tracing::info;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Offset Explorer Rust v3.0.3");
    info!("Configuration directory: ~/.offsetexplorer3/");

    // Create application state
    let app_state = Arc::new(AppState::new());

    // Run Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Server management
            tauri_commands::get_server_connections,
            tauri_commands::add_server_connection,
            tauri_commands::update_server_connection,
            tauri_commands::remove_server_connection,
            tauri_commands::connect_to_server,
            tauri_commands::disconnect_from_server,
            // Topic management
            tauri_commands::list_topics,
            tauri_commands::create_topic,
            tauri_commands::delete_topic,
            tauri_commands::get_topic_metadata,
            tauri_commands::get_topic_partitions,
            // Message operations
            tauri_commands::consume_messages,
            tauri_commands::produce_message,
            // Consumer groups
            tauri_commands::list_consumer_groups,
            tauri_commands::get_consumer_group_details,
            tauri_commands::reset_consumer_offset,
            // Tasks
            tauri_commands::get_task_progress,
            tauri_commands::cancel_task,
            tauri_commands::list_tasks,
            // Brokers
            tauri_commands::list_brokers,
            // ACLs
            tauri_commands::list_acls,
            tauri_commands::create_acl,
            tauri_commands::delete_acl,
            // Schema Registry
            tauri_commands::list_schema_subjects,
            tauri_commands::get_schema,
            tauri_commands::get_latest_schema,
            tauri_commands::register_schema,
            tauri_commands::test_compatibility,
        ])
        .setup(|_app| {
            info!("Tauri application initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

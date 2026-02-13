// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Environment singleton - manages global application state and configuration
// Compatible with Java Environment class

use crate::config::{settings::UserSettings, server_connection_clean::{ServerConnection, ServerConnectionSettings}, server_group::ServerGroupManager}, ui_events::EventBus};
use crate::ui_events::EventEvent;
use anyhow::{Result, Context, anyhow};
use std::path::PathBuf;
use std::fs;
use tracing::{info, error, warn};
use serde_json;
use tokio::sync::RwLock;

/// Application event types
#[derive(Debug, Clone, serde::Serialize)]
pub enum AppEventType {
    ServerConnectionAdded { id: i64, name: String },
    ServerConnectionRemoved { id: i64 },
    ServerConnected { id: i64 },
    ServerDisconnected { id: i64 },
    TopicAdded { name: String },
    TopicRemoved { name: String },
    ConsumerAdded { group_id: String },
    ConsumerRemoved { group_id: String },
    MessageAdded { topic: String, partition: i32, offset: i64, key: Option<String>, value: Option<String> },
    SettingsChanged,
}

impl AppEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppEventType::ServerConnectionAdded { id: _, name: _ } => "ServerConnectionAdded",
            AppEventType::ServerConnectionRemoved { id: _, name: _ } => "ServerConnectionRemoved",
            AppEventType::ServerConnected { id: _, name: _ } => "ServerConnected",
            AppEventType::ServerDisconnected { id: _, name: _ } => "ServerDisconnected",
            AppEventType::TopicAdded { name: _, name: _ } => "TopicAdded",
            AppEventType::TopicRemoved { name: _, name: _ } => "TopicRemoved",
            AppEventType::ConsumerAdded { group_id: _, name: _ } => "ConsumerAdded",
            AppEventType::ConsumerRemoved { group_id: _, name: _ } => "ConsumerRemoved",
            AppEventType::MessageAdded { topic: _, partition: _, offset: _, key: _, name: _, name: _, name: _ } => "MessageAdded",
            AppEventType::SettingsChanged => "SettingsChanged",
        }
    }
}

/// Application event with optional target
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppEvent {
    pub event_type: AppEventType,
    pub topic: Option<String>,
    pub partition: Option<i32>,
    pub offset: Option<i64>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub server_id: Option<i64>,
    pub group_id: Option<String>,
}

/// Configuration directory paths
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub user_home: PathBuf,
    pub config_dir: PathBuf,
    pub settings_file: PathBuf,
    pub server_groups_file: PathBuf,
    pub connections_file: PathBuf,
    pub browser_history_file: PathBuf,
    pub license_file: PathBuf,
}

impl ConfigPaths {
    pub fn new(user_home: PathBuf) -> Self {
        let config_dir = user_home.join(".offsetexplorer3");
        Self {
            user_home,
            config_dir,
            settings_file: config_dir.join("settings.xml"),
            server_groups_file: config_dir.join("servergroups.xml"),
            connections_file: config_dir.join("connections.xml"),
            browser_history_file: config_dir.join("browserhistory.xml"),
            license_file: config_dir.join("license.ktl"),
        }
    }
}

/// Global environment singleton
pub struct Environment {
    pub paths: ConfigPaths,
    pub settings: UserSettings,
    pub server_group_manager: ServerGroupManager,
    pub connection_settings: ServerConnectionSettings,
    pub event_bus: EventBus,
    pub event_tx: tokio::sync::broadcast::Sender<AppEvent>,
}

impl Environment {
    /// Create a new environment instance
    pub fn new() -> Result<Self> {
        let paths = ConfigPaths::new(dirs::home_dir()?);

        let event_bus = EventBus::new();
        let (event_tx, _) = event_bus.create_channel();

        let settings = UserSettings::new();

        // Try to load existing settings
        // For now, use defaults

        let server_group_manager = ServerGroupManager::new();

        let connection_settings = ServerConnectionSettings::new();

        Ok(Self {
            paths,
            settings,
            server_group_manager,
            connection_settings,
            event_bus,
            event_tx,
        })
    }

    /// Load settings from XML files
    pub fn load_settings(&mut self) -> Result<()> {
        // TODO: Load from ~/.offsetexplorer3/
        info!("Loading application settings");
        Ok(())
    }

    /// Save all settings to XML files
    pub fn save_settings(&self) -> Result<()> {
        info!("Saving application settings");
        // TODO: Save to ~/.offsetexplorer3/
        Ok(())
    }

    /// Get initial server connection count
    pub fn get_initial_server_count(&self) -> usize {
        self.connection_settings.get_connections().len()
    }

    /// Find server connection by ID
    pub fn find_connection(&self, id: i64) -> Option<ServerConnection> {
        self.connection_settings.find_connection(id)
    }

    /// Add a new server connection
    pub fn add_connection(&mut self, mut connection: ServerConnection) -> Result<()> {
        self.connection_settings.add_connection(connection.clone())?;

        // Emit event
        let event = AppEvent {
            event_type: AppEventType::ServerConnectionAdded,
            server_id: Some(connection.get_id()),
            topic: None,
            partition: None,
            offset: None,
            key: None,
            value: None,
            name: Some(connection.get_name().to_string()),
        };
        let _ = self.event_tx.send(event).ok();

        info!("Server connection added: {}", connection.get_name());
        Ok(())
    }

    /// Remove a server connection
    pub fn remove_connection(&mut self, id: i64) -> Result<()> {
        self.connection_settings.remove_connection(id)?;

        // Emit event
        let event = AppEvent {
            event_type: AppEventType::ServerConnectionRemoved,
            server_id: Some(id),
            topic: None,
            partition: None,
            offset: None,
            key: None,
            value: None,
            name: None,
        };
        let _ = self.event_tx.send(event).ok();

        info!("Server connection removed: {}", id);
        Ok(())
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AppEvent> {
        self.event_bus.subscribe()
    }

    /// Get event transmitter
    pub fn event_tx(&self) -> tokio::sync::broadcast::Sender<AppEvent> {
        self.event_tx.clone()
    }

    /// Emit an event
    pub fn emit_event(&self, event: AppEvent) {
        let _ = self.event_tx.send(event);
    }
}

/// Event bus for application-wide events
pub struct EventBus {
    pub sender: tokio::sync::broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    pub fn create_channel(&mut self) -> (tokio::sync::broadcast::Sender<AppEvent>, tokio::sync::broadcast::Receiver<AppEvent>) {
        tokio::sync::broadcast::channel(100)
    }

    pub fn publish(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }
}

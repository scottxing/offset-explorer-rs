// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Event bus for reactive UI updates

use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

/// Application event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    ServerConnectionAdded { id: i64, name: String },
    ServerConnectionRemoved { id: i64 },
    ServerConnectionConnected { id: i64 },
    ServerConnectionDisconnected { id: i64 },
    TopicAdded { name: String },
    TopicRemoved { name: String },
    ConsumerAdded { group_id: String },
    ConsumerRemoved { group_id: String },
}

/// Event bus for application-wide events
pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

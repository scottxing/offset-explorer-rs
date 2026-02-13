// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Broker model

use serde::{Serialize, Deserialize};

/// Broker model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Broker {
    pub id: i32,
    pub host: String,
    pub port: i32,
}

impl Broker {
    pub fn new(id: i32, host: String, port: i32) -> Self {
        Self { id, host, port }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_port(&self) -> i32 {
        self.port
    }
}

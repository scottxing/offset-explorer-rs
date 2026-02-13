// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Topic model

use serde::{Serialize, Deserialize};

/// Topic model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub partition_count: i32,
    pub replication_factor: i32,
}

impl Topic {
    pub fn new(name: String, partition_count: i32, replication_factor: i32) -> Self {
        Self {
            name,
            partition_count,
            replication_factor,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

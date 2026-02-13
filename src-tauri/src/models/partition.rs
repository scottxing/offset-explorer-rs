// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Partition model

use serde::{Serialize, Deserialize};

/// Partition model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub id: i32,
    pub topic_name: String,
    pub leader: i32,
}

impl Partition {
    pub fn new(id: i32, topic_name: String, leader: i32) -> Self {
        Self {
            id,
            topic_name,
            leader,
        }
    }
}

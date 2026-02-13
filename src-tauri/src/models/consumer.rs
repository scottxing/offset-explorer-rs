// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Consumer model

use serde::{Serialize, Deserialize};

/// Consumer group model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumer {
    pub group_id: String,
}

impl Consumer {
    pub fn new(group_id: String) -> Self {
        Self { group_id }
    }

    pub fn get_group_id(&self) -> &str {
        &self.group_id
    }
}

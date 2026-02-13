// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// No-key decoder for messages without keys

use anyhow::Result;

/// No key decoder
pub struct NoKeyDecoder;

impl NoKeyDecoder {
    pub fn decode(&self, _data: &[u8]) -> Result<String> {
        Ok("<no key>".to_string())
    }
}

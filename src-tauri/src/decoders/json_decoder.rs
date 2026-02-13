// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// JSON message decoder

use anyhow::Result;
use serde_json::Value;

/// JSON decoder with pretty printing
pub struct JsonDecoder {
    pretty: bool,
    color: bool,
}

impl JsonDecoder {
    /// Create a new JSON decoder
    pub fn new() -> Self {
        Self {
            pretty: true,
            color: false,
        }
    }

    /// Set pretty printing
    pub fn set_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Decode bytes to JSON string
    pub fn decode(&self, data: &[u8]) -> Result<String> {
        // First, try to decode as UTF-8 string
        let text = String::from_utf8_lossy(data);

        // Try to parse as JSON
        let value: Value = serde_json::from_str(&text)?;

        if self.pretty {
            // Pretty print with 2-space indentation
            Ok(serde_json::to_string_pretty(&value)?)
        } else {
            // Compact JSON
            Ok(serde_json::to_string(&value)?)
        }
    }

    /// Decode and validate JSON, returning structured value
    pub fn decode_value(&self, data: &[u8]) -> Result<Value> {
        let text = String::from_utf8_lossy(data);
        Ok(serde_json::from_str(&text)?)
    }
}

impl Default for JsonDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_decode() {
        let decoder = JsonDecoder::new();
        let data = br#"{"name":"test","value":123}"#;
        let result = decoder.decode(data).unwrap();
        assert!(result.contains("name"));
        assert!(result.contains("test"));
    }

    #[test]
    fn test_json_pretty() {
        let decoder = JsonDecoder::new().set_pretty(true);
        let data = br#"{"name":"test","value":123}"#;
        let result = decoder.decode(data).unwrap();
        // Pretty printed JSON should have newlines
        assert!(result.contains('\n'));
    }
}

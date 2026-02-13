// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Base64 message decoder

use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};

/// Base64 decoder
pub struct Base64Decoder {
    url_safe: bool,
}

impl Base64Decoder {
    /// Create a new Base64 decoder (standard)
    pub fn new() -> Self {
        Self { url_safe: false }
    }

    /// Create a URL-safe Base64 decoder
    pub fn url_safe() -> Self {
        Self { url_safe: true }
    }

    /// Decode Base64 bytes to string
    pub fn decode(&self, data: &[u8]) -> Result<String> {
        // First, decode as UTF-8 to get Base64 string
        let base64_str = String::from_utf8_lossy(data).to_string();

        // Decode Base64
        let decoded_bytes = if self.url_safe {
            general_purpose::URL_SAFE.decode(&base64_str)
        } else {
            general_purpose::STANDARD.decode(&base64_str)
        };

        match decoded_bytes {
            Ok(bytes) => {
                // Try to decode as UTF-8 string
                Ok(String::from_utf8_lossy(&bytes).to_string())
            }
            Err(e) => {
                Ok(format!("<Invalid Base64: {}>", e))
            }
        }
    }

    /// Encode bytes to Base64 string
    pub fn encode(&self, data: &[u8]) -> String {
        if self.url_safe {
            general_purpose::URL_SAFE_NO_PAD.encode(data)
        } else {
            general_purpose::STANDARD.encode(data)
        }
    }
}

impl Default for Base64Decoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode() {
        let decoder = Base64Decoder::new();
        let data = b"SGVsbG8gV29ybGQh"; // "Hello World"
        let result = decoder.decode(data).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_base64_encode() {
        let decoder = Base64Decoder::new();
        let data = b"Hello";
        let result = decoder.encode(data);
        assert_eq!(result, "SGVsbG8=");
    }
}

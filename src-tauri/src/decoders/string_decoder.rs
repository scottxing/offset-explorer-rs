// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// String message decoder

use anyhow::Result;

/// Character encoding options
#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    Utf8,
    Utf16Le,
    Utf16Be,
    Gbk,
    Iso88591,
}

/// String decoder with support for multiple character encodings
pub struct StringDecoder {
    encoding: Encoding,
}

impl StringDecoder {
    /// Create a new string decoder with UTF-8 encoding
    pub fn new() -> Self {
        Self { encoding: Encoding::Utf8 }
    }

    /// Create a new string decoder with specified encoding
    pub fn with_encoding(encoding: Encoding) -> Self {
        Self { encoding }
    }

    /// Decode bytes to string
    pub fn decode(&self, data: &[u8]) -> Result<String> {
        match self.encoding {
            Encoding::Utf8 => {
                Ok(String::from_utf8_lossy(data).to_string())
            }
            Encoding::Utf16Le => {
                let mut u16_data = Vec::new();
                for chunk in data.chunks_exact(2) {
                    if chunk.len() == 2 {
                        let u16_val = u16::from_le_bytes([chunk[0], chunk[1]]);
                        u16_data.push(u16_val);
                    }
                }
                Ok(String::from_utf16_lossy(&u16_data).to_string())
            }
            Encoding::Utf16Be => {
                let mut u16_data = Vec::new();
                for chunk in data.chunks_exact(2) {
                    if chunk.len() == 2 {
                        let u16_val = u16::from_be_bytes([chunk[0], chunk[1]]);
                        u16_data.push(u16_val);
                    }
                }
                Ok(String::from_utf16_lossy(&u16_data).to_string())
            }
            Encoding::Gbk => {
                // GBK encoding - would need encoding_rs, fallback to lossy UTF-8
                Ok(format!("<GBK encoding not yet implemented, {} bytes>", data.len()))
            }
            Encoding::Iso88591 => {
                // ISO-8859-1 is a single-byte encoding, 1:1 mapping to Unicode codepoints
                let decoded: String = data.iter().map(|&b| b as char).collect();
                Ok(decoded)
            }
        }
    }

    /// Try to detect encoding and decode
    pub fn decode_auto(&self, data: &[u8]) -> Result<String> {
        // Try UTF-8 first
        if std::str::from_utf8(data).is_ok() {
            return Ok(String::from_utf8_lossy(data).to_string());
        }

        // Try UTF-16 LE
        if data.len() >= 2 && data.len() % 2 == 0 {
            let mut u16_data = Vec::new();
            for chunk in data.chunks_exact(2) {
                let u16_val = u16::from_le_bytes([chunk[0], chunk[1]]);
                u16_data.push(u16_val);
            }
            if let Ok(s) = String::from_utf16(&u16_data) {
                return Ok(s);
            }

            // Try UTF-16 BE
            let mut u16_data_be = Vec::new();
            for chunk in data.chunks_exact(2) {
                let u16_val = u16::from_be_bytes([chunk[0], chunk[1]]);
                u16_data_be.push(u16_val);
            }
            if let Ok(s) = String::from_utf16(&u16_data_be) {
                return Ok(s);
            }
        }

        // Fallback to lossy UTF-8
        Ok(String::from_utf8_lossy(data).to_string())
    }
}

impl Default for StringDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_decode() {
        let decoder = StringDecoder::new();
        let data = "Hello, 世界!".as_bytes();
        let result = decoder.decode(data).unwrap();
        assert_eq!(result, "Hello, 世界!");
    }

    #[test]
    fn test_auto_decode() {
        let decoder = StringDecoder::new();
        let utf8_data = "Hello".as_bytes();
        let result = decoder.decode_auto(utf8_data).unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_invalid_utf8() {
        let decoder = StringDecoder::new();
        let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
        let result = decoder.decode(invalid_utf8);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("�"));
    }
}

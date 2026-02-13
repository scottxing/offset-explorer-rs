// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Byte array message decoder

use anyhow::Result;

/// Display format for byte array
#[derive(Debug, Clone, Copy)]
pub enum ByteArrayFormat {
    /// Hexadecimal format (default)
    Hex,
    /// Binary format
    Binary,
    /// Octal format
    Octal,
    /// Decimal format (comma-separated)
    Decimal,
}

/// Byte array decoder with multiple display formats
pub struct ByteArrayDecoder {
    format: ByteArrayFormat,
    bytes_per_line: usize,
}

impl ByteArrayDecoder {
    /// Create a new byte array decoder with hex format
    pub fn new() -> Self {
        Self {
            format: ByteArrayFormat::Hex,
            bytes_per_line: 16,
        }
    }

    /// Create a new byte array decoder with specified format
    pub fn with_format(format: ByteArrayFormat) -> Self {
        Self {
            format,
            bytes_per_line: 16,
        }
    }

    /// Set bytes per line for formatting
    pub fn set_bytes_per_line(mut self, bytes: usize) -> Self {
        self.bytes_per_line = bytes;
        self
    }

    /// Decode bytes to string representation
    pub fn decode(&self, data: &[u8]) -> Result<String> {
        match self.format {
            ByteArrayFormat::Hex => self.decode_hex(data),
            ByteArrayFormat::Binary => self.decode_binary(data),
            ByteArrayFormat::Octal => self.decode_octal(data),
            ByteArrayFormat::Decimal => self.decode_decimal(data),
        }
    }

    /// Decode to hexadecimal format with line breaks
    fn decode_hex(&self, data: &[u8]) -> Result<String> {
        let mut result = String::new();
        for (i, byte) in data.iter().enumerate() {
            // Add offset at start of line
            if i % self.bytes_per_line == 0 {
                if i > 0 {
                    result.push('\n');
                }
                result.push_str(&format!("{:08x}:  ", i));
            }

            // Add byte in hex
            result.push_str(&format!("{:02x} ", byte));

            // Add extra space after 8 bytes for readability
            if (i % self.bytes_per_line) == (self.bytes_per_line / 2) - 1 {
                result.push(' ');
            }
        }

        // Add ASCII representation at end of each line
        if !data.is_empty() {
            result.push_str(" |");
            for (i, byte) in data.iter().enumerate() {
                if i % self.bytes_per_line == 0 {
                    if i > 0 {
                        result.push_str("\n |");
                    }
                }
                // Print printable ASCII characters, dot otherwise
                if byte.is_ascii_graphic() || *byte == b' ' {
                    result.push(*byte as char);
                } else {
                    result.push('.');
                }
            }
            result.push('|');
        }

        Ok(result)
    }

    /// Decode to binary format
    fn decode_binary(&self, data: &[u8]) -> Result<String> {
        let result: Vec<String> = data
            .iter()
            .map(|b| format!("{:08b}", b))
            .collect();
        Ok(result.join(" "))
    }

    /// Decode to octal format
    fn decode_octal(&self, data: &[u8]) -> Result<String> {
        let result: Vec<String> = data
            .iter()
            .map(|b| format!("{:03o}", b))
            .collect();
        Ok(result.join(" "))
    }

    /// Decode to decimal format (comma-separated)
    fn decode_decimal(&self, data: &[u8]) -> Result<String> {
        let result: Vec<String> = data
            .iter()
            .map(|b| b.to_string())
            .collect();
        Ok(result.join(", "))
    }
}

impl Default for ByteArrayDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_decode() {
        let decoder = ByteArrayDecoder::new();
        let data = &[0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"
        let result = decoder.decode(data).unwrap();
        assert!(result.contains("48 65 6c 6c 6f"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_binary_decode() {
        let decoder = ByteArrayDecoder::with_format(ByteArrayFormat::Binary);
        let data = &[0x01, 0x02, 0x03];
        let result = decoder.decode(data).unwrap();
        assert_eq!(result, "00000001 00000010 00000011");
    }
}

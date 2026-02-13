// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Numeric message decoders

use anyhow::Result;
use byteorder::{BigEndian, LittleEndian, ByteOrder, ReadBytesExt};
use std::io::Cursor;

/// Endianness for numeric types
#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}

/// Integer decoder (32-bit)
pub struct IntegerDecoder {
    endianness: Endianness,
}

impl IntegerDecoder {
    pub fn new() -> Self {
        Self {
            endianness: Endianness::BigEndian,
        }
    }

    pub fn with_endianness(endianness: Endianness) -> Self {
        Self { endianness }
    }

    pub fn decode(&self, data: &[u8]) -> Result<String> {
        if data.len() < 4 {
            return Ok(format!("<Insufficient data: {} bytes, need 4>", data.len()));
        }

        let value = match self.endianness {
            Endianness::BigEndian => i32::from_be_bytes([data[0], data[1], data[2], data[3]]),
            Endianness::LittleEndian => i32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        };

        Ok(value.to_string())
    }
}

impl Default for IntegerDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Long decoder (64-bit)
pub struct LongDecoder {
    endianness: Endianness,
}

impl LongDecoder {
    pub fn new() -> Self {
        Self {
            endianness: Endianness::BigEndian,
        }
    }

    pub fn with_endianness(endianness: Endianness) -> Self {
        Self { endianness }
    }

    pub fn decode(&self, data: &[u8]) -> Result<String> {
        if data.len() < 8 {
            return Ok(format!("<Insufficient data: {} bytes, need 8>", data.len()));
        }

        let value = match self.endianness {
            Endianness::BigEndian => {
                i64::from_be_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ])
            }
            Endianness::LittleEndian => {
                i64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ])
            }
        };

        Ok(value.to_string())
    }
}

impl Default for LongDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Short decoder (16-bit)
pub struct ShortDecoder {
    endianness: Endianness,
}

impl ShortDecoder {
    pub fn new() -> Self {
        Self {
            endianness: Endianness::BigEndian,
        }
    }

    pub fn with_endianness(endianness: Endianness) -> Self {
        Self { endianness }
    }

    pub fn decode(&self, data: &[u8]) -> Result<String> {
        if data.len() < 2 {
            return Ok(format!("<Insufficient data: {} bytes, need 2>", data.len()));
        }

        let value = match self.endianness {
            Endianness::BigEndian => i16::from_be_bytes([data[0], data[1]]),
            Endianness::LittleEndian => i16::from_le_bytes([data[0], data[1]]),
        };

        Ok(value.to_string())
    }
}

impl Default for ShortDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Float decoder (32-bit IEEE 754)
pub struct FloatDecoder {
    endianness: Endianness,
}

impl FloatDecoder {
    pub fn new() -> Self {
        Self {
            endianness: Endianness::BigEndian,
        }
    }

    pub fn with_endianness(endianness: Endianness) -> Self {
        Self { endianness }
    }

    pub fn decode(&self, data: &[u8]) -> Result<String> {
        if data.len() < 4 {
            return Ok(format!("<Insufficient data: {} bytes, need 4>", data.len()));
        }

        let bits = match self.endianness {
            Endianness::BigEndian => u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
            Endianness::LittleEndian => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        };

        let value = f32::from_bits(bits);
        Ok(value.to_string())
    }
}

impl Default for FloatDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Double decoder (64-bit IEEE 754)
pub struct DoubleDecoder {
    endianness: Endianness,
}

impl DoubleDecoder {
    pub fn new() -> Self {
        Self {
            endianness: Endianness::BigEndian,
        }
    }

    pub fn with_endianness(endianness: Endianness) -> Self {
        Self { endianness }
    }

    pub fn decode(&self, data: &[u8]) -> Result<String> {
        if data.len() < 8 {
            return Ok(format!("<Insufficient data: {} bytes, need 8>", data.len()));
        }

        let bits = match self.endianness {
            Endianness::BigEndian => {
                u64::from_be_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ])
            }
            Endianness::LittleEndian => {
                u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ])
            }
        };

        let value = f64::from_bits(bits);
        Ok(value.to_string())
    }
}

impl Default for DoubleDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_decode() {
        let decoder = IntegerDecoder::new();
        // 1000 in big-endian bytes
        let data = &[0x00, 0x00, 0x03, 0xE8];
        let result = decoder.decode(data).unwrap();
        assert_eq!(result, "1000");
    }

    #[test]
    fn test_float_decode() {
        let decoder = FloatDecoder::new();
        // 3.14 as IEEE 754 big-endian
        let data = &[0x40, 0x48, 0xF5, 0xC3];
        let result = decoder.decode(data).unwrap();
        assert!(result.starts_with("3.14"));
    }
}

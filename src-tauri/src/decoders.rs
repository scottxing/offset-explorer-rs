// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License

// Message decoders module

pub mod string_decoder;
pub mod byte_array_decoder;
pub mod avro_decoder;
pub mod no_key_decoder;
pub mod json_decoder;
pub mod numeric_decoders;
pub mod base64_decoder;

// Re-export decoders
pub use string_decoder::{StringDecoder, Encoding};
pub use byte_array_decoder::{ByteArrayDecoder, ByteArrayFormat};
pub use avro_decoder::AvroDecoder;
pub use no_key_decoder::NoKeyDecoder;
pub use json_decoder::JsonDecoder;
pub use numeric_decoders::{
    IntegerDecoder, LongDecoder, ShortDecoder,
    FloatDecoder, DoubleDecoder, Endianness
};

pub use base64_decoder::Base64Decoder;

/// Decoder trait for pluggable decoder system
pub trait Decoder {
    /// Decode raw bytes to a readable string
    fn decode(&self, data: &[u8]) -> anyhow::Result<String>;

    /// Get the name of this decoder
    fn name(&self) -> &str {
        "Unnamed"
    }
}

// Implement Decoder trait for existing decoders
impl Decoder for StringDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "String"
    }
}

impl Decoder for ByteArrayDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "ByteArray"
    }
}

impl Decoder for AvroDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "Avro"
    }
}

impl Decoder for NoKeyDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "NoKey"
    }
}

impl Decoder for JsonDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "JSON"
    }
}

impl Decoder for IntegerDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "Integer"
    }
}

impl Decoder for LongDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "Long"
    }
}

impl Decoder for ShortDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "Short"
    }
}

impl Decoder for FloatDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "Float"
    }
}

impl Decoder for DoubleDecoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "Double"
    }
}

impl Decoder for Base64Decoder {
    fn decode(&self, data: &[u8]) -> anyhow::Result<String> {
        self.decode(data)
    }

    fn name(&self) -> &str {
        "Base64"
    }
}

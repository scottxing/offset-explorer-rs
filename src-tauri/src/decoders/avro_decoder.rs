// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Avro message decoder with Schema Registry integration

use anyhow::{Result, anyhow};
use apache_avro::{from_avro_datum, types::Value};
use serde_json::Value as JsonValue;

/// Avro decoder configuration
pub struct AvroDecoder {
    /// Optional schema for decoding (in Avro JSON format)
    schema: Option<String>,
}

impl AvroDecoder {
    /// Create a new Avro decoder with optional schema
    pub fn new() -> Self {
        Self {
            schema: None,
        }
    }

    /// Create a new Avro decoder with a specific schema
    pub fn with_schema(schema: String) -> Self {
        Self {
            schema: Some(schema),
        }
    }

    /// Decode Avro binary data to JSON string
    pub fn decode(&self, data: &[u8]) -> Result<String> {
        if data.is_empty() {
            return Ok(String::new());
        }

        // Try to decode with schema if provided
        if let Some(ref schema_str) = self.schema {
            return self.decode_with_schema(data, schema_str);
        }

        // Fallback: Try to decode without schema (raw Avro)
        self.decode_raw(data)
    }

    /// Decode Avro data with a specific schema
    fn decode_with_schema(&self, data: &[u8], schema_str: &str) -> Result<String> {
        // Parse schema from JSON string
        let schema = apache_avro::Schema::parse_str(schema_str)
            .map_err(|e| anyhow!("Failed to parse Avro schema: {}", e))?;

        // Decode the Avro data
        let mut cursor = std::io::Cursor::new(data);
        let datum = from_avro_datum(&schema, &mut cursor, None)
            .map_err(|e| anyhow!("Failed to decode Avro datum: {}", e))?;

        // Convert to JSON for display
        let json_value = self.datum_to_json(&datum)?;
        serde_json::to_string_pretty(&json_value)
            .map_err(|e| anyhow!("Failed to serialize JSON: {}", e))
    }

    /// Decode raw Avro data without schema
    fn decode_raw(&self, data: &[u8]) -> Result<String> {
        // Try to decode as raw Avro (without schema)
        let mut cursor = std::io::Cursor::new(data);
        match from_avro_datum(&apache_avro::Schema::Long, &mut cursor, None) {
            Ok(datum) => {
                let json_value = self.datum_to_json(&datum)?;
                serde_json::to_string_pretty(&json_value)
                    .map_err(|e| anyhow!("Failed to serialize JSON: {}", e))
            }
            Err(e) => {
                // If raw decoding fails, return hex dump
                Ok(format!(
                    "{{ \"error\": \"Failed to decode Avro\", \"raw_bytes\": \"{}\", \"hex\": \"{}\" }}",
                    data.len(),
                    data.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
                ))
            }
        }
    }

    /// Convert Avro datum to JSON value
    fn datum_to_json(&self, datum: &Value) -> Result<JsonValue> {
        match datum {
            Value::Null => Ok(JsonValue::Null),
            Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
            Value::Int(i) => Ok(JsonValue::Number(serde_json::Number::from(*i))),
            Value::Long(i) => Ok(JsonValue::Number(serde_json::Number::from(*i))),
            Value::Float(f) => {
                // Handle special float values
                if f.is_finite() {
                    Ok(JsonValue::Number(serde_json::Number::from_f64(*f as f64).unwrap()))
                } else if f.is_nan() {
                    Ok(JsonValue::String("NaN".to_string()))
                } else {
                    Ok(JsonValue::String("Infinity".to_string()))
                }
            }
            Value::Double(d) => {
                if d.is_finite() {
                    Ok(JsonValue::Number(serde_json::Number::from_f64(*d).unwrap()))
                } else if d.is_nan() {
                    Ok(JsonValue::String("NaN".to_string()))
                } else {
                    Ok(JsonValue::String("Infinity".to_string()))
                }
            }
            Value::Bytes(bytes) => {
                // Encode bytes as base64
                use base64::Engine;
                Ok(JsonValue::String(base64::engine::general_purpose::STANDARD.encode(bytes)))
            }
            Value::String(s) => Ok(JsonValue::String(s.clone())),
            Value::Array(arr) => {
                let elements: Result<Vec<JsonValue>> = arr
                    .iter()
                    .map(|v| self.datum_to_json(v))
                    .collect();
                Ok(JsonValue::Array(elements?))
            }
            Value::Map(map) => {
                let object: Result<serde_json::Map<String, JsonValue>> = map
                    .iter()
                    .map(|(k, v)| {
                        self.datum_to_json(v).map(|jv| (k.clone(), jv))
                    })
                    .collect();
                Ok(JsonValue::Object(object?))
            }
            Value::Union(_, box_val) => self.datum_to_json(box_val),
            Value::Fixed(_, bytes) => {
                // Fixed size bytes - encode as base64
                use base64::Engine;
                Ok(JsonValue::String(base64::engine::general_purpose::STANDARD.encode(bytes)))
            }
            Value::Enum(_, index) => {
                Ok(JsonValue::Number(serde_json::Number::from(index.to_string().parse::<i64>().unwrap_or(0))))
            }
            _ => Ok(JsonValue::String(format!("{:?}", datum))),
        }
    }
}

impl Default for AvroDecoder {
    fn default() -> Self {
        Self::new()
    }
}

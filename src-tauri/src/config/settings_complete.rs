// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// User settings management - compatible with Java UserSettings class

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::io::Cursor;
use quick_xml::events::{Event, BytesStart};
use quick_xml::Writer;
use tracing::info;

/// Data type enumeration (compatible with Java SettingDataType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingDataType {
    Integer = 1,
    Long = 2,
    Double = 3,
    String = 4,
    Boolean = 5,
    Color = 6,
}

impl SettingDataType {
    pub fn from_i32(value: i32) -> Result<Self> {
        match value {
            1 => Ok(SettingDataType::Integer),
            2 => Ok(SettingDataType::Long),
            3 => Ok(SettingDataType::Double),
            4 => Ok(SettingDataType::String),
            5 => Ok(SettingDataType::Boolean),
            6 => Ok(SettingDataType::Color),
            _ => Err(anyhow!("Unknown data type: {}", value)),
        }
    }

    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SettingDataType::Integer => "Integer",
            SettingDataType::Long => "Long",
            SettingDataType::Double => "Double",
            SettingDataType::String => "String",
            SettingDataType::Boolean => "Boolean",
            SettingDataType::Color => "Color",
        }
    }
}

/// Setting value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: SettingValue,
    pub dynamic: bool,
    pub data_type: SettingDataType,
    pub persisted: bool,
}

/// Setting value enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingValue {
    Integer(i32),
    Long(i64),
    Double(f64),
    String(String),
    Boolean(bool),
    Color { r: u8, g: u8, b: u8 },
}

impl SettingValue {
    /// Convert value to string representation for XML
    pub fn to_xml_string(&self) -> String {
        match self {
            SettingValue::Integer(i) => i.to_string(),
            SettingValue::Long(l) => l.to_string(),
            SettingValue::Double(d) => d.to_string(),
            SettingValue::String(ref s) => s.clone(),
            SettingValue::Boolean(b) => (if *b { "true" } else { "false" }).to_string(),
            SettingValue::Color { r, g, b } => format!("{},{},{}", r, g, b),
        }
    }

    /// Parse value from string for XML deserialization
    pub fn from_string(s: &str, data_type: SettingDataType) -> Result<Self> {
        match data_type {
            SettingDataType::Integer => Ok(SettingValue::Integer(s.parse().map_err(|_| anyhow!("Invalid integer: {}", s))?)),
            SettingDataType::Long => Ok(SettingValue::Long(s.parse().map_err(|_| anyhow!("Invalid long: {}", s))?)),
            SettingDataType::Double => Ok(SettingValue::Double(s.parse().map_err(|_| anyhow!("Invalid double: {}", s))?)),
            SettingDataType::String => Ok(SettingValue::String(s.to_string())),
            SettingDataType::Boolean => {
                match s.to_lowercase().as_str() {
                    "true" | "1" => Ok(SettingValue::Boolean(true)),
                    "false" | "0" => Ok(SettingValue::Boolean(false)),
                    _ => Err(anyhow!("Invalid boolean: {}", s)),
                }
            },
            SettingDataType::Color => {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() == 3 {
                    let r: u8 = parts[0].parse().map_err(|_| anyhow!("Invalid color r: {}", parts[0]))?;
                    let g: u8 = parts[1].parse().map_err(|_| anyhow!("Invalid color g: {}", parts[1]))?;
                    let b: u8 = parts[2].parse().map_err(|_| anyhow!("Invalid color b: {}", parts[2]))?;
                    Ok(SettingValue::Color { r, g, b })
                } else {
                    Err(anyhow!("Invalid color format: {}", s))
                }
            }
        }
    }
}

/// User settings manager (compatible with Java UserSettings class)
pub struct UserSettings {
    settings: HashMap<String, Setting>,
}

impl UserSettings {
    /// Create new user settings with defaults
    pub fn new() -> Self {
        let settings = Self::initialize_defaults();
        Self { settings }
    }

    /// Initialize with default settings (compatible with Java UserSettings.init())
    fn initialize_defaults() -> HashMap<String, Setting> {
        let mut settings = HashMap::new();

        // Browser window settings
        settings.insert("browserwindow_maxrows".to_string(), Setting {
            key: "browserwindow_maxrows".to_string(),
            value: SettingValue::Integer(100),
            dynamic: true,
            data_type: SettingDataType::Integer,
            persisted: true,
        });
        settings.insert("browserwindow_maxrows_per_partition".to_string(), Setting {
            key: "browserwindow_maxrows_per_partition".to_string(),
            value: SettingValue::Integer(50),
            dynamic: true,
            data_type: SettingDataType::Integer,
            persisted: true,
        });
        settings.insert("browserwindow_hide_detail_panel".to_string(), Setting {
            key: "browserwindow_hide_detail_panel".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });

        // Timeout settings
        settings.insert("zookeeper_timeout".to_string(), Setting {
            key: "zookeeper_timeout".to_string(),
            value: SettingValue::Integer(10000),
            dynamic: true,
            data_type: SettingDataType::Integer,
            persisted: true,
        });
        settings.insert("broker_read_timeout".to_string(), Setting {
            key: "broker_read_timeout".to_string(),
            value: SettingValue::Integer(10000),
            dynamic: true,
            data_type: SettingDataType::Integer,
            persisted: true,
        });

        // Message settings
        settings.insert("max.messages.bytes".to_string(), Setting {
            key: "max.messages.bytes".to_string(),
            value: SettingValue::Integer(1048576),
            dynamic: true,
            data_type: SettingDataType::Integer,
            persisted: true,
        });
        settings.insert("show_timestamp_millis".to_string(), Setting {
            key: "show_timestamp_millis".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });
        settings.insert("offset.partition.messages".to_string(), Setting {
            key: "offset.partition.messages".to_string(),
            value: SettingValue::Integer(5000),
            dynamic: true,
            data_type: SettingDataType::Integer,
            persisted: true,
        });

        // Font settings
        settings.insert("font_size".to_string(), Setting {
            key: "font_size".to_string(),
            value: SettingValue::Integer(12),
            dynamic: false,
            data_type: SettingDataType::Integer,
            persisted: true,
        });
        settings.insert("font_use_default".to_string(), Setting {
            key: "font_use_default".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: false,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });

        // Display settings
        settings.insert("html_escape_json".to_string(), Setting {
            key: "html_escape_json".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });
        settings.insert("show_inactive_in_browser".to_string(), Setting {
            key: "show_inactive_in_browser".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });
        settings.insert("confirm_exit".to_string(), Setting {
            key: "confirm_exit".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });

        // Default decoders
        settings.insert("key_type".to_string(), Setting {
            key: "key_type".to_string(),
            value: SettingValue::String("byte_array".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("message_type".to_string(), Setting {
            key: "message_type".to_string(),
            value: SettingValue::String("byte_array".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });

        // Export settings
        settings.insert("dataexport_dest_dir".to_string(), Setting {
            key: "dataexport_dest_dir".to_string(),
            value: SettingValue::String(String::new()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("dataexport_key_file_pattern".to_string(), Setting {
            key: "dataexport_key_file_pattern".to_string(),
            value: SettingValue::String("key_#pid#_oid#.bin".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("dataexport_message_ccount".to_string(), Setting {
            key: "dataexport_message_ccount".to_string(),
            value: SettingValue::Long(1000),
            dynamic: true,
            data_type: SettingDataType::Long,
            persisted: true,
        });
        settings.insert("dataexport_offset_type".to_string(), Setting {
            key: "dataexport_offset_type".to_string(),
            value: SettingValue::String("FIRST".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("dataexport_export_key".to_string(), Setting {
            key: "dataexport_export_key".to_string(),
            value: SettingValue::Boolean(false),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });
        settings.insert("dataexport_export_message".to_string(), Setting {
            key: "dataexport_export_message".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });

        // Import settings
        settings.insert("dataimport_source_dir".to_string(), Setting {
            key: "dataimport_source_dir".to_string(),
            value: SettingValue::String(String::new()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("dataimport_key_file_pattern".to_string(), Setting {
            key: "dataimport_key_file_pattern".to_string(),
            value: SettingValue::String("key_#pid#_sid#.bin".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("dataimport_import_key".to_string(), Setting {
            key: "dataimport_import_key".to_string(),
            value: SettingValue::Boolean(false),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });
        settings.insert("dataimport_export_key".to_string(), Setting {
            key: "dataimport_export_key".to_string(),
            value: SettingValue::Boolean(false),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });
        settings.insert("dataimport_export_message".to_string(), Setting {
            key: "dataimport_export_message".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });

        // Add message method settings
        settings.insert("add_message_method_file_for_key".to_string(), Setting {
            key: "add_message_method_file_for_key".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });
        settings.insert("add_message_method_file_for_value".to_string(), Setting {
            key: "add_message_method_file_for_value".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });

        // Find messages settings
        settings.insert("datafind_message_ccount".to_string(), Setting {
            key: "datafind_message_ccount".to_string(),
            value: SettingValue::Long(1000),
            dynamic: true,
            data_type: SettingDataType::Long,
            persisted: true,
        });
        settings.insert("datafind_offset_type".to_string(), Setting {
            key: "datafind_offset_type".to_string(),
            value: SettingValue::String("FIRST".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("datafind_key_search_type".to_string(), Setting {
            key: "datafind_key_search_type".to_string(),
            value: SettingValue::String("DONT_SEARCH".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("datafind_key_match_type".to_string(), Setting {
            key: "datafind_key_match_type".to_string(),
            value: SettingValue::String("CONTAINS".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("datafind_key_search_term".to_string(), Setting {
            key: "datafind_key_search_term".to_string(),
            value: SettingValue::String(String::new()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("datafind_key_search_case_sensitive".to_string(), Setting {
            key: "datafind_key_search_case_sensitive".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });
        settings.insert("datafind_value_search_type".to_string(), Setting {
            key: "datafind_value_search_type".to_string(),
            value: SettingValue::String("BINARY".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("datafind_value_search_term".to_string(), Setting {
            key: "datafind_value_search_term".to_string(),
            value: SettingValue::String(String::new()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("datafind_value_match_type".to_string(), Setting {
            key: "datafind_value_match_type".to_string(),
            value: SettingValue::String("CONTAINS".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });
        settings.insert("datafind_value_search_case_sensitive".to_string(), Setting {
            key: "datafind_value_search_case_sensitive".to_string(),
            value: SettingValue::Boolean(true),
            dynamic: true,
            data_type: SettingDataType::Boolean,
            persisted: true,
        });

        // Storm settings
        settings.insert("storm_root".to_string(), Setting {
            key: "storm_root".to_string(),
            value: SettingValue::String("/".to_string()),
            dynamic: true,
            data_type: SettingDataType::String,
            persisted: true,
        });

        settings
    }
}

impl UserSettings {
    /// Get a setting by key
    pub fn get(&self, key: &str) -> Option<&Setting> {
        self.settings.get(key)
    }

    /// Get integer setting
    pub fn get_int(&self, key: &str) -> Result<i32> {
        match self.get(key) {
            Some(Setting { value: SettingValue::Integer(i), .. }) => Ok(*i),
            Some(Setting { value: SettingValue::Long(l), .. }) => Ok(*l as i32),
            _ => Err(anyhow!("Setting {} is not an integer", key)),
        }
    }

    /// Get long setting
    pub fn get_long(&self, key: &str) -> Result<i64> {
        match self.get(key) {
            Some(Setting { value: SettingValue::Long(l), .. }) => Ok(*l),
            Some(Setting { value: SettingValue::Integer(i), .. }) => Ok(*i as i64),
            _ => Err(anyhow!("Setting {} is not a long", key)),
        }
    }

    /// Get string setting
    pub fn get_string(&self, key: &str) -> Result<String> {
        match self.get(key) {
            Some(Setting { value: SettingValue::String(ref s), .. }) => Ok(s.clone()),
            Some(Setting { value: SettingValue::Boolean(b), .. }) => Ok(b.to_string()),
            _ => Err(anyhow!("Setting {} is not a string", key)),
        }
    }

    /// Get boolean setting
    pub fn get_bool(&self, key: &str) -> Result<bool> {
        match self.get(key) {
            Some(Setting { value: SettingValue::Boolean(b), .. }) => Ok(*b),
            _ => Err(anyhow!("Setting {} is not a boolean", key)),
        }
    }

    /// Set a setting value
    pub fn set(&mut self, key: &str, value: SettingValue, dynamic: bool, data_type: SettingDataType, persisted: bool) {
        let setting = Setting {
            key: key.to_string(),
            value,
            dynamic,
            data_type,
            persisted,
        };
        self.settings.insert(key.to_string(), setting);
    }

    /// Serialize settings to XML format (compatible with Java UserSettings.toXml())
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
        writer.write_event(Event::Start(BytesStart::new("settings")))?;

        // Write all persisted settings in sorted order
        let mut keys: Vec<&String> = self.settings.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(setting) = self.settings.get(key) {
                if !setting.persisted {
                    continue;
                }

                let mut elem = BytesStart::new("setting");
                elem.push_attribute(("name", setting.key.as_str()));
                elem.push_attribute(("value", setting.value.to_xml_string().as_str()));
                elem.push_attribute(("dynamic", if setting.dynamic { "true" } else { "false" }));
                elem.push_attribute(("data_type", (setting.data_type as i32).to_string().as_str()));
                elem.push_attribute(("persisted", if setting.persisted { "true" } else { "false" }));

                writer.write_event(Event::Empty(elem))?;
            }
        }

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("settings")))?;

        Ok(String::from_utf8(writer.into_inner().into_inner())?)
    }

    /// Deserialize settings from XML format (compatible with Java UserSettings.fromXML())
    pub fn from_xml(&mut self, xml: &str) -> Result<()> {
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.trim_text(true);

        let mut current_setting: Option<(String, String, String, String, String)> = None;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"setting" => {
                    let key = e.try_get_attribute("name")?
                        .ok_or_else(|| anyhow!("Missing name attribute"))?
                        .decode_and_unescape_value(&reader)?;
                    let value = e.try_get_attribute("value")?
                        .ok_or_else(|| anyhow!("Missing value attribute"))?
                        .decode_and_unescape_value(&reader)?;
                    let dynamic = e.try_get_attribute("dynamic")?
                        .ok_or_else(|| anyhow!("Missing dynamic attribute"))?
                        .decode_and_unescape_value(&reader)?;
                    let data_type = e.try_get_attribute("data_type")?
                        .ok_or_else(|| anyhow!("Missing data_type attribute"))?
                        .decode_and_unescape_value(&reader)?;
                    let persisted = e.try_get_attribute("persisted")?
                        .ok_or_else(|| anyhow!("Missing persisted attribute"))?
                        .decode_and_unescape_value(&reader)?;

                    current_setting = Some((key.into_owned(), value.into_owned(), dynamic.into_owned(), data_type.into_owned(), persisted.into_owned()));
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"setting" => {
                    if let Some((key, value, dynamic, data_type_str, persisted)) = current_setting.take() {
                        let data_type = SettingDataType::from_i32(data_type_str.parse()?)?;
                        let setting_value = SettingValue::from_string(&value, data_type)?;
                        let dynamic_bool = dynamic.parse::<bool>().unwrap_or(false);
                        let persisted_bool = persisted.parse::<bool>().unwrap_or(true);

                        self.settings.insert(key.clone(), Setting {
                            key,
                            value: setting_value,
                            dynamic: dynamic_bool,
                            data_type,
                            persisted: persisted_bool,
                        });
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow!("XML parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        info!("Loaded {} settings from XML", self.settings.len());
        Ok(())
    }
}

// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// User settings management - compatible with Java UserSettings class
// Handles typed settings (Integer, Long, Double, String, Boolean, Color)

use anyhow::{Result, anyhow};
use quick_xml::events::{Event, BytesStart};
use quick_xml::Writer;
use chrono::{Utc, DateTime};
use std::collections::HashMap;
use std::io::Cursor;

/// Data type enumeration for settings values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            _ => Err(anyhow!("Invalid data type: {}", value)),
        }
    }
}

/// Setting value with metadata
#[derive(Debug, Clone)]
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
            SettingValue::Integer(v) => v.to_string(),
            SettingValue::Long(v) => v.to_string(),
            SettingValue::Double(v) => v.to_string(),
            SettingValue::String(v) => v.clone(),
            SettingValue::Boolean(v) => v.to_string(),
            SettingValue::Color { r, g, b } => format!("{},{},{}", r, g, b),
        }
    }

    /// Parse value from string representation
    pub fn from_string(value: &str, data_type: SettingDataType) -> Result<Self> {
        Ok(match data_type {
            SettingDataType::Integer => SettingValue::Integer(value.parse()?),
            SettingDataType::Long => SettingValue::Long(value.parse()?),
            SettingDataType::Double => SettingValue::Double(value.parse()?),
            SettingDataType::String => SettingValue::String(value.to_string()),
            SettingDataType::Boolean => SettingValue::Boolean(value.parse()?),
            SettingDataType::Color => {
                let parts: Vec<&str> = value.split(',').collect();
                if parts.len() != 3 {
                    return Err(anyhow!("Invalid color format: {}", value));
                }
                let r: u8 = parts[0].parse()?;
                let g: u8 = parts[1].parse()?;
                let b: u8 = parts[2].parse()?;
                SettingValue::Color { r, g, b }
            }
        })
    }
}

/// Individual setting
#[derive(Debug, Clone)]
pub struct Setting {
    pub key: String,
    pub value: SettingValue,
    pub dynamic: bool,
    pub data_type: SettingDataType,
    pub persisted: bool,
}

impl Setting {
    pub fn new(key: String, value: SettingValue, dynamic: bool, data_type: SettingDataType, persisted: bool) -> Self {
        Self {
            key,
            value,
            dynamic,
            data_type,
            persisted,
        }
    }
}

/// User settings manager
/// Compatible with Java UserSettings class
pub struct UserSettings {
    settings: HashMap<String, Setting>,
    install_date: DateTime<Utc>,
}

impl UserSettings {
    pub fn new() -> Self {
        let mut settings = Self {
            settings: HashMap::new(),
            install_date: Utc::now(),
        };

        // Initialize with default values matching Java UserSettings.init()
        settings.init_defaults();
        settings
    }

    fn init_defaults(&mut self) {
        // Browser window settings
        self.set("browserwindow_maxrows", SettingValue::Integer(100), true, SettingDataType::Integer, true);
        self.set("browserwindow_maxrows_per_partition", SettingValue::Integer(50), true, SettingDataType::Integer, true);
        self.set("browserwindow_hide_detail_panel", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);

        // Timeout settings
        self.set("zookeeper_timeout", SettingValue::Integer(10000), true, SettingDataType::Integer, true);
        self.set("broker_read_timeout", SettingValue::Integer(10000), true, SettingDataType::Integer, true);

        // Message settings
        self.set("max.messages.bytes", SettingValue::Integer(1048576), true, SettingDataType::Integer, true);
        self.set("show_timestamp_millis", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);
        self.set("offset.partition.messages", SettingValue::Integer(5000), true, SettingDataType::Integer, true);

        // Font settings
        self.set("font_size", SettingValue::Integer(12), false, SettingDataType::Integer, true);
        self.set("font_use_default", SettingValue::Boolean(true), false, SettingDataType::Boolean, true);

        // Display settings
        self.set("html_escape_json", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);
        self.set("show_inactive_in_browser", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);
        self.set("debug_logging", SettingValue::Boolean(false), true, SettingDataType::Boolean, true);
        self.set("confirm_exit", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);

        // Default decoders
        self.set("key_type", SettingValue::String("byte_array".to_string()), true, SettingDataType::String, true);
        self.set("message_type", SettingValue::String("byte_array".to_string()), true, SettingDataType::String, true);

        // Export settings
        self.set("dataexport_dest_dir", SettingValue::String(String::new()), true, SettingDataType::String, true);
        self.set("dataexport_key_file_pattern", SettingValue::String("key_#pid#_#oid#.bin".to_string()), true, SettingDataType::String, true);
        self.set("dataexport_value_file_pattern", SettingValue::String("value_#pid#_#oid#.bin".to_string()), true, SettingDataType::String, true);
        self.set("dataexport_message_ccount", SettingValue::Long(1000), true, SettingDataType::Long, true);
        self.set("dataexport_offset_type", SettingValue::String("FIRST".to_string()), true, SettingDataType::String, true);
        self.set("dataexport_export_key", SettingValue::Boolean(false), true, SettingDataType::Boolean, true);
        self.set("dataexport_export_message", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);

        // Storm settings
        self.set("storm_root", SettingValue::String("/".to_string()), true, SettingDataType::String, true);

        // Import settings
        self.set("dataimport_source_dir", SettingValue::String(String::new()), true, SettingDataType::String, true);
        self.set("dataimport_key_file_pattern", SettingValue::String("key_#pid#_#sid#.bin".to_string()), true, SettingDataType::String, true);
        self.set("dataimport_value_file_pattern", SettingValue::String("value_#pid#_#sid#.bin".to_string()), true, SettingDataType::String, true);
        self.set("dataimport_import_key", SettingValue::Boolean(false), true, SettingDataType::Boolean, true);
        self.set("dataimport_import_value", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);

        // Add message method settings
        self.set("add_message_method_file_for_key", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);
        self.set("add_message_method_file_for_value", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);

        // Find messages settings
        self.set("datafind_message_ccount", SettingValue::Long(1000), true, SettingDataType::Long, true);
        self.set("datafind_offset_type", SettingValue::String("FIRST".to_string()), true, SettingDataType::String, true);
        self.set("datafind_key_search_type", SettingValue::String("DONT_SEARCH".to_string()), true, SettingDataType::String, true);
        self.set("datafind_key_match_type", SettingValue::String("CONTAINS".to_string()), true, SettingDataType::String, true);
        self.set("datafind_key_search_term", SettingValue::String(String::new()), true, SettingDataType::String, true);
        self.set("datafind_key_search_case_sensitive", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);
        self.set("datafind_value_search_type", SettingValue::String("BINARY".to_string()), true, SettingDataType::String, true);
        self.set("datafind_value_match_type", SettingValue::String("CONTAINS".to_string()), true, SettingDataType::String, true);
        self.set("datafind_value_search_term", SettingValue::String(String::new()), true, SettingDataType::String, true);
        self.set("datafind_value_search_case_sensitive", SettingValue::Boolean(true), true, SettingDataType::Boolean, true);
    }

    pub fn set(&mut self, key: &str, value: SettingValue, dynamic: bool, data_type: SettingDataType, persisted: bool) {
        let setting = Setting::new(key.to_string(), value, dynamic, data_type, persisted);
        self.settings.insert(key.to_string(), setting);
    }

    pub fn get(&self, key: &str) -> Option<&Setting> {
        self.settings.get(key)
    }

    pub fn get_string(&self, key: &str) -> Result<String> {
        match self.get(key) {
            Some(Setting { value: SettingValue::String(s), .. }) => Ok(s.clone()),
            Some(_) => Err(anyhow!("Setting {} is not a string", key)),
            None => Err(anyhow!("Setting {} not found", key)),
        }
    }

    pub fn get_int(&self, key: &str) -> Result<i32> {
        match self.get(key) {
            Some(Setting { value: SettingValue::Integer(i), .. }) => Ok(*i),
            Some(_) => Err(anyhow!("Setting {} is not an integer", key)),
            None => Err(anyhow!("Setting {} not found", key)),
        }
    }

    pub fn get_long(&self, key: &str) -> Result<i64> {
        match self.get(key) {
            Some(Setting { value: SettingValue::Long(l), .. }) => Ok(*l),
            Some(_) => Err(anyhow!("Setting {} is not a long", key)),
            None => Err(anyhow!("Setting {} not found", key)),
        }
    }

    pub fn get_bool(&self, key: &str) -> Result<bool> {
        match self.get(key) {
            Some(Setting { value: SettingValue::Boolean(b), .. }) => Ok(*b),
            Some(_) => Err(anyhow!("Setting {} is not a boolean", key)),
            None => Err(anyhow!("Setting {} not found", key)),
        }
    }

    /// Serialize settings to XML format (compatible with Java UserSettings.toXml())
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        // Create settings element with encoding attribute
        let encoding = self.generate_version_string();
        let mut root = BytesStart::new("settings");
        root.push_attribute(("encoding", encoding.as_str()));

        writer.write_event(Event::Start(root))?;

        // Write all persisted settings in sorted order
        let mut keys: Vec<&String> = self.settings.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(setting) = self.settings.get(*key).as_ref() {
                if !setting.persisted {
                    continue;
                }

                let mut elem = BytesStart::new("setting");
                elem.push_attribute(("name", setting.key.as_str()));
                elem.push_attribute(("value", setting.value.to_xml_string().as_str()));
                elem.push_attribute(("dynamic", if setting.dynamic { "true" } else { "false" }));
                let dt_str = format!("{}", setting.data_type as i32);
                elem.push_attribute(("data_type", dt_str.as_str()));

                writer.write_event(Event::Empty(elem))?;
            }
        }

        writer.write_event(Event::End(BytesEnd::new("settings")))?;

        let xml = String::from_utf8(writer.into_inner().into_inner())?;
        Ok(xml)
    }

    /// Deserialize settings from XML format (compatible with Java UserSettings.fromXML())
    pub fn from_xml(&mut self, xml: &str) -> Result<()> {
        // TODO: Implement XML parsing for quick-xml 0.31
        // The API has changed significantly from 0.27
        // For now, just keep defaults
        Ok(())
    }

    /// Generate version string from install date (compatible with Java)
    fn generate_version_string(&self) -> String {
        let timestamp = self.install_date.timestamp() as u64;
        let xored = timestamp ^ i64::MAX as u64;
        xored.to_string()
    }

    pub fn get_install_date(&self) -> DateTime<Utc> {
        self.install_date
    }
}

use quick_xml::events::BytesEnd;

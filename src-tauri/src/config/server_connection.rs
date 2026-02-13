// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Server connection configuration
// Compatible with Java ServerConnection and ServerConnectionSettings classes

use crate::config::{crypto, settings_complete::UserSettings};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use quick_xml::events::{Event, BytesStart};
use quick_xml::Writer;
use std::collections::HashMap;
use std::io::Cursor;

/// Broker security types (compatible with Java BrokerSecurityType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerSecurityType {
    PLAINTEXT,
    SSL,
    SASL_PLAINTEXT,
    SASL_SSL,
}

impl BrokerSecurityType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "PLAINTEXT" => Ok(BrokerSecurityType::PLAINTEXT),
            "SSL" => Ok(BrokerSecurityType::SSL),
            "SASL_PLAINTEXT" => Ok(BrokerSecurityType::SASL_PLAINTEXT),
            "SASL_SSL" => Ok(BrokerSecurityType::SASL_SSL),
            _ => Err(anyhow!("Unknown security type: {}", s)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BrokerSecurityType::PLAINTEXT => "PLAINTEXT",
            BrokerSecurityType::SSL => "SSL",
            BrokerSecurityType::SASL_PLAINTEXT => "SASL_PLAINTEXT",
            BrokerSecurityType::SASL_SSL => "SASL_SSL",
        }
    }

    pub fn is_sasl(&self) -> bool {
        matches!(self, BrokerSecurityType::SASL_PLAINTEXT | BrokerSecurityType::SASL_SSL)
    }

    pub fn is_ssl(&self) -> bool {
        matches!(self, BrokerSecurityType::SSL | BrokerSecurityType::SASL_SSL)
    }
}

/// Cluster version (compatible with Java ClusterVersion)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterVersion {
    VERSION_0_8_0,
    VERSION_0_8_1,
    VERSION_0_8_2,
    VERSION_0_8_2_0,
    VERSION_0_8_2_1,
    VERSION_0_8_2_2,
    VERSION_0_9,
    VERSION_0_10,
    VERSION_0_10_1,
    VERSION_0_10_2,
    VERSION_0_11,
    VERSION_1_0,
    VERSION_1_1,
    VERSION_2_0,
    VERSION_2_1,
    VERSION_2_2,
    VERSION_2_3,
    VERSION_2_4,
    VERSION_2_5,
    VERSION_2_6,
    VERSION_2_7,
    VERSION_2_8,
    VERSION_3_0,
    VERSION_3_1,
    VERSION_3_2,
    VERSION_3_3,
    VERSION_3_4,
    VERSION_3_5,
    VERSION_3_6,
    VERSION_3_7,
    LATEST,
}

impl ClusterVersion {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "1.0.0" | "VERSION_1_0_0" => Ok(ClusterVersion::VERSION_1_0),
            "LATEST" => Ok(ClusterVersion::LATEST),
            _ => Ok(ClusterVersion::LATEST), // Default to LATEST for unknown versions
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ClusterVersion::VERSION_1_0 => "VERSION_1_0_0",
            ClusterVersion::LATEST => "LATEST",
            _ => "VERSION_1_0_0",
        }
    }
}

/// Topic folder (for organizing topics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicFolder {
    pub name: String,
}

impl TopicFolder {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

/// Topic decoder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDecoderConfig {
    pub key_decoder: Option<String>,
    pub message_decoder: Option<String>,
    pub string_type: Option<String>,
    pub header_type: Option<String>,
    pub offset_request_type: Option<String>,
    pub parent_folder: Option<String>,
}

/// Server connection configuration
/// Compatible with Java ServerConnection class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConnection {
    pub id: i64,
    pub server_group_id: i64,
    pub name: String,
    pub host: String,
    pub port: Option<i64>,
    pub chroot: String,
    pub version: ClusterVersion,
    pub broker_security_type: BrokerSecurityType,
    pub bootstrap_servers: String,
    pub validate_ssl_endpoint_hostname: bool,
    pub enable_poller: bool,

    // SSL settings
    pub truststore_location: Option<String>,
    pub truststore_password: Option<String>,
    pub keystore_location: Option<String>,
    pub keystore_password: Option<String>,
    pub keystore_privatekey: Option<String>,

    // SASL settings
    pub sasl_mechanism: Option<String>,
    pub sasl_callback: Option<String>,
    pub sasl_endpoint_token: Option<String>,
    pub jaas_config: Option<String>,

    // Schema Registry settings
    pub schema_registry_endpoint: Option<String>,
    pub schema_registry_basic_auth: Option<String>,
    pub schema_registry_ssl_truststore_location: Option<String>,
    pub schema_registry_ssl_truststore_password: Option<String>,
    pub schema_registry_ssl_keystore_location: Option<String>,
    pub schema_registry_ssl_keystore_password: Option<String>,
    pub schema_registry_ssl_keystore_private_key: Option<String>,

    // Topic folders and decoder configs
    pub folders: Vec<TopicFolder>,
    pub topic_configs: HashMap<String, TopicDecoderConfig>,
}

impl ServerConnection {
    pub fn new(id: i64, name: String) -> Self {
        Self {
            id,
            server_group_id: 1,
            name,
            host: String::new(),
            port: None,
            chroot: "/".to_string(),
            version: ClusterVersion::VERSION_1_0,
            broker_security_type: BrokerSecurityType::PLAINTEXT,
            bootstrap_servers: String::new(),
            validate_ssl_endpoint_hostname: true,
            enable_poller: true,
            truststore_location: None,
            truststore_password: None,
            keystore_location: None,
            keystore_password: None,
            keystore_privatekey: None,
            sasl_mechanism: None,
            sasl_callback: None,
            sasl_endpoint_token: None,
            jaas_config: None,
            schema_registry_endpoint: None,
            schema_registry_basic_auth: None,
            schema_registry_ssl_truststore_location: None,
            schema_registry_ssl_truststore_password: None,
            schema_registry_ssl_keystore_location: None,
            schema_registry_ssl_keystore_password: None,
            schema_registry_ssl_keystore_private_key: None,
            folders: Vec::new(),
            topic_configs: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn set_host(&mut self, host: String) {
        self.host = host;
    }

    pub fn get_port(&self) -> Option<i64> {
        self.port
    }

    pub fn set_port(&mut self, port: Option<i64>) {
        self.port = port;
    }

    pub fn no_zookeeper_mode(&self) -> bool {
        self.host.is_empty()
    }

    pub fn get_bootstrap_servers(&self) -> String {
        if self.bootstrap_servers.is_empty() {
            if let Some(port) = self.port {
                format!("{}:{}", self.host, port)
            } else {
                self.host.clone()
            }
        } else {
            self.bootstrap_servers.clone()
        }
    }

    /// Add sasl_username and sasl_password fields for compatibility
    /// These map to sasl_callback and sasl_endpoint_token
    pub fn get_sasl_username(&self) -> Option<&String> {
        self.sasl_callback.as_ref()
    }

    pub fn get_sasl_password(&self) -> Option<&String> {
        self.sasl_endpoint_token.as_ref()
    }

    pub fn get_security_type(&self) -> &BrokerSecurityType {
        &self.broker_security_type
    }
}

/// Server connection settings manager
/// Compatible with Java ServerConnectionSettings class
pub struct ServerConnectionSettings {
    connections: Vec<ServerConnection>,
    next_id: i64,
}

impl ServerConnectionSettings {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            next_id: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn get_connections(&self) -> &[ServerConnection] {
        &self.connections
    }

    pub fn find_connection(&self, id: i64) -> Option<ServerConnection> {
        self.connections.iter().find(|c| c.id == id).cloned()
    }

    pub fn find_connection_by_name(&self, name: &str) -> Option<ServerConnection> {
        self.connections.iter().find(|c| c.name.eq_ignore_ascii_case(name)).cloned()
    }

    pub fn add_connection(&mut self, mut connection: ServerConnection) -> Result<()> {
        // Check for duplicate name
        if self.find_connection_by_name(&connection.name).is_some() {
            return Err(anyhow!("Server connection with name '{}' already exists", connection.name));
        }

        // Generate new ID if not set
        if connection.id == 0 {
            connection.id = self.generate_id();
        }

        self.connections.push(connection);
        Ok(())
    }

    pub fn remove_connection(&mut self, id: i64) -> Result<()> {
        let pos = self.connections.iter().position(|c| c.id == id)
            .ok_or_else(|| anyhow!("Connection not found: {}", id))?;
        self.connections.remove(pos);
        Ok(())
    }

    fn generate_id(&mut self) -> i64 {
        self.next_id += 1;
        self.next_id
    }

    /// Serialize to XML (compatible with Java ServerConnectionSettings.toXml())
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
        writer.write_event(Event::Start(BytesStart::new("connections")))?;

        for conn in &self.connections {
            self.write_connection(&mut writer, conn)?;
        }

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("connections")))?;

        let xml = String::from_utf8(writer.into_inner().into_inner())?;
        Ok(xml)
    }

    fn write_connection<W: std::io::Write>(&self, writer: &mut Writer<W>, conn: &ServerConnection) -> Result<()> {
        let mut elem = BytesStart::new("connection");
        elem.push_attribute(("name", conn.name.as_str()));
        elem.push_attribute(("host", conn.host.as_str()));
        elem.push_attribute(("chroot", conn.chroot.as_str()));
        elem.push_attribute(("port", conn.port.map(|p| p.to_string()).unwrap_or_default().as_str()));
        elem.push_attribute(("id", conn.id.to_string().as_str()));
        elem.push_attribute(("group", conn.server_group_id.to_string().as_str()));
        elem.push_attribute(("version", conn.version.as_str()));
        elem.push_attribute(("broker_security_type", conn.broker_security_type.as_str()));
        elem.push_attribute(("bootstrap_servers", conn.bootstrap_servers.as_str()));
        elem.push_attribute(("validate_ssl_endpoint_hostname", if conn.validate_ssl_endpoint_hostname { "true" } else { "false" }));
        elem.push_attribute(("enable_poller", if conn.enable_poller { "true" } else { "false" }));

        // SSL settings
        if let Some(ref loc) = conn.truststore_location {
            elem.push_attribute(("truststore_location", loc.as_str()));
        }
        if let Some(ref pwd) = conn.truststore_password {
            let encrypted = crypto::encrypt_password(pwd)?;
            elem.push_attribute(("truststore_password", encrypted.as_str()));
        }
        if let Some(ref loc) = conn.keystore_location {
            elem.push_attribute(("keystore_location", loc.as_str()));
        }
        if let Some(ref pwd) = conn.keystore_password {
            let encrypted = crypto::encrypt_password(pwd)?;
            elem.push_attribute(("keystore_password", encrypted.as_str()));
        }
        if let Some(ref key) = conn.keystore_privatekey {
            let encrypted = crypto::encrypt_password(key)?;
            elem.push_attribute(("keystore_privatekey", encrypted.as_str()));
        }

        // SASL settings
        if let Some(ref mech) = conn.sasl_mechanism {
            elem.push_attribute(("sasl_mechanism", mech.as_str()));
        }
        if let Some(ref cb) = conn.sasl_callback {
            elem.push_attribute(("sasl_callback", cb.as_str()));
        }
        if let Some(ref token) = conn.sasl_endpoint_token {
            elem.push_attribute(("sasl_endpoint_token", token.as_str()));
        }
        if let Some(ref jaas) = conn.jaas_config {
            elem.push_attribute(("jaas_config", jaas.as_str()));
        }

        // Schema Registry settings
        if let Some(ref endpoint) = conn.schema_registry_endpoint {
            elem.push_attribute(("schema_registry_endpoint", endpoint.as_str()));
        }
        if let Some(ref auth) = conn.schema_registry_basic_auth {
            elem.push_attribute(("schema_registry_basic_auth", auth.as_str()));
        }
        if let Some(ref loc) = conn.schema_registry_ssl_truststore_location {
            elem.push_attribute(("schema_registry_ssl_truststore_location", loc.as_str()));
        }
        if let Some(ref pwd) = conn.schema_registry_ssl_truststore_password {
            let encrypted = crypto::encrypt_password(pwd)?;
            elem.push_attribute(("schema_registry_ssl_truststore_password", encrypted.as_str()));
        }
        if let Some(ref loc) = conn.schema_registry_ssl_keystore_location {
            elem.push_attribute(("schema_registry_ssl_keystore_location", loc.as_str()));
        }
        if let Some(ref pwd) = conn.schema_registry_ssl_keystore_password {
            let encrypted = crypto::encrypt_password(pwd)?;
            elem.push_attribute(("schema_registry_ssl_keystore_password", encrypted.as_str()));
        }
        if let Some(ref key) = conn.schema_registry_ssl_keystore_private_key {
            let encrypted = crypto::encrypt_password(key)?;
            elem.push_attribute(("schema_registry_ssl_keystore_private_key", encrypted.as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        // Write folders
        if !conn.folders.is_empty() {
            writer.write_event(Event::Start(BytesStart::new("folders")))?;
            for folder in &conn.folders {
                let mut folder_elem = BytesStart::new("folder");
                folder_elem.push_attribute(("name", folder.name.as_str()));
                writer.write_event(Event::Empty(folder_elem))?;
            }
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("folders")))?;
        }

        // Write topic configs
        if !conn.topic_configs.is_empty() {
            writer.write_event(Event::Start(BytesStart::new("topics")))?;
            for (topic_name, config) in &conn.topic_configs {
                let mut topic_elem = BytesStart::new("topic");
                topic_elem.push_attribute(("name", topic_name.as_str()));
                if let Some(ref decoder) = config.key_decoder {
                    topic_elem.push_attribute(("key_decoder", decoder.as_str()));
                }
                if let Some(ref decoder) = config.message_decoder {
                    topic_elem.push_attribute(("message_decoder", decoder.as_str()));
                }
                if let Some(ref st) = config.string_type {
                    topic_elem.push_attribute(("string_type", st.as_str()));
                }
                if let Some(ref ht) = config.header_type {
                    topic_elem.push_attribute(("header_type", ht.as_str()));
                }
                if let Some(ref ort) = config.offset_request_type {
                    topic_elem.push_attribute(("offset_request_type", ort.as_str()));
                }
                if let Some(ref folder) = config.parent_folder {
                    topic_elem.push_attribute(("parent_folder", folder.as_str()));
                }
                writer.write_event(Event::Empty(topic_elem))?;
            }
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("topics")))?;
        }

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("connection")))?;
        Ok(())
    }

    /// Deserialize from XML (compatible with Java ServerConnectionSettings.fromXml())
    pub fn from_xml(&mut self, xml: &str, _settings: &UserSettings) -> Result<()> {
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.trim_text(true);

        let mut in_connections = false;
        let mut in_connection = false;
        let mut current_connection: Option<ServerConnection> = None;
        let mut in_folders = false;
        let mut in_topics = false;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"connections" => in_connections = true,
                        b"connection" if in_connections => {
                            in_connection = true;
                            current_connection = Some(self.parse_connection_start(e)?);
                        }
                        b"folders" if in_connection => in_folders = true,
                        b"topics" if in_connection => in_topics = true,
                        b"folder" if in_folders => {
                            if let Some(ref mut conn) = current_connection {
                                if let Some(name) = e.try_get_attribute("name")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
                                    conn.folders.push(TopicFolder::new(name.into_owned()));
                                }
                            }
                        }
                        b"topic" if in_topics => {
                            if let Some(ref mut conn) = current_connection {
                                if let Some(name) = e.try_get_attribute("name")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
                                    let topic_name = name.into_owned();
                                    let config = TopicDecoderConfig {
                                        key_decoder: e.try_get_attribute("key_decoder")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()).map(|s| s.into_owned()),
                                        message_decoder: e.try_get_attribute("message_decoder")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()).map(|s| s.into_owned()),
                                        string_type: e.try_get_attribute("string_type")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()).map(|s| s.into_owned()),
                                        header_type: e.try_get_attribute("header_type")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()).map(|s| s.into_owned()),
                                        offset_request_type: e.try_get_attribute("offset_request_type")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()).map(|s| s.into_owned()),
                                        parent_folder: e.try_get_attribute("parent_folder")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()).map(|s| s.into_owned()),
                                    };
                                    conn.topic_configs.insert(topic_name, config);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"connections" => in_connections = false,
                        b"connection" if in_connection => {
                            in_connection = false;
                            if let Some(conn) = current_connection.take() {
                                self.connections.push(conn);
                            }
                        }
                        b"folders" if in_connection => in_folders = false,
                        b"topics" if in_connection => in_topics = false,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow!("XML parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    fn parse_connection_start(&self, e: &BytesStart) -> Result<ServerConnection> {
        let mut conn = ServerConnection::new(0, String::new());

        // Create a dummy reader for attribute decoding
        use quick_xml::Reader;
        let reader = Reader::from_str("");

        // Parse required attributes
        if let Some(name) = e.try_get_attribute("name")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.name = name.into_owned();
        }
        if let Some(id) = e.try_get_attribute("id")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.id = id.parse().unwrap_or(0);
        }
        if let Some(group) = e.try_get_attribute("group")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.server_group_id = group.parse().unwrap_or(1);
        }

        if let Some(host) = e.try_get_attribute("host")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.host = host.into_owned();
        }
        if let Some(chroot) = e.try_get_attribute("chroot")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.chroot = chroot.into_owned();
        }
        if let Some(port) = e.try_get_attribute("port")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.port = port.parse().ok();
        }
        if let Some(version) = e.try_get_attribute("version")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.version = ClusterVersion::from_str(&version.into_owned())?;
        }
        if let Some(security) = e.try_get_attribute("broker_security_type")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.broker_security_type = BrokerSecurityType::from_str(&security.into_owned())?;
        }
        if let Some(bootstrap) = e.try_get_attribute("bootstrap_servers")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.bootstrap_servers = bootstrap.into_owned();
        }
        if let Some(validate) = e.try_get_attribute("validate_ssl_endpoint_hostname")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.validate_ssl_endpoint_hostname = validate.parse().unwrap_or(true);
        }
        if let Some(poller) = e.try_get_attribute("enable_poller")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.enable_poller = poller.parse().unwrap_or(true);
        }

        // Parse SSL settings
        if let Some(loc) = e.try_get_attribute("truststore_location")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.truststore_location = Some(loc.into_owned());
        }
        if let Some(pwd) = e.try_get_attribute("truststore_password")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.truststore_password = Some(crypto::decrypt_password(&pwd.into_owned())?);
        }
        if let Some(loc) = e.try_get_attribute("keystore_location")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.keystore_location = Some(loc.into_owned());
        }
        if let Some(pwd) = e.try_get_attribute("keystore_password")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.keystore_password = Some(crypto::decrypt_password(&pwd.into_owned())?);
        }
        if let Some(key) = e.try_get_attribute("keystore_privatekey")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.keystore_privatekey = Some(crypto::decrypt_password(&key.into_owned())?);
        }

        // Parse SASL settings
        if let Some(mech) = e.try_get_attribute("sasl_mechanism")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.sasl_mechanism = Some(mech.into_owned());
        }
        if let Some(cb) = e.try_get_attribute("sasl_callback")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.sasl_callback = Some(cb.into_owned());
        }
        if let Some(token) = e.try_get_attribute("sasl_endpoint_token")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.sasl_endpoint_token = Some(token.into_owned());
        }
        if let Some(jaas) = e.try_get_attribute("jaas_config")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.jaas_config = Some(jaas.into_owned());
        }

        // Parse Schema Registry settings
        if let Some(endpoint) = e.try_get_attribute("schema_registry_endpoint")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.schema_registry_endpoint = Some(endpoint.into_owned());
        }
        if let Some(auth) = e.try_get_attribute("schema_registry_basic_auth")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.schema_registry_basic_auth = Some(auth.into_owned());
        }
        if let Some(loc) = e.try_get_attribute("schema_registry_ssl_truststore_location")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.schema_registry_ssl_truststore_location = Some(loc.into_owned());
        }
        if let Some(pwd) = e.try_get_attribute("schema_registry_ssl_truststore_password")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.schema_registry_ssl_truststore_password = Some(crypto::decrypt_password(&pwd.into_owned())?);
        }
        if let Some(loc) = e.try_get_attribute("schema_registry_ssl_keystore_location")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.schema_registry_ssl_keystore_location = Some(loc.into_owned());
        }
        if let Some(pwd) = e.try_get_attribute("schema_registry_ssl_keystore_password")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.schema_registry_ssl_keystore_password = Some(crypto::decrypt_password(&pwd.into_owned())?);
        }
        if let Some(key) = e.try_get_attribute("schema_registry_ssl_keystore_private_key")?.and_then(|a| a.decode_and_unescape_value(&reader).ok()) {
            conn.schema_registry_ssl_keystore_private_key = Some(crypto::decrypt_password(&key.into_owned())?);
        }

        Ok(conn)
    }

    pub fn sort_by_name(&mut self) {
        self.connections.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }
}

// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Server connection configuration and management - Complete implementation

use crate::config::{crypto, settings_complete::UserSettings};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::io::Cursor;
use quick_xml::events::{Event, BytesStart};
use quick_xml::Writer;

/// Server connection - Complete implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConnection {
    pub id: i64,
    pub server_group_id: i64,
    pub name: String,
    pub host: String,
    pub port: Option<i64>,
    pub chroot: String,
    pub version: String,
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

    // Topic folders and decoder configs
    pub folders: Vec<super::TopicFolder>,
    pub topic_configs: HashMap<String, super::TopicDecoderConfig>,
}

impl ServerConnection {
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
}

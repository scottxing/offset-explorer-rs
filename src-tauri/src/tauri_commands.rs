// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Tauri command handlers
// Bridges Rust backend with frontend UI

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::State;
use tracing::{debug, info, warn};

use crate::acls::{AclBinding, AclFilter, CreateAclRequest};
use crate::async_ops::TaskManager;
use crate::config::ServerConnection;
use crate::kafka::mapper::{KafkaMapper, KafkaMessage};
use crate::schema_registry::client::{SchemaInfo as SchemaInfoInternal, SchemaRegistryClient, SchemaType};

// ==================== Application State ====================

/// Global application state
pub struct AppState {
    /// Connected Kafka mappers (server_id -> mapper)
    pub connections: Arc<Mutex<HashMap<i64, Arc<KafkaMapper>>>>,
    /// Saved server configurations
    pub server_configs: Arc<Mutex<Vec<ServerConnection>>>,
    /// Next server ID
    pub next_id: Arc<Mutex<i64>>,
    /// Task manager for background operations
    pub task_manager: Arc<TaskManager>,
    /// Flag indicating if shutdown is in progress
    pub is_shutting_down: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            server_configs: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(1)),
            task_manager: Arc::new(TaskManager::new()),
            is_shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn add_connection(&self, id: i64, mapper: Arc<KafkaMapper>) {
        let mut conns = self.connections.lock().unwrap();
        conns.insert(id, mapper);
        info!("Added connection ID {}", id);
    }

    pub fn remove_connection(&self, id: i64) {
        let mut conns = self.connections.lock().unwrap();
        conns.remove(&id);
        info!("Removed connection ID {}", id);
    }

    pub fn get_connection(&self, id: i64) -> Option<Arc<KafkaMapper>> {
        let conns = self.connections.lock().unwrap();
        conns.get(&id).cloned()
    }

    pub fn add_server_config(&self, config: ServerConnection) -> i64 {
        let id = {
            let mut next = self.next_id.lock().unwrap();
            let id = *next;
            *next += 1;
            id
        };
        let mut configs = self.server_configs.lock().unwrap();
        let mut config = config;
        config.id = id;
        configs.push(config);
        info!("Added server config ID {}", id);
        id
    }

    pub fn get_server_configs(&self) -> Vec<ServerConnection> {
        let configs = self.server_configs.lock().unwrap();
        configs.clone()
    }

    pub fn remove_server_config(&self, id: i64) {
        let mut configs = self.server_configs.lock().unwrap();
        configs.retain(|c| c.id != id);
        info!("Removed server config ID {}", id);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Type Definitions ====================

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConnectionRequest {
    pub name: String,
    #[serde(rename = "bootstrapServers")]
    pub bootstrap_servers: Option<String>,
    #[serde(rename = "securityType")]
    pub security_type: Option<String>,
    pub zookeeper_hosts: Option<String>,
    pub zookeeper_chroot: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTopicRequest {
    pub name: String,
    pub partitions: i32,
    #[serde(rename = "replicationFactor")]
    pub replication_factor: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProduceMessageRequest {
    pub topic: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct KafkaMessageResponse {
    pub topic: Option<String>,
    pub partition: i32,
    pub offset: i64,
    pub key: Option<String>,
    pub value: Option<String>,
    pub timestamp: i64,
}

impl From<KafkaMessage> for KafkaMessageResponse {
    fn from(msg: KafkaMessage) -> Self {
        Self {
            topic: msg.topic,
            partition: msg.partition,
            offset: msg.offset,
            key: msg.key.map(|k| String::from_utf8_lossy(&k).to_string()),
            value: msg.payload.map(|p| String::from_utf8_lossy(&p).to_string()),
            timestamp: msg.timestamp,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TopicMetadataResponse {
    pub name: String,
    #[serde(rename = "partitionCount")]
    pub partition_count: i32,
    #[serde(rename = "replicationFactor")]
    pub replication_factor: i32,
    pub internal: bool,
    pub partitions: Vec<PartitionInfo>,
}

#[derive(Debug, Serialize)]
pub struct PartitionInfo {
    pub id: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct ConsumerGroupResponse {
    #[serde(rename = "groupId")]
    pub group_id: String,
    pub state: String,
    #[serde(rename = "protocolType")]
    pub protocol_type: Option<String>,
    pub members: Vec<ConsumerMemberResponse>,
}

#[derive(Debug, Serialize)]
pub struct ConsumerMemberResponse {
    #[serde(rename = "memberId")]
    pub member_id: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "clientHost")]
    pub client_host: String,
}

#[derive(Debug, Serialize)]
pub struct SchemaInfo {
    pub subject: String,
    pub version: i32,
    pub id: i32,
    pub schema: String,
    #[serde(rename = "schemaType")]
    pub schema_type: String,
}

impl From<SchemaInfoInternal> for SchemaInfo {
    fn from(info: SchemaInfoInternal) -> Self {
        Self {
            subject: info.subject,
            version: info.version,
            id: info.id,
            schema: info.schema,
            schema_type: format!("{:?}", info.schema_type),
        }
    }
}

// ==================== Tauri Commands ====================

// --- Server Management ---

#[tauri::command]
pub fn get_server_connections(state: State<'_, Arc<AppState>>) -> Result<Vec<ServerConnection>, String> {
    info!("Getting all server connections");
    Ok(state.get_server_configs())
}

#[tauri::command]
pub fn add_server_connection(request: ServerConnectionRequest, state: State<'_, Arc<AppState>>) -> Result<i64, String> {
    info!("Adding server connection: {}", request.name);

    let mut config = ServerConnection::new(0, request.name.clone());
    config.bootstrap_servers = request.bootstrap_servers.unwrap_or_default();
    if !config.bootstrap_servers.is_empty() {
        config.host = config.bootstrap_servers.split(',').next().unwrap_or("").split(':').next().unwrap_or("").to_string();
    }

    let id = state.add_server_config(config);
    info!("Server added with ID: {}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_server_connection(id: i64, request: ServerConnectionRequest, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Updating server connection ID {}: {}", id, request.name);
    // TODO: Implement update
    Ok(())
}

#[tauri::command]
pub fn remove_server_connection(id: i64, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Removing server connection ID {}", id);
    state.remove_server_config(id);
    state.remove_connection(id);
    Ok(())
}

#[tauri::command]
pub fn connect_to_server(id: i64, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Connecting to server ID {}", id);

    if state.get_connection(id).is_some() {
        return Err(format!("Already connected to server ID {}", id));
    }

    let conn = ServerConnection::new(id, format!("Server-{}", id));
    let mapper = KafkaMapper::new(conn).map_err(|e| e.to_string())?;
    state.add_connection(id, Arc::new(mapper));

    info!("Successfully connected to server ID {}", id);
    Ok(())
}

#[tauri::command]
pub fn disconnect_from_server(id: i64, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Disconnecting from server ID {}", id);

    if let Some(mapper) = state.get_connection(id) {
        mapper.close().map_err(|e| e.to_string())?;
        state.remove_connection(id);
        info!("Successfully disconnected from server ID {}", id);
    } else {
        warn!("Server ID {} was not connected", id);
    }

    Ok(())
}

// --- Topic Management ---

#[tauri::command]
pub fn list_topics(server_id: i64, state: State<'_, Arc<AppState>>) -> Result<Vec<String>, String> {
    info!("Listing topics for server ID {}", server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    mapper.list_topics().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_topic(
    server_id: i64,
    request: CreateTopicRequest,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Creating topic '{}' on server ID {}", request.name, server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    mapper
        .create_topic(&request.name, request.partitions, request.replication_factor)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_topic(server_id: i64, topic_name: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Deleting topic '{}' on server ID {}", topic_name, server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    mapper.delete_topic(&topic_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_topic_metadata(
    server_id: i64,
    topic_name: String,
    state: State<'_, Arc<AppState>>,
) -> Result<TopicMetadataResponse, String> {
    info!("Getting metadata for topic '{}' on server ID {}", topic_name, server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    let metadata = mapper.get_topic_metadata(&topic_name).map_err(|e| e.to_string())?;

    Ok(TopicMetadataResponse {
        name: metadata.name,
        partition_count: metadata.partitions.len() as i32,
        replication_factor: if metadata.partitions.is_empty() {
            0
        } else {
            metadata.partitions[0].replicas.len() as i32
        },
        internal: metadata.internal,
        partitions: metadata
            .partitions
            .iter()
            .map(|p| PartitionInfo {
                id: p.id,
                leader: p.leader,
                replicas: p.replicas.clone(),
                isr: p.isr.clone(),
            })
            .collect(),
    })
}

#[tauri::command]
pub fn get_topic_partitions(
    server_id: i64,
    topic_name: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<PartitionInfo>, String> {
    info!("Getting partitions for topic '{}' on server ID {}", topic_name, server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    let metadata = mapper.get_topic_metadata(&topic_name).map_err(|e| e.to_string())?;

    Ok(metadata
        .partitions
        .iter()
        .map(|p| PartitionInfo {
            id: p.id,
            leader: p.leader,
            replicas: p.replicas.clone(),
            isr: p.isr.clone(),
        })
        .collect())
}

// --- Message Operations ---

#[tauri::command]
pub fn consume_messages(
    server_id: i64,
    topic: String,
    partition: Option<i32>,
    offset: Option<i64>,
    limit: usize,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<KafkaMessageResponse>, String> {
    info!(
        "Consuming messages from topic '{}' on server ID {} (limit: {})",
        topic, server_id, limit
    );

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    let consumer = mapper.create_consumer("offset-explorer-temp").map_err(|e| e.to_string())?;

    if let Some(p) = partition {
        let start_offset = offset.unwrap_or(0);
        consumer.assign(&topic, p, start_offset).map_err(|e| e.to_string())?;
    } else {
        consumer.subscribe(&[&topic]).map_err(|e| e.to_string())?;
    }

    let mut messages = Vec::new();
    for _ in 0..limit {
        match consumer.poll(1000).map_err(|e| e.to_string())? {
            Some(msg) => {
                messages.push(KafkaMessageResponse::from(msg));
            }
            None => break,
        }
    }

    info!("Consumed {} messages", messages.len());
    Ok(messages)
}

#[tauri::command]
pub fn produce_message(
    server_id: i64,
    request: ProduceMessageRequest,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!(
        "Producing message to topic '{}' on server ID {}",
        request.topic, server_id
    );

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    let key = request.key.map(|k| k.into_bytes());
    let value = request.value.map(|v| v.into_bytes());

    mapper
        .produce_message(&request.topic, key, value)
        .map_err(|e| e.to_string())
}

// --- Consumer Groups ---

#[tauri::command]
pub fn list_consumer_groups(
    server_id: i64,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ConsumerGroupResponse>, String> {
    info!("Listing consumer groups for server ID {}", server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    let groups = mapper.list_consumer_groups().map_err(|e| e.to_string())?;

    Ok(groups
        .iter()
        .map(|g| ConsumerGroupResponse {
            group_id: g.group_id.clone(),
            state: g.state.clone(),
            protocol_type: g.protocol_type.clone(),
            members: g
                .members
                .iter()
                .map(|m| ConsumerMemberResponse {
                    member_id: m.member_id.clone(),
                    client_id: m.client_id.clone(),
                    client_host: m.client_host.clone(),
                })
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub fn get_consumer_group_details(
    server_id: i64,
    group_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    info!(
        "Getting details for consumer group '{}' on server ID {}",
        group_id, server_id
    );

    let _mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    Ok(serde_json::json!({
        "groupId": group_id,
        "state": "Stable",
        "members": []
    }))
}

#[tauri::command]
pub fn reset_consumer_offset(
    server_id: i64,
    group_id: String,
    topic: String,
    partition: i32,
    offset: i64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!(
        "Resetting offset for group '{}' topic {} partition {} to {}",
        group_id, topic, partition, offset
    );

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    let consumer = mapper.create_consumer(&group_id).map_err(|e| e.to_string())?;
    consumer
        .assign(&topic, partition, offset)
        .map_err(|e| e.to_string())?;
    consumer.commit().map_err(|e| e.to_string())?;

    info!("Offset reset successfully");
    Ok(())
}

// --- Tasks ---

#[tauri::command]
pub fn get_task_progress(
    task_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<crate::async_ops::TaskProgress, String> {
    debug!("Getting progress for task: {}", task_id);

    let progress = state
        .task_manager
        .get_task_progress(&task_id)
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    Ok(progress)
}

#[tauri::command]
pub fn cancel_task(task_id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Cancelling task: {}", task_id);
    state.task_manager.cancel_task(&task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tasks() -> Result<Vec<String>, String> {
    debug!("Listing all tasks");
    Ok(Vec::new())
}

// --- Brokers ---

#[tauri::command]
pub fn list_brokers(server_id: i64, state: State<'_, Arc<AppState>>) -> Result<Vec<i32>, String> {
    info!("Listing brokers for server ID {}", server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    mapper.list_brokers().map_err(|e| e.to_string())
}

// --- ACLs ---

#[tauri::command]
pub fn list_acls(
    server_id: i64,
    filter: Option<AclFilter>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<AclBinding>, String> {
    info!("Listing ACLs for server ID {}", server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    mapper.list_acls(filter).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_acl(
    server_id: i64,
    request: CreateAclRequest,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Creating ACL on server ID {}", server_id);

    request.validate().map_err(|e| e.to_string())?;

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    mapper.create_acl(&request).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_acl(
    server_id: i64,
    principal: String,
    resource_type: String,
    resource_name: String,
    operation: String,
    permission_type: String,
    host: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Deleting ACL on server ID {}", server_id);

    let mapper = state
        .get_connection(server_id)
        .ok_or_else(|| format!("Not connected to server ID {}", server_id))?;

    mapper
        .delete_acl(
            &principal,
            &resource_type,
            &resource_name,
            &operation,
            &permission_type,
            &host,
        )
        .map_err(|e| e.to_string())
}

// --- Schema Registry ---

#[tauri::command]
pub fn list_schema_subjects(registry_url: String) -> Result<Vec<String>, String> {
    info!("Listing Schema Registry subjects");

    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    let client =
        SchemaRegistryClient::new(registry_url).map_err(|e| format!("Failed to create client: {}", e))?;

    rt.block_on(async { client.get_subjects().await })
        .map_err(|e| format!("Failed to list subjects: {}", e))
}

#[tauri::command]
pub fn get_schema(
    registry_url: String,
    subject: String,
    version: i32,
) -> Result<SchemaInfo, String> {
    info!("Getting schema {} version {}", subject, version);

    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    let client =
        SchemaRegistryClient::new(registry_url).map_err(|e| format!("Failed to create client: {}", e))?;

    rt.block_on(async { client.get_schema(&subject, version).await })
        .map(|info| info.into())
        .map_err(|e| format!("Failed to get schema: {}", e))
}

#[tauri::command]
pub fn get_latest_schema(registry_url: String, subject: String) -> Result<SchemaInfo, String> {
    info!("Getting latest schema for {}", subject);

    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    let client =
        SchemaRegistryClient::new(registry_url).map_err(|e| format!("Failed to create client: {}", e))?;

    rt.block_on(async { client.get_latest_schema(&subject).await })
        .map(|info| info.into())
        .map_err(|e| format!("Failed to get schema: {}", e))
}

#[tauri::command]
pub fn register_schema(
    registry_url: String,
    subject: String,
    schema: String,
    schema_type: String,
) -> Result<i32, String> {
    info!("Registering schema for {}", subject);

    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    let client =
        SchemaRegistryClient::new(registry_url).map_err(|e| format!("Failed to create client: {}", e))?;

    let schema_type_enum = match schema_type.as_str() {
        "AVRO" => SchemaType::AVRO,
        "PROTOBUF" => SchemaType::PROTOBUF,
        "JSON" => SchemaType::JSON,
        _ => return Err(format!("Invalid schema type: {}", schema_type)),
    };

    rt.block_on(async { client.register_schema(&subject, &schema, schema_type_enum).await })
        .map_err(|e| format!("Failed to register schema: {}", e))
}

#[tauri::command]
pub fn test_compatibility(
    registry_url: String,
    subject: String,
    schema: String,
    schema_type: String,
) -> Result<bool, String> {
    info!("Testing compatibility for {}", subject);

    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    let client =
        SchemaRegistryClient::new(registry_url).map_err(|e| format!("Failed to create client: {}", e))?;

    let schema_type_enum = match schema_type.as_str() {
        "AVRO" => SchemaType::AVRO,
        "PROTOBUF" => SchemaType::PROTOBUF,
        "JSON" => SchemaType::JSON,
        _ => return Err(format!("Invalid schema type: {}", schema_type)),
    };

    rt.block_on(async { client.check_compatibility(&subject, &schema, schema_type_enum).await })
        .map_err(|e| format!("Failed to check compatibility: {}", e))
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert!(state.get_connection(1).is_none());
    }
}

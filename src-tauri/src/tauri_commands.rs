// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Tauri command handlers
// Bridges Rust backend with frontend UI

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use tracing::{info, debug, warn};

use crate::config::{ServerConnection, ServerConnectionSettings};
use crate::kafka::mapper::{KafkaMapper, KafkaMessage};
use crate::async_ops::TaskManager;
use crate::acls::{AclBinding, CreateAclRequest, AclFilter};
use crate::schema_registry::client::{SchemaRegistryClient, SchemaInfo as SchemaInfoInternal, SchemaType};

/// Global application state
pub struct AppState {
    /// Connected Kafka mappers (server_id -> mapper)
    pub connections: Arc<Mutex<HashMap<i64, Arc<KafkaMapper>>>>,

    /// Task manager for background operations
    pub task_manager: Arc<TaskManager>,

    /// Flag indicating if shutdown is in progress
    pub is_shutting_down: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            task_manager: Arc::new(TaskManager::new()),
            is_shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Add a connection
    pub fn add_connection(&self, id: i64, mapper: Arc<KafkaMapper>) {
        let mut conns = self.connections.lock().unwrap();
        conns.insert(id, mapper);
        info!("Added connection ID {}", id);
    }

    /// Remove a connection
    pub fn remove_connection(&self, id: i64) {
        let mut conns = self.connections.lock().unwrap();
        conns.remove(&id);
        info!("Removed connection ID {}", id);
    }

    /// Get a connection
    pub fn get_connection(&self, id: i64) -> Option<Arc<KafkaMapper>> {
        let conns = self.connections.lock().unwrap();
        conns.get(&id).cloned()
    }
}

/// Server connection request
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

/// Topic creation request
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTopicRequest {
    pub name: String,
    pub partitions: i32,
    #[serde(rename = "replicationFactor")]
    pub replication_factor: i32,
}

/// Message production request
#[derive(Debug, Deserialize, Serialize)]
pub struct ProduceMessageRequest {
    pub topic: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

/// Message response
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

/// Topic metadata response
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

/// Partition information
#[derive(Debug, Serialize)]
pub struct PartitionInfo {
    pub id: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>,
}

/// Consumer group response
#[derive(Debug, Serialize)]
pub struct ConsumerGroupResponse {
    #[serde(rename = "groupId")]
    pub group_id: String,
    pub state: String,
    #[serde(rename = "protocolType")]
    pub protocol_type: Option<String>,
    pub members: Vec<ConsumerMemberResponse>,
}

/// Consumer member response
#[derive(Debug, Serialize)]
pub struct ConsumerMemberResponse {
    #[serde(rename = "memberId")]
    pub member_id: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "clientHost")]
    pub client_host: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(err: anyhow::Error) -> Self {
        Self {
            error: "Error".to_string(),
            message: err.to_string(),
        }
    }
}

/// Schema information for frontend
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

/// Tauri command handlers
pub struct TauriCommands {
    state: Arc<AppState>,
}

impl TauriCommands {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    // ==================== Server Management Commands ====================

    /// Get all server connections
    pub fn get_server_connections(&self) -> Result<Vec<ServerConnectionSettings>> {
        info!("Getting all server connections");
        // TODO: Load from configuration
        Ok(Vec::new())
    }

    /// Add a new server connection
    pub fn add_server_connection(&self, request: ServerConnectionRequest) -> Result<i64> {
        info!("Adding server connection: {}", request.name);

        // Create ServerConnection from request
        let conn = ServerConnection::new(1, request.name.clone());
        // TODO: Save to configuration
        // TODO: Create KafkaMapper and add to state

        Ok(1)
    }

    /// Update an existing server connection
    pub fn update_server_connection(&self, id: i64, request: ServerConnectionRequest) -> Result<()> {
        info!("Updating server connection ID {}: {}", id, request.name);
        // TODO: Update in configuration
        // TODO: Recreate KafkaMapper if connected
        Ok(())
    }

    /// Remove a server connection
    pub fn remove_server_connection(&self, id: i64) -> Result<()> {
        info!("Removing server connection ID {}", id);

        // Disconnect if connected
        if let Some(_mapper) = self.state.get_connection(id) {
            self.disconnect_from_server(id)?;
        }

        self.state.remove_connection(id);
        // TODO: Remove from configuration
        Ok(())
    }

    /// Connect to a Kafka server
    pub fn connect_to_server(&self, id: i64) -> Result<()> {
        info!("Connecting to server ID {}", id);

        // Check if already connected
        if self.state.get_connection(id).is_some() {
            return Err(anyhow!("Already connected to server ID {}", id));
        }

        // TODO: Load ServerConnection from configuration
        let conn = ServerConnection::new(id, format!("Server-{}", id));

        // Create Kafka mapper
        let mapper = Arc::new(KafkaMapper::new(conn)?);
        self.state.add_connection(id, mapper);

        info!("Successfully connected to server ID {}", id);
        Ok(())
    }

    /// Disconnect from a Kafka server
    pub fn disconnect_from_server(&self, id: i64) -> Result<()> {
        info!("Disconnecting from server ID {}", id);

        if let Some(mapper) = self.state.get_connection(id) {
            mapper.close()?;
            self.state.remove_connection(id);
            info!("Successfully disconnected from server ID {}", id);
        } else {
            warn!("Server ID {} was not connected", id);
        }

        Ok(())
    }

    // ==================== Topic Management Commands ====================

    /// List topics for a server
    pub fn list_topics(&self, server_id: i64) -> Result<Vec<String>> {
        info!("Listing topics for server ID {}", server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        mapper.list_topics()
    }

    /// Create a new topic
    pub fn create_topic(&self, server_id: i64, request: CreateTopicRequest) -> Result<()> {
        info!("Creating topic '{}' on server ID {}", request.name, server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        mapper.create_topic(&request.name, request.partitions, request.replication_factor)
    }

    /// Delete a topic
    pub fn delete_topic(&self, server_id: i64, topic_name: String) -> Result<()> {
        info!("Deleting topic '{}' on server ID {}", topic_name, server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        mapper.delete_topic(&topic_name)
    }

    /// Get topic metadata
    pub fn get_topic_metadata(&self, server_id: i64, topic_name: String) -> Result<TopicMetadataResponse> {
        info!("Getting metadata for topic '{}' on server ID {}", topic_name, server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        let metadata = mapper.get_topic_metadata(&topic_name)?;

        Ok(TopicMetadataResponse {
            name: metadata.name,
            partition_count: metadata.partitions.len() as i32,
            replication_factor: if metadata.partitions.is_empty() { 0 } else { metadata.partitions[0].replicas.len() as i32 },
            internal: metadata.internal,
            partitions: metadata.partitions.iter().map(|p| PartitionInfo {
                id: p.id,
                leader: p.leader,
                replicas: p.replicas.clone(),
                isr: p.isr.clone(),
            }).collect(),
        })
    }

    /// Get topic partitions
    pub fn get_topic_partitions(&self, server_id: i64, topic_name: String) -> Result<Vec<PartitionInfo>> {
        info!("Getting partitions for topic '{}' on server ID {}", topic_name, server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        let metadata = mapper.get_topic_metadata(&topic_name)?;

        Ok(metadata.partitions.iter().map(|p| PartitionInfo {
            id: p.id,
            leader: p.leader,
            replicas: p.replicas.clone(),
            isr: p.isr.clone(),
        }).collect())
    }

    // ==================== Message Operation Commands ====================

    /// Consume messages from a topic
    pub fn consume_messages(
        &self,
        server_id: i64,
        topic: &str,
        partition: Option<i32>,
        offset: Option<i64>,
        limit: usize,
    ) -> Result<Vec<KafkaMessageResponse>> {
        info!("Consuming messages from topic '{}' on server ID {} (limit: {})",
              topic, server_id, limit);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        let consumer = mapper.create_consumer("offset-explorer-temp")?;

        // Assign partition if specified
        if let Some(p) = partition {
            let start_offset = offset.unwrap_or(0);
            consumer.assign(topic, p, start_offset)?;
        } else {
            consumer.subscribe(&[topic])?;
        }

        let mut messages = Vec::new();
        for _ in 0..limit {
            match consumer.poll(1000)? {
                Some(msg) => {
                    messages.push(KafkaMessageResponse::from(msg));
                }
                None => break,
            }
        }

        info!("Consumed {} messages", messages.len());
        Ok(messages)
    }

    /// Produce a message
    pub fn produce_message(&self, server_id: i64, request: ProduceMessageRequest) -> Result<()> {
        info!("Producing message to topic '{}' on server ID {}", request.topic, server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        let key = request.key.map(|k| k.into_bytes());
        let value = request.value.map(|v| v.into_bytes());

        mapper.produce_message(&request.topic, key, value)
    }

    // ==================== Consumer Group Commands ====================

    /// List consumer groups
    pub fn list_consumer_groups(&self, server_id: i64) -> Result<Vec<ConsumerGroupResponse>> {
        info!("Listing consumer groups for server ID {}", server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        let groups = mapper.list_consumer_groups()?;

        Ok(groups.iter().map(|g| ConsumerGroupResponse {
            group_id: g.group_id.clone(),
            state: g.state.clone(),
            protocol_type: g.protocol_type.clone(),
            members: g.members.iter().map(|m| ConsumerMemberResponse {
                member_id: m.member_id.clone(),
                client_id: m.client_id.clone(),
                client_host: m.client_host.clone(),
            }).collect(),
        }).collect())
    }

    /// Get consumer group details
    pub fn get_consumer_group_details(&self, server_id: i64, group_id: String) -> Result<serde_json::Value> {
        info!("Getting details for consumer group '{}' on server ID {}", group_id, server_id);

        // Let underscore compiler know we intentionally ignore this
        let _mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        // TODO: Implement detailed consumer group info
        // This would require getting consumer offsets, lag, etc.
        Ok(serde_json::json!({
            "groupId": group_id,
            "state": "Stable",
            "members": []
        }))
    }

    /// Reset consumer offset
    pub fn reset_consumer_offset(
        &self,
        server_id: i64,
        group_id: String,
        topic: &str,
        partition: i32,
        offset: i64,
    ) -> Result<()> {
        info!("Resetting offset for group '{}' topic {} partition {} to {} on server ID {}",
              group_id, topic, partition, offset, server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        let consumer = mapper.create_consumer(&group_id)?;
        consumer.assign(topic, partition, offset)?;
        consumer.commit()?;

        info!("Offset reset successfully");
        Ok(())
    }

    // ==================== Background Task Commands ====================

    /// Get task progress
    pub fn get_task_progress(&self, task_id: String) -> Result<crate::async_ops::TaskProgress> {
        debug!("Getting progress for task: {}", task_id);

        let progress = self.state.task_manager.get_task_progress(&task_id)
            .ok_or_else(|| anyhow!("Task {} not found", task_id))?;

        Ok(progress)
    }

    /// Cancel a task
    pub fn cancel_task(&self, task_id: String) -> Result<()> {
        info!("Cancelling task: {}", task_id);
        self.state.task_manager.cancel_task(&task_id)
    }

    /// List all tasks
    pub fn list_tasks(&self) -> Result<Vec<String>> {
        debug!("Listing all tasks");
        // TODO: Implement task listing
        Ok(Vec::new())
    }

    // ==================== Broker Commands ====================

    /// List brokers
    pub fn list_brokers(&self, server_id: i64) -> Result<Vec<i32>> {
        info!("Listing brokers for server ID {}", server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        mapper.list_brokers()
    }

    // ==================== ACL Management Commands ====================

    /// List ACL bindings
    pub fn list_acls(&self, server_id: i64, filter: Option<AclFilter>) -> Result<Vec<AclBinding>> {
        info!("Listing ACLs for server ID {}", server_id);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        mapper.list_acls(filter)
    }

    /// Create a new ACL
    pub fn create_acl(&self, server_id: i64, request: CreateAclRequest) -> Result<()> {
        info!("Creating ACL on server ID {}: {:?}", server_id, request);

        // Validate request
        request.validate()?;

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        mapper.create_acl(&request)
    }

    /// Delete an ACL
    pub fn delete_acl(
        &self,
        server_id: i64,
        principal: String,
        resource_type: String,
        resource_name: String,
        operation: String,
        permission_type: String,
        host: String,
    ) -> Result<()> {
        info!("Deleting ACL on server ID {}: {}={} on {}",
              server_id, principal, operation, resource_name);

        let mapper = self.state.get_connection(server_id)
            .ok_or_else(|| anyhow!("Not connected to server ID {}", server_id))?;

        mapper.delete_acl(&principal, &resource_type, &resource_name, &operation, &permission_type, &host)
    }

    // ==================== Schema Registry Commands ====================

    /// List all subjects in Schema Registry
    pub fn list_schema_subjects(&self, registry_url: String) -> Result<Vec<String>> {
        info!("Listing Schema Registry subjects");

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create runtime: {}", e))?;

        let client = SchemaRegistryClient::new(registry_url)
            .map_err(|e| anyhow!("Failed to create client: {}", e))?;
        let subjects = rt.block_on(async {
            client.get_subjects().await
        }).map_err(|e| anyhow!("Failed to list subjects: {}", e))?;

        Ok(subjects)
    }

    /// Get a specific schema from Schema Registry
    pub fn get_schema(&self, registry_url: String, subject: String, version: i32) -> Result<SchemaInfo> {
        info!("Getting schema {} version {} from {}", subject, version, registry_url);

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create runtime: {}", e))?;

        let client = SchemaRegistryClient::new(registry_url)
            .map_err(|e| anyhow!("Failed to create client: {}", e))?;
        let schema_info = rt.block_on(async {
            client.get_schema(&subject, version).await
        }).map_err(|e| anyhow!("Failed to get schema: {}", e))?;

        Ok(schema_info.into())
    }

    /// Get the latest schema for a subject
    pub fn get_latest_schema(&self, registry_url: String, subject: String) -> Result<SchemaInfo> {
        info!("Getting latest schema for {} from {}", subject, registry_url);

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create runtime: {}", e))?;

        let client = SchemaRegistryClient::new(registry_url)
            .map_err(|e| anyhow!("Failed to create client: {}", e))?;
        let schema_info = rt.block_on(async {
            client.get_latest_schema(&subject).await
        }).map_err(|e| anyhow!("Failed to get latest schema: {}", e))?;

        Ok(schema_info.into())
    }

    /// Register a new schema in Schema Registry
    pub fn register_schema(&self, registry_url: String, subject: String, schema: String, schema_type: String) -> Result<i32> {
        info!("Registering new schema for {} in {}", subject, registry_url);

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create runtime: {}", e))?;

        let client = SchemaRegistryClient::new(registry_url)
            .map_err(|e| anyhow!("Failed to create client: {}", e))?;
        let schema_type_enum = match schema_type.as_str() {
            "AVRO" => SchemaType::AVRO,
            "PROTOBUF" => SchemaType::PROTOBUF,
            "JSON" => SchemaType::JSON,
            _ => return Err(anyhow!("Invalid schema type: {}", schema_type)),
        };

        let id = rt.block_on(async {
            client.register_schema(&subject, &schema, schema_type_enum).await
        }).map_err(|e| anyhow!("Failed to register schema: {}", e))?;

        info!("Registered schema with ID: {}", id);
        Ok(id)
    }

    /// Test schema compatibility
    pub fn test_compatibility(&self, registry_url: String, subject: String, schema: String, schema_type: String) -> Result<bool> {
        info!("Testing compatibility for {} in {}", subject, registry_url);

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create runtime: {}", e))?;

        let client = SchemaRegistryClient::new(registry_url)
            .map_err(|e| anyhow!("Failed to create client: {}", e))?;
        let schema_type_enum = match schema_type.as_str() {
            "AVRO" => SchemaType::AVRO,
            "PROTOBUF" => SchemaType::PROTOBUF,
            "JSON" => SchemaType::JSON,
            _ => return Err(anyhow!("Invalid schema type: {}", schema_type)),
        };

        let is_compatible = rt.block_on(async {
            client.check_compatibility(&subject, &schema, schema_type_enum).await
        }).map_err(|e| anyhow!("Failed to check compatibility: {}", e))?;

        Ok(is_compatible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_conversion() {
        let err = anyhow!("Test error");
        let response: ErrorResponse = err.into();
        assert_eq!(response.error, "Error");
    }

    #[test]
    fn test_kafka_message_conversion() {
        let kafka_msg = KafkaMessage {
            topic: Some("test-topic".to_string()),
            partition: 0,
            offset: 123,
            key: Some(b"key".to_vec()),
            payload: Some(b"value".to_vec()),
            timestamp: 123456789,
        };

        let response: KafkaMessageResponse = kafka_msg.into();
        assert_eq!(response.topic, Some("test-topic".to_string()));
        assert_eq!(response.partition, 0);
        assert_eq!(response.offset, 123);
    }
}

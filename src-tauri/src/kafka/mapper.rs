// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Kafka operations wrapper around rdkafka
// Basic implementation for PLAINTEXT connections

use anyhow::{Result, anyhow};
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::Message;
use rdkafka::metadata::Metadata;
use rdkafka::producer::{BaseProducer, BaseRecord};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::error::KafkaError;
use rdkafka::Offset;
use std::time::Duration;
use tracing::{info, debug, warn, error};
use crate::config::ServerConnection;

/// Kafka mapper - wrapper around rdkafka for Kafka operations
/// Compatible with Java KafkaMapper class
pub struct KafkaMapper {
    connection: ServerConnection,
    admin: Option<AdminClient<rdkafka::client::DefaultClientContext>>,
    producer: Option<BaseProducer<rdkafka::producer::DefaultProducerContext>>,
}

impl KafkaMapper {
    /// Create a new Kafka mapper from server connection
    pub fn new(connection: ServerConnection) -> Result<Self> {
        info!("Creating Kafka mapper for: {}", connection.get_name());

        // Build bootstrap servers
        let bootstrap_servers = connection.get_bootstrap_servers();

        // Check if security is enabled
        let security_type = connection.get_security_type();
        if security_type != &crate::config::BrokerSecurityType::PLAINTEXT {
            warn!("Security type {:?} not yet fully supported, using basic configuration",
                  security_type);
            // TODO: Implement full SSL/SASL support when system libraries are available
        }

        // Create admin client
        let admin = Self::create_admin_client(&bootstrap_servers, &connection)?;

        // Create producer
        let producer = Self::create_producer(&bootstrap_servers, &connection)?;

        Ok(Self {
            connection,
            admin: Some(admin),
            producer: Some(producer),
        })
    }

    /// Create admin client
    fn create_admin_client(
        bootstrap_servers: &str,
        connection: &ServerConnection
    ) -> Result<AdminClient<rdkafka::client::DefaultClientContext>> {
        let mut config = ClientConfig::new();
        config.set("bootstrap.servers", bootstrap_servers);

        // Add security configuration based on connection type
        match connection.get_security_type() {
            crate::config::BrokerSecurityType::PLAINTEXT => {
                // No additional config needed
            }
            crate::config::BrokerSecurityType::SSL => {
                // TODO: Add SSL configuration when system libraries available
                warn!("SSL configuration not yet implemented");
            }
            crate::config::BrokerSecurityType::SASL_PLAINTEXT => {
                // TODO: Add SASL configuration when system libraries available
                warn!("SASL_PLAINTEXT configuration not yet implemented");
            }
            crate::config::BrokerSecurityType::SASL_SSL => {
                // TODO: Add SASL_SSL configuration when system libraries available
                warn!("SASL_SSL configuration not yet implemented");
            }
        }

        // Set client ID
        config.set("client.id", format!("offset-explorer-rust-{}", connection.get_name()));

        let admin: AdminClient<_> = config
            .create()
            .map_err(|e| anyhow!("Failed to create admin client: {}", e))?;

        Ok(admin)
    }

    /// Create producer
    fn create_producer(
        bootstrap_servers: &str,
        connection: &ServerConnection
    ) -> Result<BaseProducer<rdkafka::producer::DefaultProducerContext>> {
        let mut config = ClientConfig::new();
        config.set("bootstrap.servers", bootstrap_servers);

        // Add security configuration
        match connection.get_security_type() {
            crate::config::BrokerSecurityType::PLAINTEXT => {
                // No additional config needed
            }
            _ => {
                warn!("Security type {:?} not yet supported for producer",
                      connection.get_security_type());
            }
        }

        config.set("client.id", format!("offset-explorer-rust-producer-{}", connection.get_name()));

        let producer: BaseProducer<_> = config
            .create()
            .map_err(|e| anyhow!("Failed to create producer: {}", e))?;

        Ok(producer)
    }

    /// List all topics
    pub fn list_topics(&self) -> Result<Vec<String>> {
        info!("Listing topics for: {}", self.connection.get_name());

        let admin = self.admin.as_ref()
            .ok_or_else(|| anyhow!("Admin client not initialized"))?;

        // Get metadata with timeout
        let metadata = admin.inner()
            .fetch_metadata(None, Duration::from_secs(10))
            .map_err(|e| anyhow!("Failed to fetch metadata: {}", e))?;

        // Extract topic names
        let topics: Vec<String> = metadata
            .topics()
            .iter()
            .map(|t| t.name().to_string())
            .collect();

        info!("Found {} topics", topics.len());
        Ok(topics)
    }

    /// Create a new topic
    pub fn create_topic(&self, name: &str, partitions: i32, replication_factor: i32) -> Result<()> {
        info!("Creating topic: {} with {} partitions and replication factor {}",
              name, partitions, replication_factor);

        let admin = self.admin.as_ref()
            .ok_or_else(|| anyhow!("Admin client not initialized"))?;

        // Validate inputs
        if name.is_empty() {
            return Err(anyhow!("Topic name cannot be empty"));
        }
        if partitions <= 0 {
            return Err(anyhow!("Partitions must be positive"));
        }
        if replication_factor <= 0 {
            return Err(anyhow!("Replication factor must be positive"));
        }

        // Create new topic
        let new_topic = NewTopic::new(
            name,
            partitions,
            TopicReplication::Fixed(replication_factor),
        );

        // Create admin options
        let admin_opts = AdminOptions::new()
            .request_timeout(Some(Duration::from_secs(30)))
            .operation_timeout(Some(Duration::from_secs(30)));

        // Use tokio runtime for async admin operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create runtime: {}", e))?;

        // Execute create topic
        let results = rt.block_on(async {
            admin.create_topics([&new_topic], &admin_opts).await
        }).map_err(|e| anyhow!("Failed to create topic: {}", e))?;

        // Wait for results
        for result in results {
            match result {
                Ok(_) => {
                    info!("Topic {} created successfully", name);
                }
                Err((topic, error)) => {
                    // Check if topic already exists
                    if error.to_string().contains("already exists") {
                        warn!("Topic {} already exists", topic);
                        return Err(anyhow!("Topic '{}' already exists", topic));
                    }
                    error!("Failed to create topic {}: {}", topic, error);
                    return Err(anyhow!("Failed to create topic '{}': {}", topic, error));
                }
            }
        }

        Ok(())
    }

    /// Delete a topic
    pub fn delete_topic(&self, name: &str) -> Result<()> {
        info!("Deleting topic: {}", name);

        let admin = self.admin.as_ref()
            .ok_or_else(|| anyhow!("Admin client not initialized"))?;

        if name.is_empty() {
            return Err(anyhow!("Topic name cannot be empty"));
        }

        // Create admin options
        let admin_opts = AdminOptions::new()
            .request_timeout(Some(Duration::from_secs(30)))
            .operation_timeout(Some(Duration::from_secs(30)));

        // Use tokio runtime for async admin operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create runtime: {}", e))?;

        // Execute delete topic
        let results = rt.block_on(async {
            admin.delete_topics(&[name], &admin_opts).await
        }).map_err(|e| anyhow!("Failed to delete topic: {}", e))?;

        // Wait for results
        for result in results {
            match result {
                Ok(_) => {
                    info!("Topic {} deleted successfully", name);
                }
                Err((topic, error)) => {
                    error!("Failed to delete topic {}: {}", topic, error);
                    return Err(anyhow!("Failed to delete topic '{}': {}", topic, error));
                }
            }
        }

        Ok(())
    }

    /// Get topic metadata
    pub fn get_topic_metadata(&self, name: &str) -> Result<TopicMetadata> {
        info!("Getting metadata for topic: {}", name);

        let admin = self.admin.as_ref()
            .ok_or_else(|| anyhow!("Admin client not initialized"))?;

        // Fetch metadata
        let metadata = admin.inner()
            .fetch_metadata(None, Duration::from_secs(10))
            .map_err(|e| anyhow!("Failed to fetch metadata: {}", e))?;

        // Find the topic
        let topic_metadata = metadata
            .topics()
            .iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| anyhow!("Topic '{}' not found", name))?;

        // Check if internal topic
        let internal = topic_metadata.name().starts_with("__") ||
                       topic_metadata.name().starts_with("_");

        // Extract partition metadata
        let partitions: Vec<PartitionMetadata> = topic_metadata
            .partitions()
            .iter()
            .map(|p| PartitionMetadata {
                id: p.id(),
                leader: p.leader(),
                replicas: p.replicas().to_vec(),
                isr: p.isr().to_vec(),
            })
            .collect();

        Ok(TopicMetadata {
            name: name.to_string(),
            partitions,
            internal,
        })
    }

    /// Get list of broker IDs
    pub fn list_brokers(&self) -> Result<Vec<i32>> {
        info!("Listing brokers for: {}", self.connection.get_name());

        let admin = self.admin.as_ref()
            .ok_or_else(|| anyhow!("Admin client not initialized"))?;

        // Fetch metadata
        let metadata = admin.inner()
            .fetch_metadata(None, Duration::from_secs(10))
            .map_err(|e| anyhow!("Failed to fetch metadata: {}", e))?;

        // Extract broker IDs
        let brokers: Vec<i32> = metadata
            .brokers()
            .iter()
            .map(|b| b.id())
            .collect();

        info!("Found {} brokers", brokers.len());
        Ok(brokers)
    }

    /// Create a consumer for consuming messages
    pub fn create_consumer(&self, group_id: &str) -> Result<ConsumerWrapper> {
        info!("Creating consumer with group: {}", group_id);

        let bootstrap_servers = self.connection.get_bootstrap_servers();

        let mut config = ClientConfig::new();
        config.set("bootstrap.servers", &bootstrap_servers);
        config.set("group.id", group_id);
        config.set("enable.auto.commit", "false");
        config.set("client.id", format!("offset-explorer-rust-consumer-{}",
                                        self.connection.get_name()));

        // Add security configuration
        match self.connection.get_security_type() {
            crate::config::BrokerSecurityType::PLAINTEXT => {
                // No additional config needed
            }
            _ => {
                warn!("Security type {:?} not yet supported for consumer",
                      self.connection.get_security_type());
            }
        }

        let consumer: BaseConsumer = config
            .create()
            .map_err(|e| anyhow!("Failed to create consumer: {}", e))?;

        Ok(ConsumerWrapper::new(consumer))
    }

    /// Produce a message to a topic
    pub fn produce_message(
        &self,
        topic: &str,
        key: Option<Vec<u8>>,
        payload: Option<Vec<u8>>
    ) -> Result<()> {
        debug!("Producing message to topic: {}", topic);

        let producer = self.producer.as_ref()
            .ok_or_else(|| anyhow!("Producer not initialized"))?;

        // Create base record
        let mut record = BaseRecord::to(topic);

        // Set payload
        if let Some(ref data) = payload {
            record = record.payload(data);
        }

        // Set key
        if let Some(ref key_data) = key {
            record = record.key(key_data);
        }

        // Send message
        producer.send(record)
            .map_err(|(e, _)| anyhow!("Failed to produce message: {}", e))?;

        debug!("Message produced successfully to topic: {}", topic);
        Ok(())
    }

    /// Get consumer group offsets
    pub fn list_consumer_groups(&self) -> Result<Vec<ConsumerGroupInfo>> {
        info!("Listing consumer groups");

        let admin = self.admin.as_ref()
            .ok_or_else(|| anyhow!("Admin client not initialized"))?;

        // List consumer groups - use metadata to get group info
        let metadata = admin.inner()
            .fetch_metadata(None, Duration::from_secs(10))
            .map_err(|e| anyhow!("Failed to fetch metadata: {}", e))?;

        // Note: rdkafka's AdminClient doesn't have a direct list_consumer_groups method
        // This is a stub implementation that returns empty list
        // TODO: Implement using Kafka Admin API directly or wait for rdkafka support

        warn!("Consumer group listing not fully implemented - returning empty list");
        Ok(Vec::new())
    }

    // ==================== ACL Management Methods ====================

    /// List ACL bindings with optional filter
    pub fn list_acls(&self, _filter: Option<crate::acls::AclFilter>) -> Result<Vec<crate::acls::AclBinding>> {
        info!("Listing ACLs for: {}", self.connection.get_name());

        // Note: rdkafka doesn't have native ACL management in the basic AdminClient
        // This is a stub implementation for now
        // TODO: Implement using Kafka Admin API directly or wait for rdkafka support
        warn!("ACL listing not fully implemented - returning empty list");
        Ok(Vec::new())
    }

    /// Create a new ACL
    pub fn create_acl(&self, request: &crate::acls::CreateAclRequest) -> Result<()> {
        info!("Creating ACL on: {}", self.connection.get_name());

        // Note: rdkafka doesn't have native ACL management in the basic AdminClient
        // TODO: Implement using Kafka Admin API directly
        warn!("ACL creation not fully implemented");
        Ok(())
    }

    /// Delete an ACL
    pub fn delete_acl(
        &self,
        _principal: &str,
        _resource_type: &str,
        _resource_name: &str,
        _operation: &str,
        _permission_type: &str,
        _host: &str,
    ) -> Result<()> {
        info!("Deleting ACL on: {}", self.connection.get_name());

        // Note: rdkafka doesn't have native ACL management in the basic AdminClient
        // TODO: Implement using Kafka Admin API directly
        warn!("ACL deletion not fully implemented");
        Ok(())
    }

    /// Close the mapper and release resources
    pub fn close(&self) -> Result<()> {
        info!("Closing Kafka mapper for: {}", self.connection.get_name());

        // Drop admin and producer
        // They will be automatically cleaned up when dropped
        drop(&self.admin);
        drop(&self.producer);

        Ok(())
    }
}

/// Topic metadata
#[derive(Debug, Clone)]
pub struct TopicMetadata {
    pub name: String,
    pub partitions: Vec<PartitionMetadata>,
    pub internal: bool,
}

/// Partition metadata
#[derive(Debug, Clone)]
pub struct PartitionMetadata {
    pub id: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>,
}

/// Consumer group information
#[derive(Debug, Clone)]
pub struct ConsumerGroupInfo {
    pub group_id: String,
    pub state: String,
    pub protocol_type: Option<String>,
    pub members: Vec<ConsumerMember>,
}

/// Consumer member information
#[derive(Debug, Clone)]
pub struct ConsumerMember {
    pub member_id: String,
    pub client_id: String,
    pub client_host: String,
}

/// Consumer wrapper for message consumption
pub struct ConsumerWrapper {
    consumer: BaseConsumer,
}

impl ConsumerWrapper {
    fn new(consumer: BaseConsumer) -> Self {
        Self { consumer }
    }

    /// Subscribe to topics
    pub fn subscribe(&self, topics: &[&str]) -> Result<()> {
        self.consumer.subscribe(topics)
            .map_err(|e| anyhow!("Failed to subscribe: {}", e))
    }

    /// Assign specific partitions
    pub fn assign(&self, topic: &str, partition: i32, offset: i64) -> Result<()> {
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(topic, partition, rdkafka::Offset::Offset(offset))?;

        self.consumer.assign(&tpl)
            .map_err(|e| anyhow!("Failed to assign partition: {}", e))
    }

    /// Poll for messages
    pub fn poll(&self, timeout_ms: u32) -> Result<Option<KafkaMessage>> {
        match self.consumer.poll(Duration::from_millis(timeout_ms as u64)) {
            Some(Ok(msg)) => {
                let kafka_msg = KafkaMessage {
                    topic: Some(msg.topic().to_string()),
                    partition: msg.partition(),
                    offset: msg.offset(),
                    key: msg.key().map(|k| k.to_vec()),
                    payload: msg.payload().map(|p| p.to_vec()),
                    timestamp: msg.timestamp().to_millis().unwrap_or(0),
                };
                Ok(Some(kafka_msg))
            }
            Some(Err(e)) => {
                error!("Consumer error: {}", e);
                Err(anyhow!("Consumer error: {}", e))
            }
            None => Ok(None),
        }
    }

    /// Get consumer position (offsets)
    pub fn position(&self) -> Result<Vec<(String, i32, i64)>> {
        let tpl = self.consumer.position()
            .map_err(|e| anyhow!("Failed to get position: {}", e))?;

        let result: Vec<(String, i32, i64)> = tpl
            .elements()
            .iter()
            .filter_map(|elem| {
                let topic = elem.topic();
                let offset = match elem.offset() {
                    rdkafka::Offset::Offset(o) => o,
                    _ => return None,
                };
                Some((topic.to_string(), elem.partition(), offset))
            })
            .collect();

        Ok(result)
    }

    /// Commit offsets
    pub fn commit(&self) -> Result<()> {
        let assignment = self.consumer.assignment()
            .map_err(|e| anyhow!("Failed to get assignment: {}", e))?;
        self.consumer.commit(&assignment, rdkafka::consumer::CommitMode::Sync)
            .map_err(|e| anyhow!("Failed to commit: {}", e))
    }

    /// Seek to offset
    pub fn seek(&self, topic: &str, partition: i32, offset: i64) -> Result<()> {
        self.consumer.seek(
            topic,
            partition,
            rdkafka::Offset::Offset(offset),
            Duration::from_secs(10)
        ).map_err(|e| anyhow!("Failed to seek: {}", e))
    }
}

/// Kafka message
#[derive(Debug, Clone)]
pub struct KafkaMessage {
    pub topic: Option<String>,
    pub partition: i32,
    pub offset: i64,
    pub key: Option<Vec<u8>>,
    pub payload: Option<Vec<u8>>,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_structs() {
        let partition = PartitionMetadata {
            id: 0,
            leader: 1,
            replicas: vec![1, 2],
            isr: vec![1],
        };
        assert_eq!(partition.id, 0);
        assert_eq!(partition.leader, 1);
    }

    #[test]
    fn test_consumer_group_info() {
        let group = ConsumerGroupInfo {
            group_id: "test-group".to_string(),
            state: "Stable".to_string(),
            protocol_type: Some("consumer".to_string()),
            members: vec![],
        };
        assert_eq!(group.group_id, "test-group");
    }
}

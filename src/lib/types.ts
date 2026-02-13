// Type definitions for Rust backend responses

/// Server connection settings
export interface ServerConnectionSettings {
  id: number;
  name: string;
  host: string;
  port?: number;
  bootstrapServers?: string;
  securityType: string;
  zookeeperHosts?: string;
  zookeeperChroot?: string;
}

/// Task progress information
export interface TaskProgress {
  current: number;
  total: number;
  message: string;
  isComplete: boolean;
  error?: string;
}

/// Topic metadata
export interface TopicMetadata {
  name: string;
  partitionCount: number;
  replicationFactor: number;
  internal: boolean;
  partitions: PartitionInfo[];
}

/// Partition information
export interface PartitionInfo {
  id: number;
  leader: number;
  replicas: number[];
  isr: number[];
}

/// Consumer group
export interface ConsumerGroup {
  groupId: string;
  state: string;
  protocolType?: string;
}

/// Consumer member
export interface ConsumerMember {
  memberId: string;
  clientId: string;
  clientHost: string;
}

/// ZooKeeper node information
export interface ZkNode {
  path: string;
  data: string;
  isDirectory: boolean;
  childCount: number;
  creationTime: number;
  modificationTime: number;
  version: number;
  cversion: number;
  stat: ZkStat;
  acl: ZkAcl[];
}

/// ZooKeeper stat information
export interface ZkStat {
  version: number;
  cversion: number;
  dataLength: number;
}

/// ZooKeeper ACL information
export interface ZkAcl {
  scheme: string;
  id: string;
  permissions: string;
  host: string;
}

/// Kafka message
export interface KafkaMessage {
  topic?: string;
  partition: number;
  key?: string;
  value?: string;
  timestamp: number;
  offset: number; // Required parameter, moved before limit
  limit?: number;
}

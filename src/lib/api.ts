// Tauri API invocation layer
// Bridges frontend Svelte components with Rust backend

import { invoke } from '@tauri-apps/api/core';
import type { InvokeArgs } from '@tauri-apps/api/core';
import type { ServerConnectionSettings } from './types';

// ==================== Type Definitions ====================

/// Server connection request
export interface ServerConnectionRequest {
  name: string;
  bootstrapServers?: string;
  securityType?: string;
  zookeeperHosts?: string;
  zookeeperChroot?: string;
}

/// Topic creation request
export interface CreateTopicRequest {
  name: string;
  partitions: number;
  replicationFactor: number;
}

/// Message production request
export interface ProduceMessageRequest {
  topic: string;
  key?: string;
  value?: string;
  headers?: Record<string, string>;
}

/// Topic metadata response
export interface TopicMetadataResponse {
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

/// Consumer group response
export interface ConsumerGroupResponse {
  groupId: string;
  state: string;
  protocolType?: string;
  members: ConsumerMemberResponse[];
}

/// Consumer member response
export interface ConsumerMemberResponse {
  memberId: string;
  clientId: string;
  clientHost: string;
}

/// Message response
export interface KafkaMessageResponse {
  topic?: string;
  partition: number;
  offset: number;
  key?: string;
  value?: string;
  timestamp: number;
}

/// Error response
export interface ErrorResponse {
  error: string;
  message: string;
}

// ==================== Tauri Command API ====================

/**
 * Server Management Commands
 */
export const getServerConnections = async (): Promise<ServerConnectionSettings[]> => {
  return await invoke<ServerConnectionSettings[]>('get_server_connections');
};

export const addServerConnection = async (request: ServerConnectionRequest): Promise<number> => {
  return await invoke<number>('add_server_connection', { request });
};

export const updateServerConnection = async (id: number, request: ServerConnectionRequest): Promise<void> => {
  return await invoke<void>('update_server_connection', { id, request });
};

export const removeServerConnection = async (id: number): Promise<void> => {
  return await invoke<void>('remove_server_connection', { id });
};

export const connectToServer = async (id: number): Promise<void> => {
  return await invoke<void>('connect_to_server', { id });
};

export const disconnectFromServer = async (id: number): Promise<void> => {
  return await invoke<void>('disconnect_from_server', { id });
};

/**
 * Topic Management Commands
 */
export const listTopics = async (serverId: number): Promise<string[]> => {
  return await invoke<string[]>('list_topics', { serverId });
};

export const createTopic = async (serverId: number, request: CreateTopicRequest): Promise<void> => {
  return await invoke<void>('create_topic', { serverId, request });
};

export const deleteTopic = async (serverId: number, topicName: string): Promise<void> => {
  return await invoke<void>('delete_topic', { serverId, topicName });
};

export const getTopicMetadata = async (serverId: number, topicName: string): Promise<TopicMetadataResponse> => {
  return await invoke<TopicMetadataResponse>('get_topic_metadata', { serverId, topicName });
};

export const getTopicPartitions = async (serverId: number, topicName: string): Promise<PartitionInfo[]> => {
  return await invoke<PartitionInfo[]>('get_topic_partitions', { serverId, topicName });
};

/**
 * Message Operation Commands
 */
export const consumeMessages = async (
  serverId: number,
  topic: string,
  limit: number,
  partition?: number,
  offset?: number
): Promise<KafkaMessageResponse[]> => {
  return await invoke<KafkaMessageResponse[]>('consume_messages', {
    serverId,
    topic,
    partition,
    offset,
    limit
  });
};

export const produceMessage = async (serverId: number, request: ProduceMessageRequest): Promise<void> => {
  return await invoke<void>('produce_message', { serverId, request });
};

/**
 * Consumer Group Commands
 */
export const listConsumerGroups = async (serverId: number): Promise<ConsumerGroupResponse[]> => {
  return await invoke<ConsumerGroupResponse[]>('list_consumer_groups', { serverId });
};

export const getConsumerGroupDetails = async (serverId: number, groupId: string): Promise<any> => {
  return await invoke<any>('get_consumer_group_details', { serverId, groupId });
};

export const resetConsumerOffset = async (
  serverId: number,
  groupId: string,
  topic: string,
  partition: number,
  offset: number
): Promise<void> => {
  return await invoke<void>('reset_consumer_offset', {
    serverId,
    groupId,
    topic,
    partition,
    offset
  });
};

/**
 * Background Task Commands
 */
export const getTaskProgress = async (taskId: string): Promise<TaskProgress> => {
  return await invoke<TaskProgress>('get_task_progress', { taskId });
};

export const cancelTask = async (taskId: string): Promise<void> => {
  return await invoke<void>('cancel_task', { taskId });
};

export const listTasks = async (): Promise<string[]> => {
  return await invoke<string[]>('list_tasks');
};

/**
 * ZooKeeper Browser Commands
 */
export interface ZkNode {
  path: string;
  data: string;
  isDirectory: boolean;
  childCount: number;
  creationTime: number;
  modificationTime: number;
  version: number;
  acl: ZkAcl[];
  stat: ZkStat;
}

export interface ZkAcl {
  scheme: string;
  id: string;
  permissions: string;
  host: string;
}

export interface ZkStat {
  dataLength: number;
  version: number;
  cversion: number;
  aversion: number;
  ephemeralOwner: string;
  ctime: number;
  mtime: number;
  pzxid: number;
  numChildren: number;
}

export interface CreateZkNodeRequest {
  path: string;
  data: string;
  isBinary: boolean;
  createMode: string;
  acl: ZkAcl[];
}

export interface UpdateZkNodeRequest {
  path: string;
  data: string;
  expectedVersion?: number;
  isBinary: boolean;
}

export interface DeleteZkNodeRequest {
  path: string;
  recursive?: boolean;
  expectedVersion?: number;
}

export const getZkChildren = async (serverId: number, path: string): Promise<ZkNode[]> => {
  return await invoke<ZkNode[]>('get_zk_children', { serverId, path });
};

export const getZkNode = async (serverId: number, path: string): Promise<ZkNode> => {
  return await invoke<ZkNode>('get_zk_node', { serverId, path });
};

export const createZkNode = async (serverId: number, request: CreateZkNodeRequest): Promise<void> => {
  return await invoke<void>('create_zk_node', { serverId, request });
};

export const updateZkNode = async (serverId: number, request: UpdateZkNodeRequest): Promise<void> => {
  return await invoke<void>('update_zk_node', { serverId, request });
};

export const deleteZkNode = async (serverId: number, request: DeleteZkNodeRequest): Promise<void> => {
  return await invoke<void>('delete_zk_node', { serverId, request });
};

/**
 * ACL Management Commands
 */
export interface AclBinding {
  principal: string;
  resourceType: string;
  resourceName: string;
  operation: string;
  permissionType: string;
  host?: string;
  wildcard?: boolean;
}

export interface CreateAclRequest {
  principal: string;
  resourceType: string;
  resourceName: string;
  operation: string;
  permissionType: string;
  host?: string;
  wildcard?: boolean;
}

export const listAcls = async (serverId: number, filter?: {
  resourceType?: string;
  resourceName?: string;
  principal?: string;
  operation?: string;
  permissionType?: string;
}): Promise<AclBinding[]> => {
  return await invoke<AclBinding[]>('list_acls', { serverId, filter });
};

export const createAcl = async (serverId: number, request: CreateAclRequest): Promise<void> => {
  return await invoke<void>('create_acl', { serverId, request });
};

export const deleteAcl = async (
  serverId: number,
  principal: string,
  resourceType: string,
  resourceName: string,
  operation: string,
  permissionType: string
): Promise<void> => {
  return await invoke<void>('delete_acl', {
    serverId,
    principal,
    resourceType,
    resourceName,
    operation,
    permissionType
  });
};

// ==================== Type Imports ====================

/// Re-export types from Rust backend
export type { ServerConnectionSettings } from './types';
export type TaskProgress = {
  current: number;
  total: number;
  message: string;
  isComplete: boolean;
  error?: string;
};

// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// ZooKeeper browser module

use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

/// ZooKeeper node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkNode {
    /// Node path
    pub path: String,

    /// Node data (base64 encoded if binary)
    pub data: String,

    /// Is this a directory?
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,

    /// Number of children
    #[serde(rename = "childCount")]
    pub child_count: usize,

    /// Node creation time
    #[serde(rename = "creationTime")]
    pub creation_time: i64,

    /// Node modification time
    #[serde(rename = "modificationTime")]
    pub modification_time: i64,

    /// Node version
    pub version: i32,

    /// ACL list
    pub acl: Vec<ZkAcl>,

    /// Stat information
    pub stat: ZkStat,
}

/// ZooKeeper ACL information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkAcl {
    /// Schema for ACL
    pub scheme: String,

    /// ID (e.g., "username")
    #[serde(rename = "id")]
    pub id: String,

    /// Permissions
    pub permissions: String,

    /// Host
    pub host: String,
}

/// ZooKeeper stat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkStat {
    /// Data length
    #[serde(rename = "dataLength")]
    pub data_length: usize,

    /// Version
    pub version: i32,

    /// Cversion
    pub cversion: i32,

    /// Aversion
    pub aversion: i32,

    /// Ephemeral owner
    #[serde(rename = "ephemeralOwner")]
    pub ephemeral_owner: String,

    /// Creation time
    #[serde(rename = "ctime")]
    pub ctime: i64,

    /// Modification time
    #[serde(rename = "mtime")]
    pub mtime: i64,

    /// PZxid
    pub pzxid: i64,

    /// Number of children
    #[serde(rename = "numChildren")]
    pub num_children: i32,
}

/// ZooKeeper connection request
#[derive(Debug, Deserialize, Serialize)]
pub struct ZkConnectionRequest {
    /// Connection name
    pub name: String,

    /// ZooKeeper hosts (comma-separated)
    #[serde(rename = "zkHosts")]
    pub zk_hosts: String,

    /// Chroot path
    #[serde(rename = "chroot")]
    pub chroot: Option<String>,

    /// Session timeout (ms)
    #[serde(rename = "sessionTimeout")]
    pub session_timeout: Option<u32>,

    /// Authentication scheme
    #[serde(rename = "authScheme")]
    pub auth_scheme: Option<String>,

    /// Authentication (for digest/sasl)
    pub auth: Option<String>,
}

/// ZooKeeper node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkNodeData {
    /// Node path
    pub path: String,

    /// Node data
    pub data: String,

    /// Is binary data?
    #[serde(rename = "isBinary")]
    pub is_binary: bool,
}

/// ZooKeeper create request
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateZkNodeRequest {
    /// Node path
    pub path: String,

    /// Node data
    pub data: String,

    /// Is binary data?
    #[serde(rename = "isBinary")]
    pub is_binary: bool,

    /// Create mode (persistent, ephemeral, sequence)
    #[serde(rename = "createMode")]
    pub create_mode: String,

    /// ACL list
    pub acl: Vec<ZkAcl>,
}

/// ZooKeeper update request
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateZkNodeRequest {
    /// Node path
    pub path: String,

    /// Node data
    pub data: String,

    /// Expected version (for optimistic locking)
    #[serde(rename = "expectedVersion")]
    pub expected_version: Option<i32>,

    /// Is binary data?
    #[serde(rename = "isBinary")]
    pub is_binary: bool,
}

impl CreateZkNodeRequest {
    /// Validate create request
    pub fn validate(&self) -> Result<()> {
        if self.path.is_empty() {
            return Err(anyhow!("Path cannot be empty"));
        }

        if !self.path.starts_with('/') {
            return Err(anyhow!("Path must start with /"));
        }

        // Validate create mode
        let valid_modes = ["persistent", "ephemeral", "sequential", "container"];
        if !valid_modes.contains(&self.create_mode.as_str()) {
            return Err(anyhow!("Invalid create mode: {}", self.create_mode));
        }

        Ok(())
    }
}

/// ZooKeeper delete request
#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteZkNodeRequest {
    /// Node path
    pub path: String,

    /// Expected version (for optimistic locking)
    #[serde(rename = "expectedVersion")]
    pub expected_version: Option<i32>,

    /// Recursive delete?
    pub recursive: bool,
}

/// ZooKeeper children response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkChildren {
    /// Node path
    pub path: String,

    /// Child names
    pub children: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zk_path_validation() {
        let valid_request = CreateZkNodeRequest {
            path: "/test/path".to_string(),
            data: "data".to_string(),
            is_binary: false,
            create_mode: "persistent".to_string(),
            acl: vec![],
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateZkNodeRequest {
            path: "invalid/path".to_string(),
            data: "data".to_string(),
            is_binary: false,
            create_mode: "persistent".to_string(),
            acl: vec![],
        };

        assert!(invalid_request.validate().is_err());
    }
}

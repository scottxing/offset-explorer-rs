// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// ACL (Access Control List) management for Kafka

use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

/// ACL binding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclBinding {
    /// Principal (User:username or User:* for all users)
    #[serde(rename = "principal")]
    pub principal: String,

    /// Resource type (Topic, Group, Cluster, etc.)
    #[serde(rename = "resourceType")]
    pub resource_type: String,

    /// Resource name (topic name, consumer group ID, etc.)
    #[serde(rename = "resourceName")]
    pub resource_name: String,

    /// Operation (READ, WRITE, DELETE, etc.)
    #[serde(rename = "operation")]
    pub operation: AclOperation,

    /// Permission type (ALLOW, DENY)
    #[serde(rename = "permissionType")]
    pub permission_type: AclPermission,

    /// Host pattern (* for all hosts, or specific host)
    #[serde(rename = "host")]
    pub host: String,

    /// Is this a wildcard pattern?
    #[serde(rename = "wildcard")]
    pub wildcard: bool,
}

/// ACL operation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AclOperation {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Create operation
    Create,
    /// Delete operation
    Delete,
    /// Alter operation
    Alter,
    /// Describe operation
    Describe,
    /// Cluster action
    ClusterAction,
    /// Describe configs
    DescribeConfigs,
    /// Alter configs
    AlterConfigs,
    /// All operations
    All,
}

/// ACL permission types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AclPermission {
    /// Allow access
    Allow,
    /// Deny access
    Deny,
}

/// Resource types for ACLs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AclResourceType {
    /// Topic resource
    Topic,
    /// Consumer group resource
    Group,
    /// Cluster resource
    Cluster,
    /// Transactional ID resource
    TransactionalId,
    /// Delegation token resource
    DelegationToken,
}

/// ACL creation request
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAclRequest {
    /// Principal (e.g., "User:alice" or "User:*")
    pub principal: String,

    /// Resource type (Topic, Group, Cluster, etc.)
    #[serde(rename = "resourceType")]
    pub resource_type: String,

    /// Resource name (topic name, group ID, etc.)
    #[serde(rename = "resourceName")]
    pub resource_name: String,

    /// Operation (Read, Write, Delete, etc.)
    #[serde(rename = "operation")]
    pub operation: String,

    /// Permission type (Allow, Deny)
    #[serde(rename = "permissionType")]
    pub permission_type: String,

    /// Host pattern (* for all hosts)
    #[serde(rename = "host")]
    pub host: Option<String>,

    /// Is this a wildcard pattern?
    #[serde(rename = "wildcard")]
    pub wildcard: Option<bool>,
}

impl CreateAclRequest {
    /// Validate ACL request
    pub fn validate(&self) -> Result<()> {
        if self.principal.is_empty() {
            return Err(anyhow!("Principal cannot be empty"));
        }

        if self.resource_name.is_empty() {
            return Err(anyhow!("Resource name cannot be empty"));
        }

        // Validate resource type
        let valid_types = ["Topic", "Group", "Cluster", "TransactionalId", "DelegationToken"];
        if !valid_types.contains(&self.resource_type.as_str()) {
            return Err(anyhow!("Invalid resource type: {}", self.resource_type));
        }

        // Validate operation
        let valid_operations = ["Read", "Write", "Create", "Delete", "Alter", "Describe",
                           "ClusterAction", "DescribeConfigs", "AlterConfigs", "All"];
        if !valid_operations.contains(&self.operation.as_str()) {
            return Err(anyhow!("Invalid operation: {}", self.operation));
        }

        // Validate permission type
        if self.permission_type != "Allow" && self.permission_type != "Deny" {
            return Err(anyhow!("Invalid permission type: {}", self.permission_type));
        }

        Ok(())
    }
}

/// ACL filter for listing
#[derive(Debug, Deserialize, Serialize)]
pub struct AclFilter {
    /// Resource type to filter by
    #[serde(rename = "resourceType")]
    pub resource_type: Option<String>,

    /// Resource name pattern to filter by
    #[serde(rename = "resourceName")]
    pub resource_name: Option<String>,

    /// Principal to filter by
    pub principal: Option<String>,

    /// Operation to filter by
    pub operation: Option<String>,

    /// Permission type to filter by
    #[serde(rename = "permissionType")]
    pub permission_type: Option<String>,

    /// Host to filter by
    pub host: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_acl_request() {
        let valid_request = CreateAclRequest {
            principal: "User:alice".to_string(),
            resource_type: "Topic".to_string(),
            resource_name: "test-topic".to_string(),
            operation: "Read".to_string(),
            permission_type: "Allow".to_string(),
            host: Some("*".to_string()),
            wildcard: Some(false),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateAclRequest {
            principal: "".to_string(),
            resource_type: "Topic".to_string(),
            resource_name: "test-topic".to_string(),
            operation: "Read".to_string(),
            permission_type: "Allow".to_string(),
            host: None,
            wildcard: None,
        };

        assert!(invalid_request.validate().is_err());
    }
}

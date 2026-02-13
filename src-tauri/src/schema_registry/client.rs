// Copyright 2024 Offset Explorer Rust Team
// Licensed under Proprietary License
//
// Schema Registry REST client
// Confluent Schema Registry API implementation

use anyhow::{Result, anyhow};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

/// Schema Registry client
/// Compatible with Confluent Schema Registry REST API
pub struct SchemaRegistryClient {
    base_url: String,
    client: Arc<Client>,
    schema_cache: Arc<RwLock<HashMap<String, CachedSchema>>>,
    auth: Option<SchemaRegistryAuth>,
}

/// Authentication configuration
#[derive(Clone, Debug)]
pub enum SchemaRegistryAuth {
    Basic { username: String, password: String },
    Bearer { token: String },
}

/// Cached schema with metadata
#[derive(Clone, Debug)]
struct CachedSchema {
    schema: SchemaInfo,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub subject: String,
    pub version: i32,
    pub id: i32,
    pub schema: String,
    pub schema_type: SchemaType,
}

/// Schema type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SchemaType {
    AVRO,
    PROTOBUF,
    JSON,
}

impl SchemaRegistryClient {
    /// Create a new Schema Registry client
    pub fn new(base_url: String) -> Result<Self> {
        Self::with_auth(base_url, None)
    }

    /// Create a new client with authentication
    pub fn with_auth(base_url: String, auth: Option<SchemaRegistryAuth>) -> Result<Self> {
        // Ensure base URL doesn't have trailing slash
        let base_url = base_url.trim_end_matches('/');

        let mut client_builder = Client::builder();

        // Add timeout
        client_builder = client_builder.timeout(std::time::Duration::from_secs(30));

        // Add basic auth if provided
        if let Some(SchemaRegistryAuth::Basic { username, password }) = &auth {
            // Request basic auth header will be added per-request
        }

        let client = Arc::new(client_builder.build()?);

        Ok(Self {
            base_url: base_url.to_string(),
            client,
            schema_cache: Arc::new(RwLock::new(HashMap::new())),
            auth,
        })
    }

    /// Get all subjects
    pub async fn get_subjects(&self) -> Result<Vec<String>> {
        info!("Fetching all subjects from Schema Registry");

        let url = format!("{}/subjects", self.base_url);
        let response = self.execute_get(&url).await?;

        let subjects: Vec<String> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse subjects response: {}", e))?;

        info!("Found {} subjects", subjects.len());
        Ok(subjects)
    }

    /// Get all versions of a subject
    pub async fn get_subject_versions(&self, subject: &str) -> Result<Vec<i32>> {
        info!("Fetching versions for subject: {}", subject);

        let url = format!("{}/subjects/{}/versions", self.base_url, subject);
        let response = self.execute_get(&url).await?;

        let versions: Vec<i32> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse versions response: {}", e))?;

        Ok(versions)
    }

    /// Get a specific version of a subject's schema
    pub async fn get_schema(&self, subject: &str, version: i32) -> Result<SchemaInfo> {
        info!("Fetching schema {} version {}", subject, version);

        // Check cache first
        {
            let cache = self.schema_cache.read().await;
            let cache_key = format!("{}:v{}", subject, version);
            if let Some(cached) = cache.get(&cache_key) {
                // Cache is valid for 1 hour
                let age = chrono::Utc::now() - cached.timestamp;
                if age.num_hours() < 1 {
                    debug!("Returning cached schema for {}:v{}", subject, version);
                    return Ok(cached.schema.clone());
                }
            }
        }

        let url = format!("{}/subjects/{}/versions/{}", self.base_url, subject, version);
        let response = self.execute_get(&url).await?;

        let schema: SchemaInfo = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse schema response: {}", e))?;

        // Cache the schema
        {
            let mut cache = self.schema_cache.write().await;
            let cache_key = format!("{}:v{}", subject, version);
            cache.insert(cache_key, CachedSchema {
                schema: schema.clone(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(schema)
    }

    /// Get the latest version of a subject's schema
    pub async fn get_latest_schema(&self, subject: &str) -> Result<SchemaInfo> {
        info!("Fetching latest schema for subject: {}", subject);

        // Try to get using version "latest"
        match self.get_schema(subject, -1).await {
            Ok(schema_info) => Ok(schema_info),
            Err(_) => {
                // Fallback to getting versions and fetching the last one
                let versions = self.get_subject_versions(subject).await?;
                if versions.is_empty() {
                    return Err(anyhow!("No versions found for subject: {}", subject));
                }
                let latest_version = *versions.iter().max().unwrap();
                self.get_schema(subject, latest_version).await
            }
        }
    }

    /// Register a new schema
    pub async fn register_schema(&self, subject: &str, schema: &str, schema_type: SchemaType) -> Result<i32> {
        info!("Registering new schema for subject: {}", subject);

        let url = format!("{}/subjects/{}", self.base_url, subject);

        #[derive(Serialize)]
        struct SchemaRequest {
            schema: String,
            schema_type: SchemaType,
        }

        let request = SchemaRequest {
            schema: schema.to_string(),
            schema_type,
        };

        let response = self.execute_post(&url, &request).await?;

        // Response contains {"id": 123}
        #[derive(Deserialize)]
        struct SchemaIdResponse {
            id: i32,
        }

        let schema_id: SchemaIdResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse schema ID response: {}", e))?;

        info!("Registered schema with ID: {}", schema_id.id);
        Ok(schema_id.id)
    }

    /// Check if a schema is compatible with existing versions
    pub async fn check_compatibility(
        &self,
        subject: &str,
        schema: &str,
        schema_type: SchemaType,
    ) -> Result<bool> {
        info!("Checking compatibility for subject: {}", subject);

        let url = format!("{}/compatibility/subjects/{}", self.base_url, subject);

        #[derive(Serialize)]
        struct CompatibilityRequest {
            schema: String,
            schema_type: SchemaType,
        }

        let request = CompatibilityRequest {
            schema: schema.to_string(),
            schema_type,
        };

        match self.execute_post(&url, &request).await {
            Ok(response) => {
                if response.status() == StatusCode::OK {
                    info!("Schema is compatible");
                    Ok(true)
                } else {
                    let text = response.text().await.unwrap_or_default();
                    warn!("Schema compatibility check failed: {}", text);
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Error checking compatibility: {}", e);
                Ok(false)
            }
        }
    }

    /// Get schema by ID
    pub async fn get_schema_by_id(&self, id: i32) -> Result<SchemaInfo> {
        info!("Fetching schema by ID: {}", id);

        // Check cache first
        {
            let cache = self.schema_cache.read().await;
            let cache_key = format!("id:{}", id);
            if let Some(cached) = cache.get(&cache_key) {
                let age = chrono::Utc::now() - cached.timestamp;
                if age.num_hours() < 1 {
                    debug!("Returning cached schema for ID {}", id);
                    return Ok(cached.schema.clone());
                }
            }
        }

        let url = format!("{}/schemas/ids/{}", self.base_url, id);
        let response = self.execute_get(&url).await?;

        let schema: SchemaInfo = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse schema by ID response: {}", e))?;

        // Cache the schema
        {
            let mut cache = self.schema_cache.write().await;
            let cache_key = format!("id:{}", id);
            cache.insert(cache_key, CachedSchema {
                schema: schema.clone(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(schema)
    }

    /// Execute GET request with authentication
    async fn execute_get(&self, url: &str) -> Result<reqwest::Response> {
        debug!("GET {}", url);

        let mut request = self.client.get(url);

        // Add authentication
        if let Some(SchemaRegistryAuth::Basic { username, password }) = &self.auth {
            request = request.basic_auth(username, Some(password));
        } else if let Some(SchemaRegistryAuth::Bearer { token }) = &self.auth {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to execute GET request: {}", e))?;

        Ok(response)
    }

    /// Execute POST request with authentication
    async fn execute_post<T: Serialize>(&self, url: &str, body: &T) -> Result<reqwest::Response> {
        debug!("POST {}", url);

        let mut request = self.client.post(url);

        // Add authentication
        if let Some(SchemaRegistryAuth::Basic { username, password }) = &self.auth {
            request = request.basic_auth(username, Some(password));
        } else if let Some(SchemaRegistryAuth::Bearer { token }) = &self.auth {
            request = request.bearer_auth(token);
        }

        let response = request
            .json(body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to execute POST request: {}", e))?;

        Ok(response)
    }

    /// Clear the schema cache
    pub async fn clear_cache(&self) {
        let mut cache = self.schema_cache.write().await;
        cache.clear();
        info!("Schema cache cleared");
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.schema_cache.read().await;
        let count = cache.len();
        let age_hours = cache
            .values()
            .map(|c| (chrono::Utc::now() - c.timestamp).num_minutes())
            .min()
            .unwrap_or(0) / 60;
        (count, age_hours as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_type_serialization() {
        let schema_type = SchemaType::AVRO;
        let json = serde_json::to_string(&schema_type).unwrap();
        assert_eq!(json, "\"AVRO\"");
    }
}

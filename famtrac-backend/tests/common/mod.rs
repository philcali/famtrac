// Common test utilities for integration testing with DynamoDB Local

pub mod mocks;

use aws_sdk_dynamodb::config::Credentials;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection,
    ProjectionType, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Configuration for DynamoDB Local test instance
#[allow(dead_code)]
pub struct DynamoDbLocalConfig {
    pub port: u16,
    pub table_name: String,
}

/// Handle to a running DynamoDB Local process
pub struct DynamoDbLocalInstance {
    process: Child,
    pub config: DynamoDbLocalConfig,
    pub client: Client,
}

impl DynamoDbLocalInstance {
    /// Start a new DynamoDB Local instance on a random available port
    pub async fn start(table_name: String) -> Result<Self, String> {
        // Check if DynamoDB Local JAR exists
        let jar_path = "../dynamodb/DynamoDBLocal.jar";
        if !Path::new(jar_path).exists() {
            return Err(format!(
                "DynamoDB Local not found at {}. Run scripts/setup-dynamodb-local.sh first.",
                jar_path
            ));
        }

        // Find an available port
        let port = find_available_port()?;

        // Start DynamoDB Local process
        let process = Command::new("java")
            .arg("-Djava.library.path=./dynamodb/DynamoDBLocal_lib")
            .arg("-jar")
            .arg(jar_path)
            .arg("-inMemory")
            .arg("-port")
            .arg(port.to_string())
            .spawn()
            .map_err(|e| format!("Failed to start DynamoDB Local: {}", e))?;

        // Wait for DynamoDB Local to be ready
        thread::sleep(Duration::from_secs(2));

        // Create AWS client
        let endpoint_url = format!("http://localhost:{}", port);
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(&endpoint_url)
            .region("us-east-1")
            .credentials_provider(Credentials::for_tests())
            .load()
            .await;

        let client = Client::new(&config);

        let instance = Self {
            process,
            config: DynamoDbLocalConfig { port, table_name },
            client,
        };

        // Create the test table
        instance.create_test_table().await?;

        Ok(instance)
    }

    /// Create the test table with proper schema
    async fn create_test_table(&self) -> Result<(), String> {
        self.client
            .create_table()
            .table_name(&self.config.table_name)
            .billing_mode(BillingMode::PayPerRequest)
            // Primary key: PK (partition key), SK (sort key)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("PK")
                    .key_type(KeyType::Hash)
                    .build()
                    .map_err(|e| format!("Failed to build PK key schema: {}", e))?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("SK")
                    .key_type(KeyType::Range)
                    .build()
                    .map_err(|e| format!("Failed to build SK key schema: {}", e))?,
            )
            // Attribute definitions
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("PK")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .map_err(|e| format!("Failed to build PK attribute: {}", e))?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("SK")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .map_err(|e| format!("Failed to build SK attribute: {}", e))?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("owner_id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .map_err(|e| format!("Failed to build owner_id attribute: {}", e))?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("created_at")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .map_err(|e| format!("Failed to build created_at attribute: {}", e))?,
            )
            // GSI-1 for owner_id lookups
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("GSI-1")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("owner_id")
                            .key_type(KeyType::Hash)
                            .build()
                            .map_err(|e| format!("Failed to build GSI-1 PK: {}", e))?,
                    )
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("created_at")
                            .key_type(KeyType::Range)
                            .build()
                            .map_err(|e| format!("Failed to build GSI-1 SK: {}", e))?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .build()
                    .map_err(|e| format!("Failed to build GSI-1: {}", e))?,
            )
            .send()
            .await
            .map_err(|e| format!("Failed to create table: {}", e))?;

        Ok(())
    }

    /// Delete the test table
    pub async fn delete_test_table(&self) -> Result<(), String> {
        self.client
            .delete_table()
            .table_name(&self.config.table_name)
            .send()
            .await
            .map_err(|e| format!("Failed to delete table: {}", e))?;

        Ok(())
    }
}

impl Drop for DynamoDbLocalInstance {
    fn drop(&mut self) {
        // Kill the DynamoDB Local process
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Find an available port for DynamoDB Local
fn find_available_port() -> Result<u16, String> {
    TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to find available port: {}", e))?
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|e| format!("Failed to get port from listener: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run if DynamoDB Local is installed
    async fn test_dynamodb_local_startup() {
        let instance = DynamoDbLocalInstance::start("test-table".to_string())
            .await
            .expect("Failed to start DynamoDB Local");

        // Verify we can list tables
        let result = instance.client.list_tables().send().await;
        assert!(result.is_ok());

        let table_names = result.unwrap().table_names().to_vec();
        assert!(table_names.contains(&"test-table".to_string()));

        // Cleanup
        instance
            .delete_test_table()
            .await
            .expect("Failed to delete test table");
    }
}

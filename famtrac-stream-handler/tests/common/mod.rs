// Common test utilities for stream handler integration tests with DynamoDB Local

use aws_sdk_dynamodb::config::Credentials;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, BillingMode, GlobalSecondaryIndex, KeySchemaElement,
    KeyType, Projection, ProjectionType, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Handle to a running DynamoDB Local process
pub struct DynamoDbLocalInstance {
    process: Child,
    #[allow(dead_code)]
    pub port: u16,
    pub table_name: String,
    pub client: Client,
}

impl DynamoDbLocalInstance {
    /// Start a new DynamoDB Local instance on a random available port
    pub async fn start(table_name: &str) -> Result<Self, String> {
        let jar_path = if Path::new("dynamodb/DynamoDBLocal.jar").exists() {
            "dynamodb/DynamoDBLocal.jar".to_string()
        } else if Path::new("../dynamodb/DynamoDBLocal.jar").exists() {
            "../dynamodb/DynamoDBLocal.jar".to_string()
        } else {
            return Err(
                "DynamoDB Local not found at dynamodb/DynamoDBLocal.jar or ../dynamodb/DynamoDBLocal.jar. Skipping integration test.".to_string()
            );
        };

        let lib_path = if Path::new("dynamodb/DynamoDBLocal_lib").exists() {
            "dynamodb/DynamoDBLocal_lib".to_string()
        } else {
            "../dynamodb/DynamoDBLocal_lib".to_string()
        };

        let port = find_available_port()?;

        let process = Command::new("java")
            .arg(format!("-Djava.library.path={lib_path}"))
            .arg("-jar")
            .arg(&jar_path)
            .arg("-inMemory")
            .arg("-port")
            .arg(port.to_string())
            .spawn()
            .map_err(|e| format!("Failed to start DynamoDB Local: {e}"))?;

        // Wait for DynamoDB Local to be ready (with retries)
        thread::sleep(Duration::from_secs(3));

        let endpoint_url = format!("http://localhost:{port}");
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(&endpoint_url)
            .region("us-east-1")
            .credentials_provider(Credentials::for_tests())
            .load()
            .await;

        let client = Client::new(&config);

        let instance = Self {
            process,
            port,
            table_name: table_name.to_string(),
            client,
        };

        // Retry table creation in case DynamoDB Local isn't fully ready
        let mut last_err = String::new();
        for _ in 0..5 {
            match instance.create_table().await {
                Ok(()) => return Ok(instance),
                Err(e) => {
                    last_err = e;
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
        Err(format!("Failed to create table after retries: {last_err}"))
    }

    async fn create_table(&self) -> Result<(), String> {
        self.client
            .create_table()
            .table_name(&self.table_name)
            .billing_mode(BillingMode::PayPerRequest)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("PK")
                    .key_type(KeyType::Hash)
                    .build()
                    .unwrap(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("SK")
                    .key_type(KeyType::Range)
                    .build()
                    .unwrap(),
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("PK")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .unwrap(),
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("SK")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .unwrap(),
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("family_id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .unwrap(),
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("GSI-family_id")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("family_id")
                            .key_type(KeyType::Hash)
                            .build()
                            .unwrap(),
                    )
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("SK")
                            .key_type(KeyType::Range)
                            .build()
                            .unwrap(),
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .build()
                    .unwrap(),
            )
            .send()
            .await
            .map_err(|e| format!("Failed to create table: {e}"))?;

        // Wait for GSI to become active
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }

    /// Put a raw item into the table
    pub async fn put_item(&self, item: HashMap<String, AttributeValue>) {
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .expect("Failed to put item");
    }

    /// Get a single item by PK/SK
    pub async fn get_item(&self, pk: &str, sk: &str) -> Option<HashMap<String, AttributeValue>> {
        self.client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(pk.to_string()))
            .key("SK", AttributeValue::S(sk.to_string()))
            .send()
            .await
            .expect("Failed to get item")
            .item
    }

    /// Query items by PK and SK prefix
    #[allow(dead_code)]
    pub async fn query_items(
        &self,
        pk: &str,
        sk_prefix: &str,
    ) -> Vec<HashMap<String, AttributeValue>> {
        self.client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk)")
            .expression_attribute_values(":pk", AttributeValue::S(pk.to_string()))
            .expression_attribute_values(":sk", AttributeValue::S(sk_prefix.to_string()))
            .send()
            .await
            .expect("Failed to query items")
            .items
            .unwrap_or_default()
    }
}

impl Drop for DynamoDbLocalInstance {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

fn find_available_port() -> Result<u16, String> {
    TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to find available port: {e}"))?
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|e| format!("Failed to get port from listener: {e}"))
}

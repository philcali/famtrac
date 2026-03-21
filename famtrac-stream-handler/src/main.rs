pub mod classify;
pub mod dynamo_util;
pub mod handlers;
pub mod parser;
pub mod router;

use aws_lambda_events::event::dynamodb::Event as DynamoDbEvent;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;

use classify::{classify_record, RecordChange};

/// A single failed record identifier for `ReportBatchItemFailures`.
#[derive(Debug, Serialize)]
pub struct BatchItemFailure {
    #[serde(rename = "itemIdentifier")]
    pub item_identifier: String,
}

/// Response returned by the stream handler, listing only the records that failed.
/// Lambda's `ReportBatchItemFailures` feature uses this to retry only failed records.
#[derive(Debug, Serialize)]
pub struct StreamHandlerResponse {
    #[serde(rename = "batchItemFailures")]
    pub batch_item_failures: Vec<BatchItemFailure>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = Client::new(&config);
    let table_name = std::env::var("TABLE_NAME").unwrap_or_else(|_| "FamtracData".to_string());

    let handler = service_fn(move |event: LambdaEvent<DynamoDbEvent>| {
        let client = client.clone();
        let table_name = table_name.clone();
        async move { handle_stream_event(event, &client, &table_name).await }
    });
    lambda_runtime::run(handler).await?;
    Ok(())
}

/// Process a single classified record change, returning `Ok(())` on success or
/// an error if the operation failed.
async fn process_record(
    record: &aws_lambda_events::event::dynamodb::EventRecord,
    client: &Client,
    table_name: &str,
) -> Result<(), Error> {
    match classify_record(record) {
        RecordChange::ShareActivated(share) => {
            handlers::mirror::handle_share_activated(client, table_name, &share, "").await?;
        }
        RecordChange::ShareRevoked {
            share_id,
            family_id,
            accepter_id,
        } => {
            handlers::revoke::handle_share_revoked(
                client,
                table_name,
                &share_id,
                &family_id,
                &accepter_id,
            )
            .await?;
        }
        RecordChange::SharePermissionUpdated(share) => {
            handlers::permission::handle_permission_updated(client, table_name, &share).await?;
        }
        RecordChange::ResourceChanged(change) => {
            handlers::propagate::handle_resource_changed(client, table_name, &change, "").await?;
        }
        RecordChange::Ignored => {}
    }
    Ok(())
}

/// Main stream event handler — classifies each record and dispatches to the appropriate action.
/// Returns a `StreamHandlerResponse` with `batchItemFailures` listing only the event IDs of
/// records that failed processing. Successfully processed records are not retried.
async fn handle_stream_event(
    event: LambdaEvent<DynamoDbEvent>,
    client: &Client,
    table_name: &str,
) -> Result<StreamHandlerResponse, Error> {
    let dynamo_event = event.payload;
    let mut batch_item_failures = Vec::new();

    for record in &dynamo_event.records {
        if let Err(err) = process_record(record, client, table_name).await {
            eprintln!("Failed to process record {}: {:?}", record.event_id, err);
            batch_item_failures.push(BatchItemFailure {
                item_identifier: record.event_id.clone(),
            });
        }
    }

    Ok(StreamHandlerResponse {
        batch_item_failures,
    })
}

// ---------------------------------------------------------------------------
// Stream record classification — moved to classify.rs
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Tests for ReportBatchItemFailures response
    // -----------------------------------------------------------------------

    #[test]
    fn test_stream_handler_response_empty_serialization() {
        let response = StreamHandlerResponse {
            batch_item_failures: vec![],
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["batchItemFailures"], serde_json::json!([]));
    }

    #[test]
    fn test_stream_handler_response_with_failures_serialization() {
        let response = StreamHandlerResponse {
            batch_item_failures: vec![
                BatchItemFailure {
                    item_identifier: "event-id-1".to_string(),
                },
                BatchItemFailure {
                    item_identifier: "event-id-2".to_string(),
                },
            ],
        };
        let json = serde_json::to_value(&response).unwrap();
        let failures = json["batchItemFailures"].as_array().unwrap();
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0]["itemIdentifier"], "event-id-1");
        assert_eq!(failures[1]["itemIdentifier"], "event-id-2");
    }

    #[test]
    fn test_batch_item_failure_serialization() {
        let failure = BatchItemFailure {
            item_identifier: "test-event-123".to_string(),
        };
        let json = serde_json::to_value(&failure).unwrap();
        assert_eq!(json["itemIdentifier"], "test-event-123");
        // Ensure the field name is camelCase as Lambda expects
        assert!(json.get("item_identifier").is_none());
    }
}

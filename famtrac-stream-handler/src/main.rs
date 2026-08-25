use famtrac_stream_handler::classify::{classify_record, ChangeKind, RecordChange};
use famtrac_stream_handler::handlers;
use famtrac_stream_handler::router::Router;

use aws_lambda_events::event::dynamodb::Event as DynamoDbEvent;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use std::sync::Arc;

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

/// Build the Router with all handler registrations.
fn build_router() -> Router {
    let mut router = Router::new();

    // Share activation → mirror resources into accepter's partition
    router.register(
        ChangeKind::ShareActivated,
        Box::new(|client, table_name, change, sync_token| {
            Box::pin(async move {
                if let RecordChange::ShareActivated(ref share) = *change {
                    handlers::mirror::handle_share_activated(
                        &client,
                        &table_name,
                        share,
                        &sync_token,
                    )
                    .await?;
                }
                Ok(())
            })
        }),
    );

    // Share revocation → delete all mirrored records for the share
    router.register(
        ChangeKind::ShareRevoked,
        Box::new(|client, table_name, change, _sync_token| {
            Box::pin(async move {
                if let RecordChange::ShareRevoked {
                    ref share_id,
                    ref family_id,
                    ref accepter_id,
                } = *change
                {
                    handlers::revoke::handle_share_revoked(
                        &client,
                        &table_name,
                        share_id,
                        family_id,
                        accepter_id,
                    )
                    .await?;
                }
                Ok(())
            })
        }),
    );

    // Permission scope updated → update mirrored records
    router.register(
        ChangeKind::SharePermissionUpdated,
        Box::new(|client, table_name, change, _sync_token| {
            Box::pin(async move {
                if let RecordChange::SharePermissionUpdated(ref share) = *change {
                    handlers::permission::handle_permission_updated(&client, &table_name, share)
                        .await?;
                }
                Ok(())
            })
        }),
    );

    // Resource changed → propagate to mirrors or write back to owner
    router.register(
        ChangeKind::ResourceChanged,
        Box::new(|client, table_name, change, sync_token| {
            Box::pin(async move {
                if let RecordChange::ResourceChanged(ref rc) = *change {
                    handlers::propagate::handle_resource_changed(
                        &client,
                        &table_name,
                        rc,
                        &sync_token,
                    )
                    .await?;
                }
                Ok(())
            })
        }),
    );

    router
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = Arc::new(Client::new(&config));
    let table_name = std::env::var("TABLE_NAME").unwrap_or_else(|_| "FamtracData".to_string());
    let router = Arc::new(build_router());

    let handler = service_fn(move |event: LambdaEvent<DynamoDbEvent>| {
        let client = Arc::clone(&client);
        let table_name = table_name.clone();
        let router = Arc::clone(&router);
        async move { handle_stream_event(event, client, &table_name, &router).await }
    });
    lambda_runtime::run(handler).await?;
    Ok(())
}

/// Main stream event handler — classifies each record and dispatches through
/// the Router. Returns a `StreamHandlerResponse` with `batchItemFailures`
/// listing only the event IDs of records that failed processing.
async fn handle_stream_event(
    event: LambdaEvent<DynamoDbEvent>,
    client: Arc<Client>,
    table_name: &str,
    router: &Router,
) -> Result<StreamHandlerResponse, Error> {
    // Generate sync_token from the Lambda request ID (unique per invocation)
    let sync_token = event.context.request_id.clone();
    let dynamo_event = event.payload;
    let mut batch_item_failures = Vec::new();

    for record in &dynamo_event.records {
        let change = classify_record(record);
        if let Err(err) = router
            .dispatch(Arc::clone(&client), table_name, change, &sync_token)
            .await
        {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(json.get("item_identifier").is_none());
    }

    #[test]
    fn test_build_router_registers_all_handlers() {
        let router = build_router();
        // Verify the router has handlers for all 4 change kinds
        assert!(router.supported_change_kinds().len() == 4);
    }
}

pub mod classify;
pub mod dynamo_util;
pub mod parser;
pub mod router;

use aws_lambda_events::event::dynamodb::Event as DynamoDbEvent;
use aws_sdk_dynamodb::types::AttributeValue as DdbAttributeValue;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;

use famtrac_backend::domain::Share;

use classify::{classify_record, ChangeOperation, RecordChange, ResourceChange};
use dynamo_util::{
    annotate_item, conditional_put, conditional_update_permission, delete_item, extract_family_id,
    extract_owner_id, get_item, is_mirrored_resource, put_item, query_items, rekey_item,
};
use parser::parse_share_from_attrs;

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
            mirror_resources(client, table_name, &share).await?;
        }
        RecordChange::ShareRevoked { .. } => {
            // TODO: cleanup_mirrored will be implemented in handlers/revoke.rs
        }
        RecordChange::SharePermissionUpdated(share) => {
            update_mirrored_permissions(client, table_name, &share).await?;
        }
        RecordChange::ResourceChanged(change) => {
            propagate_change(client, table_name, &change).await?;
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
// Mirror resources on share activation
// ---------------------------------------------------------------------------

/// Mirror the shared family, all its dependents, and all their activities into
/// the accepter's partition. Each mirrored record is annotated with the share's
/// `share_id` and `permission_scope`. Conditional writes (`attribute_not_exists`)
/// ensure idempotency — duplicate stream deliveries are safely ignored.
async fn mirror_resources(client: &Client, table_name: &str, share: &Share) -> Result<(), Error> {
    let accepter_id = match &share.accepter_id {
        Some(id) => id,
        None => return Ok(()), // No accepter identity yet — nothing to mirror
    };

    let share_id_str = share.id.0.to_string();
    let scope_json =
        serde_json::to_string(&share.permission_scope).unwrap_or_else(|_| "{}".to_string());

    // 1. Fetch the original family record
    let family_item = get_item(
        client,
        table_name,
        &format!("OWNER#{}", share.requester_id.0),
        &format!("FAMILY#{}", share.family_id.0),
    )
    .await?;

    let family_item = match family_item {
        Some(item) => item,
        None => return Ok(()), // Family not found — nothing to mirror
    };

    // Mirror the family into the accepter's partition with rekeyed PK
    let mirrored_family = rekey_item(
        family_item,
        &format!("OWNER#{}", accepter_id.0),
        &share_id_str,
        &scope_json,
    );
    conditional_put(client, table_name, mirrored_family).await?;

    // 2. Fetch all dependents for this family
    let dependents = query_items(
        client,
        table_name,
        &format!("FAMILY#{}", share.family_id.0),
        "DEPENDENT#",
    )
    .await?;

    for dep_item in &dependents {
        let mirrored_dep = annotate_item(dep_item.clone(), &share_id_str, &scope_json);
        conditional_put(client, table_name, mirrored_dep).await?;
    }

    // 3. For each dependent, fetch and mirror all activities
    for dep_item in &dependents {
        let dep_id = match dep_item.get("id").and_then(|v| v.as_s().ok()) {
            Some(id) => id.clone(),
            None => continue,
        };

        let activities = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
            "ACTIVITY#",
        )
        .await?;

        for act_item in activities {
            let mirrored_act = annotate_item(act_item, &share_id_str, &scope_json);
            conditional_put(client, table_name, mirrored_act).await?;
        }
    }

    Ok(())
}

/// Mirror the shared family, all its dependents, and all their activities into
/// share records. Returns shares where `status = active`.
async fn find_active_shares_for_family(
    client: &Client,
    table_name: &str,
    owner_id: &str,
    family_id: &str,
) -> Result<Vec<Share>, Error> {
    let result = client
        .query()
        .table_name(table_name)
        .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
        .filter_expression("family_id = :fid AND #st = :active")
        .expression_attribute_names("#st", "status")
        .expression_attribute_values(":pk", DdbAttributeValue::S(format!("OWNER#{}", owner_id)))
        .expression_attribute_values(":sk_prefix", DdbAttributeValue::S("SHARE#".to_string()))
        .expression_attribute_values(":fid", DdbAttributeValue::S(family_id.to_string()))
        .expression_attribute_values(":active", DdbAttributeValue::S("active".to_string()))
        .send()
        .await?;

    let items = result.items.unwrap_or_default();
    let mut shares = Vec::new();
    for item in &items {
        if let Some(share) = parse_share_from_attrs(item) {
            shares.push(share);
        }
    }
    Ok(shares)
}

// ---------------------------------------------------------------------------
// Update mirrored permissions on share permission_scope change
// ---------------------------------------------------------------------------

/// Update the `permission_scope` attribute on all mirrored records that carry
/// the given share's `share_id`. This is triggered when a requester updates
/// the permission scope on an active share.
///
/// Updates:
/// 1. The mirrored Family record in the accepter's `OWNER#` partition
/// 2. All mirrored Dependent records under `FAMILY#{family_id}`
/// 3. All mirrored Activity records under `FAMILY#{family_id}#DEPENDENT#{dep_id}`
///
/// Each update uses a condition expression `share_id = :sid` so that only
/// mirrored copies (not originals) are affected. ConditionalCheckFailedExceptions
/// are silently ignored (the record may not exist or may not be mirrored).
async fn update_mirrored_permissions(
    client: &Client,
    table_name: &str,
    share: &Share,
) -> Result<(), Error> {
    let accepter_id = match &share.accepter_id {
        Some(id) => id,
        None => return Ok(()), // No accepter identity yet — nothing to update
    };

    let share_id_str = share.id.0.to_string();
    let scope_json =
        serde_json::to_string(&share.permission_scope).unwrap_or_else(|_| "{}".to_string());

    // 1. Update the mirrored Family record in the accepter's partition
    conditional_update_permission(
        client,
        table_name,
        &format!("OWNER#{}", accepter_id.0),
        &format!("FAMILY#{}", share.family_id.0),
        &share_id_str,
        &scope_json,
    )
    .await?;

    // 2. Query all Dependents for this family and update mirrored ones
    let dependents = query_items(
        client,
        table_name,
        &format!("FAMILY#{}", share.family_id.0),
        "DEPENDENT#",
    )
    .await?;

    for dep_item in &dependents {
        let dep_sk = match dep_item.get("SK").and_then(|v| v.as_s().ok()) {
            Some(sk) => sk.clone(),
            None => continue,
        };

        conditional_update_permission(
            client,
            table_name,
            &format!("FAMILY#{}", share.family_id.0),
            &dep_sk,
            &share_id_str,
            &scope_json,
        )
        .await?;

        // 3. For each dependent, query and update mirrored Activities
        let dep_id = match dep_item.get("id").and_then(|v| v.as_s().ok()) {
            Some(id) => id.clone(),
            None => continue,
        };

        let activities = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
            "ACTIVITY#",
        )
        .await?;

        for act_item in &activities {
            let act_sk = match act_item.get("SK").and_then(|v| v.as_s().ok()) {
                Some(sk) => sk.clone(),
                None => continue,
            };

            conditional_update_permission(
                client,
                table_name,
                &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
                &act_sk,
                &share_id_str,
                &scope_json,
            )
            .await?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Propagate resource changes to mirrored copies
// ---------------------------------------------------------------------------

/// Propagate a resource change (create/update/delete) to all mirrored copies.
///
/// Two cases:
/// 1. **Owned resource changed**: Find all active shares for the family, propagate
///    the change to each accepter's mirrored copy.
/// 2. **Mirrored resource changed (write-back)**: The change was made on a mirrored
///    copy by an accepter — propagate back to the original owner's partition.
async fn propagate_change(
    client: &Client,
    table_name: &str,
    change: &ResourceChange,
) -> Result<(), Error> {
    // Determine which image to inspect (new for insert/modify, old for remove)
    let image = match change.operation {
        ChangeOperation::Remove => &change.old_image,
        _ => &change.new_image,
    };

    // If this is a mirrored resource, propagate back to the original (write-back)
    if is_mirrored_resource(image) {
        return propagate_writeback(client, table_name, change).await;
    }

    // This is an owned resource — propagate to all active share mirrors
    propagate_to_mirrors(client, table_name, change).await
}

/// Propagate an owned resource change to all mirrored copies in accepter partitions.
async fn propagate_to_mirrors(
    client: &Client,
    table_name: &str,
    change: &ResourceChange,
) -> Result<(), Error> {
    let image = match change.operation {
        ChangeOperation::Remove => &change.old_image,
        _ => &change.new_image,
    };

    let family_id = match extract_family_id(&change.pk, &change.sk, image) {
        Some(fid) => fid,
        None => return Ok(()), // Can't determine family — skip
    };

    let owner_id = match extract_owner_id(&change.pk, image) {
        Some(oid) => oid,
        None => {
            // For Dependent/Activity records, we may need to look up the family
            // to find the owner. Query the family record.
            match find_owner_for_family(client, table_name, &family_id).await? {
                Some(oid) => oid,
                None => return Ok(()),
            }
        }
    };

    let active_shares =
        find_active_shares_for_family(client, table_name, &owner_id, &family_id).await?;

    if active_shares.is_empty() {
        return Ok(());
    }

    let is_family_record = change.sk.starts_with("FAMILY#");

    for share in &active_shares {
        let accepter_id = match &share.accepter_id {
            Some(id) => &id.0,
            None => continue, // No accepter identity — skip
        };

        let share_id_str = share.id.0.to_string();
        let scope_json =
            serde_json::to_string(&share.permission_scope).unwrap_or_else(|_| "{}".to_string());

        match change.operation {
            ChangeOperation::Insert | ChangeOperation::Modify => {
                if is_family_record {
                    // Family records are rekeyed into accepter's OWNER partition
                    let mirrored = rekey_item(
                        change.new_image.clone(),
                        &format!("OWNER#{}", accepter_id),
                        &share_id_str,
                        &scope_json,
                    );
                    put_item(client, table_name, mirrored).await?;
                } else {
                    // Dependent/Activity records keep the same PK/SK.
                    // The original item is shared, so we don't need to create
                    // separate copies. The share metadata is on the mirrored
                    // Family record, and permission checks use that context.
                    // Nothing to do here — the original item IS the shared item.
                }
            }
            ChangeOperation::Remove => {
                if is_family_record {
                    // Delete the mirrored Family from the accepter's partition
                    delete_item(
                        client,
                        table_name,
                        &format!("OWNER#{}", accepter_id),
                        &change.sk,
                    )
                    .await?;
                }
                // For Dependent/Activity removes: the original item is deleted,
                // which is the same item the accepter sees. No extra cleanup needed.
            }
        }
    }

    Ok(())
}

/// Find the owner of a family by scanning for the Family record.
/// This is needed when processing Dependent/Activity changes where the PK
/// doesn't contain the owner_id.
async fn find_owner_for_family(
    client: &Client,
    table_name: &str,
    family_id: &str,
) -> Result<Option<String>, Error> {
    // We need to find which OWNER# partition has a FAMILY#{family_id} record.
    // Use a GSI or scan — but since we don't have a GSI for this, we'll use
    // a query on the family_id. The Family record has PK=OWNER#{owner_id} and
    // SK=FAMILY#{family_id}. We can't query by SK alone without a GSI.
    //
    // Alternative: look at the DynamoDB stream record's old/new image for
    // an owner_id attribute. If not present, we need to scan.
    //
    // For now, use a scan with a filter — this is acceptable for the stream
    // handler since it processes events asynchronously.
    let result = client
        .scan()
        .table_name(table_name)
        .filter_expression("SK = :sk AND begins_with(PK, :pk_prefix)")
        .expression_attribute_values(":sk", DdbAttributeValue::S(format!("FAMILY#{}", family_id)))
        .expression_attribute_values(":pk_prefix", DdbAttributeValue::S("OWNER#".to_string()))
        .limit(1)
        .send()
        .await?;

    if let Some(items) = result.items {
        if let Some(item) = items.first() {
            if let Some(pk) = item.get("PK").and_then(|v| v.as_s().ok()) {
                return Ok(pk.strip_prefix("OWNER#").map(|s| s.to_string()));
            }
        }
    }

    Ok(None)
}

/// Propagate a write-back from a mirrored resource to the original owner's partition.
///
/// When an accepter modifies a mirrored resource (e.g., creates an Activity),
/// the stream handler detects the `share_id` on the record and propagates the
/// change back to the original family's partition.
async fn propagate_writeback(
    client: &Client,
    table_name: &str,
    change: &ResourceChange,
) -> Result<(), Error> {
    let image = match change.operation {
        ChangeOperation::Remove => &change.old_image,
        _ => &change.new_image,
    };

    let is_family_record = change.sk.starts_with("FAMILY#");

    if is_family_record {
        // Mirrored Family records have PK=OWNER#{accepter_id}.
        // We need to find the original owner and write back to their partition.
        let share_id_str = match image.get("share_id").and_then(|v| v.as_s().ok()) {
            Some(s) => s.clone(),
            None => return Ok(()),
        };

        let family_id = match extract_family_id(&change.pk, &change.sk, image) {
            Some(fid) => fid,
            None => return Ok(()),
        };

        // Look up the share to find the original owner
        let _share_id: uuid::Uuid = match share_id_str.parse() {
            Ok(id) => id,
            Err(_) => return Ok(()),
        };

        // Find the original owner by looking up the family in non-mirrored partitions
        let original_owner = match find_owner_for_family(client, table_name, &family_id).await? {
            Some(oid) => oid,
            None => return Ok(()),
        };

        match change.operation {
            ChangeOperation::Insert | ChangeOperation::Modify => {
                // Write back to the original owner's partition, stripping share metadata
                let mut original_item = change.new_image.clone();
                original_item.insert(
                    "PK".to_string(),
                    DdbAttributeValue::S(format!("OWNER#{}", original_owner)),
                );
                original_item.remove("share_id");
                original_item.remove("permission_scope");
                put_item(client, table_name, original_item).await?;
            }
            ChangeOperation::Remove => {
                delete_item(
                    client,
                    table_name,
                    &format!("OWNER#{}", original_owner),
                    &change.sk,
                )
                .await?;
            }
        }
    } else {
        // Dependent/Activity mirrored records share the same PK/SK as originals.
        // A write-back on these means the accepter modified the shared item directly.
        // The change is already on the original item — no extra propagation needed.
        // However, we should ensure the share metadata is preserved on the item.
        if change.operation == ChangeOperation::Insert
            || change.operation == ChangeOperation::Modify
        {
            // Re-annotate the item with share metadata if it was stripped
            let share_id_str = match image.get("share_id").and_then(|v| v.as_s().ok()) {
                Some(s) => s.clone(),
                None => return Ok(()),
            };
            let scope_json = match image.get("permission_scope").and_then(|v| v.as_s().ok()) {
                Some(s) => s.clone(),
                None => return Ok(()),
            };

            // The item already has the correct PK/SK. Just ensure share metadata is present.
            let mut item = change.new_image.clone();
            item.insert("share_id".to_string(), DdbAttributeValue::S(share_id_str));
            item.insert(
                "permission_scope".to_string(),
                DdbAttributeValue::S(scope_json),
            );
            put_item(client, table_name, item).await?;
        }
    }

    Ok(())
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

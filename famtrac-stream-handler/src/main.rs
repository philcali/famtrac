use aws_lambda_events::event::dynamodb::{Event as DynamoDbEvent, EventRecord, OperationType};
use aws_sdk_dynamodb::types::AttributeValue as DdbAttributeValue;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use std::collections::HashMap;

use famtrac_backend::domain::{
    FamilyId, IdentityId, PermissionScope, Share, ShareId, ShareStatus, Timestamp,
};

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

/// The type of DynamoDB Stream operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeOperation {
    Insert,
    Modify,
    Remove,
}

/// Context for a resource change event (Family, Dependent, or Activity).
#[derive(Debug)]
pub struct ResourceChange {
    /// The PK of the changed record.
    pub pk: String,
    /// The SK of the changed record.
    pub sk: String,
    /// The type of change.
    pub operation: ChangeOperation,
    /// The new image of the record (empty for REMOVE).
    pub new_image: HashMap<String, DdbAttributeValue>,
    /// The old image of the record (empty for INSERT).
    pub old_image: HashMap<String, DdbAttributeValue>,
}

/// Classification of a DynamoDB Stream record change.
#[derive(Debug)]
pub enum RecordChange {
    /// A share transitioned to active status — mirror resources into accepter's partition.
    ShareActivated(Share),
    /// A share was deleted — clean up mirrored records.
    ShareRevoked(ShareId),
    /// A share's permission scope was updated — update mirrored record metadata.
    SharePermissionUpdated(Share),
    /// A family/dependent/activity was created, updated, or deleted — propagate to mirrored copies.
    ResourceChanged(ResourceChange),
    /// Record change is not relevant to stream processing.
    Ignored,
}

// TODO(future-spec): Refactor the stream handler into a "classify and route" architecture.
// The current monolith classifies DynamoDB Stream records and processes them inline. Instead,
// extract classification into a router that dispatches to composable handler functions
// (e.g. share_activated, resource_changed, write_audit_entry). This would let multiple
// concerns (mirroring, audit logging, notifications, etc.) subscribe to the same stream
// events without growing this file further.

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
    record: &EventRecord,
    client: &Client,
    table_name: &str,
) -> Result<(), Error> {
    match classify_record(record) {
        RecordChange::ShareActivated(share) => {
            mirror_resources(client, table_name, &share).await?;
        }
        RecordChange::ShareRevoked(_share_id) => {
            // TODO (task 13.8): cleanup_mirrored(share_id).await?
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

/// Fetch a single item by PK and SK.
async fn get_item(
    client: &Client,
    table_name: &str,
    pk: &str,
    sk: &str,
) -> Result<Option<HashMap<String, DdbAttributeValue>>, Error> {
    let result = client
        .get_item()
        .table_name(table_name)
        .key("PK", DdbAttributeValue::S(pk.to_string()))
        .key("SK", DdbAttributeValue::S(sk.to_string()))
        .send()
        .await?;

    Ok(result.item)
}

/// Query all items matching a PK and SK prefix.
async fn query_items(
    client: &Client,
    table_name: &str,
    pk: &str,
    sk_prefix: &str,
) -> Result<Vec<HashMap<String, DdbAttributeValue>>, Error> {
    let result = client
        .query()
        .table_name(table_name)
        .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
        .expression_attribute_values(":pk", DdbAttributeValue::S(pk.to_string()))
        .expression_attribute_values(":sk_prefix", DdbAttributeValue::S(sk_prefix.to_string()))
        .send()
        .await?;

    Ok(result.items.unwrap_or_default())
}

/// Rekey an item's PK to a new value and annotate with share metadata.
/// Used for Family records that need to move into the accepter's OWNER partition.
fn rekey_item(
    mut item: HashMap<String, DdbAttributeValue>,
    new_pk: &str,
    share_id: &str,
    scope_json: &str,
) -> HashMap<String, DdbAttributeValue> {
    item.insert("PK".to_string(), DdbAttributeValue::S(new_pk.to_string()));
    item.insert(
        "share_id".to_string(),
        DdbAttributeValue::S(share_id.to_string()),
    );
    item.insert(
        "permission_scope".to_string(),
        DdbAttributeValue::S(scope_json.to_string()),
    );
    item
}

/// Annotate an item with share metadata without changing its PK/SK.
/// Used for Dependent and Activity records that keep their original keys.
fn annotate_item(
    mut item: HashMap<String, DdbAttributeValue>,
    share_id: &str,
    scope_json: &str,
) -> HashMap<String, DdbAttributeValue> {
    item.insert(
        "share_id".to_string(),
        DdbAttributeValue::S(share_id.to_string()),
    );
    item.insert(
        "permission_scope".to_string(),
        DdbAttributeValue::S(scope_json.to_string()),
    );
    item
}

/// Put an item with a condition that it doesn't already exist (idempotent write).
/// If the item already exists, the ConditionalCheckFailedException is silently ignored.
async fn conditional_put(
    client: &Client,
    table_name: &str,
    item: HashMap<String, DdbAttributeValue>,
) -> Result<(), Error> {
    let result = client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .condition_expression("attribute_not_exists(PK) AND attribute_not_exists(SK)")
        .send()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            // ConditionalCheckFailedException means the item already exists — that's fine
            // for idempotent mirroring.
            if err
                .as_service_error()
                .map(|e| e.is_conditional_check_failed_exception())
                .unwrap_or(false)
            {
                Ok(())
            } else {
                Err(Box::new(err))
            }
        }
    }
}

/// Put an item, overwriting any existing item at the same PK/SK.
async fn put_item(
    client: &Client,
    table_name: &str,
    item: HashMap<String, DdbAttributeValue>,
) -> Result<(), Error> {
    client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await?;
    Ok(())
}

/// Delete an item by PK and SK.
async fn delete_item(client: &Client, table_name: &str, pk: &str, sk: &str) -> Result<(), Error> {
    client
        .delete_item()
        .table_name(table_name)
        .key("PK", DdbAttributeValue::S(pk.to_string()))
        .key("SK", DdbAttributeValue::S(sk.to_string()))
        .send()
        .await?;
    Ok(())
}

/// Query all active share records for a given family by scanning the owner's
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
        if let Some(share) = parse_share_from_ddb_item(item) {
            shares.push(share);
        }
    }
    Ok(shares)
}

/// Parse a `Share` from a raw DynamoDB `HashMap<String, DdbAttributeValue>`.
fn parse_share_from_ddb_item(item: &HashMap<String, DdbAttributeValue>) -> Option<Share> {
    let get = |key: &str| -> Option<String> { item.get(key).and_then(|v| v.as_s().ok()).cloned() };

    let id = get("id")?.parse().ok()?;
    let family_id = get("family_id")?.parse().ok()?;
    let requester_id = get("requester_id")?;
    let accepter_email = get("accepter_email")?;
    let accepter_id = get("accepter_id").map(IdentityId);

    let scope_json = get("permission_scope")?;
    let permission_scope: PermissionScope = serde_json::from_str(&scope_json).ok()?;

    let status_str = get("status")?;
    let status: ShareStatus = serde_json::from_str(&format!("\"{}\"", status_str)).ok()?;

    let created_at = get("created_at")?
        .parse::<chrono::DateTime<chrono::Utc>>()
        .ok()?;
    let updated_at = get("updated_at")?
        .parse::<chrono::DateTime<chrono::Utc>>()
        .ok()?;
    let expires_at = get("expires_at")
        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .map(Timestamp::from_datetime);

    Some(Share {
        id: ShareId(id),
        family_id: FamilyId(family_id),
        requester_id: IdentityId(requester_id),
        accepter_email,
        accepter_id,
        permission_scope,
        status,
        created_at: Timestamp::from_datetime(created_at),
        updated_at: Timestamp::from_datetime(updated_at),
        expires_at,
    })
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

/// Update the `permission_scope` attribute on a single item, conditioned on
/// `share_id = :sid`. Silently ignores ConditionalCheckFailedException (the
/// item may not exist or may not be a mirrored copy for this share).
async fn conditional_update_permission(
    client: &Client,
    table_name: &str,
    pk: &str,
    sk: &str,
    share_id: &str,
    scope_json: &str,
) -> Result<(), Error> {
    let result = client
        .update_item()
        .table_name(table_name)
        .key("PK", DdbAttributeValue::S(pk.to_string()))
        .key("SK", DdbAttributeValue::S(sk.to_string()))
        .update_expression("SET permission_scope = :new_scope")
        .condition_expression("share_id = :sid")
        .expression_attribute_values(":sid", DdbAttributeValue::S(share_id.to_string()))
        .expression_attribute_values(":new_scope", DdbAttributeValue::S(scope_json.to_string()))
        .send()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            if err
                .as_service_error()
                .map(|e| e.is_conditional_check_failed_exception())
                .unwrap_or(false)
            {
                Ok(()) // Item doesn't exist or isn't mirrored for this share — fine
            } else {
                Err(Box::new(err))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Propagate resource changes to mirrored copies
// ---------------------------------------------------------------------------

/// Extract the family_id from a resource's DynamoDB image or PK.
///
/// For Family records: read the `id` attribute (the family_id IS the record id).
/// For Dependent records: read the `family_id` attribute, or parse from PK `FAMILY#{fid}`.
/// For Activity records: read the `family_id` attribute, or parse from PK `FAMILY#{fid}#DEPENDENT#{did}`.
fn extract_family_id(
    pk: &str,
    sk: &str,
    image: &HashMap<String, DdbAttributeValue>,
) -> Option<String> {
    // Try the image's family_id attribute first (present on Dependent/Activity)
    if let Some(fid) = image.get("family_id").and_then(|v| v.as_s().ok()) {
        return Some(fid.clone());
    }

    // For Family records, the id attribute IS the family_id
    if sk.starts_with("FAMILY#") {
        if let Some(id) = image.get("id").and_then(|v| v.as_s().ok()) {
            return Some(id.clone());
        }
        // Fallback: parse from SK
        return sk.strip_prefix("FAMILY#").map(|s| s.to_string());
    }

    // For Dependent records: PK = FAMILY#{fid}
    if pk.starts_with("FAMILY#") && !pk.contains("#DEPENDENT#") {
        return pk.strip_prefix("FAMILY#").map(|s| s.to_string());
    }

    // For Activity records: PK = FAMILY#{fid}#DEPENDENT#{did}
    if pk.starts_with("FAMILY#") && pk.contains("#DEPENDENT#") {
        let after_family = pk.strip_prefix("FAMILY#")?;
        let fid = after_family.split("#DEPENDENT#").next()?;
        return Some(fid.to_string());
    }

    None
}

/// Extract the owner_id from a resource's DynamoDB image or PK.
/// For Family records: PK = OWNER#{owner_id}, or read `owner_id` attribute.
/// For Dependent/Activity records: read `owner_id` attribute if present, otherwise
/// we need to look up the family to find the owner.
fn extract_owner_id(pk: &str, image: &HashMap<String, DdbAttributeValue>) -> Option<String> {
    // Try the image's owner_id attribute first
    if let Some(oid) = image.get("owner_id").and_then(|v| v.as_s().ok()) {
        return Some(oid.clone());
    }

    // For Family records: PK = OWNER#{owner_id}
    if pk.starts_with("OWNER#") {
        return pk.strip_prefix("OWNER#").map(|s| s.to_string());
    }

    None
}

/// Check if a resource record is a mirrored copy (has a share_id attribute).
fn is_mirrored_resource(image: &HashMap<String, DdbAttributeValue>) -> bool {
    image.get("share_id").and_then(|v| v.as_s().ok()).is_some()
}

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

/// Convert a `serde_dynamo::Item` to a `HashMap<String, DdbAttributeValue>` for use
/// with the AWS SDK DynamoDB client.
fn convert_image(image: &serde_dynamo::Item) -> HashMap<String, DdbAttributeValue> {
    image
        .inner()
        .iter()
        .filter_map(|(k, v)| {
            let ddb_val = match v {
                serde_dynamo::AttributeValue::S(s) => Some(DdbAttributeValue::S(s.clone())),
                serde_dynamo::AttributeValue::N(n) => Some(DdbAttributeValue::N(n.clone())),
                serde_dynamo::AttributeValue::Bool(b) => Some(DdbAttributeValue::Bool(*b)),
                serde_dynamo::AttributeValue::Null(true) => Some(DdbAttributeValue::Null(true)),
                _ => None, // Other types not used in this schema
            };
            ddb_val.map(|v| (k.clone(), v))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Stream record classification
// ---------------------------------------------------------------------------

/// Extract a string value from a `serde_dynamo::Item` by attribute name.
fn get_str(item: &serde_dynamo::Item, key: &str) -> Option<String> {
    match item.inner().get(key)? {
        serde_dynamo::AttributeValue::S(s) => Some(s.clone()),
        _ => None,
    }
}

/// Attempt to parse a `Share` from a `serde_dynamo::Item` (the new or old image).
fn parse_share_from_image(image: &serde_dynamo::Item) -> Option<Share> {
    let id = get_str(image, "id")?.parse().ok()?;
    let family_id = get_str(image, "family_id")?.parse().ok()?;
    let requester_id = get_str(image, "requester_id")?;
    let accepter_email = get_str(image, "accepter_email")?;
    let accepter_id = get_str(image, "accepter_id").map(IdentityId);

    let scope_json = get_str(image, "permission_scope")?;
    let permission_scope: PermissionScope = serde_json::from_str(&scope_json).ok()?;

    let status_str = get_str(image, "status")?;
    let status: ShareStatus = serde_json::from_str(&format!("\"{}\"", status_str)).ok()?;

    let created_at = get_str(image, "created_at")?
        .parse::<chrono::DateTime<chrono::Utc>>()
        .ok()?;
    let updated_at = get_str(image, "updated_at")?
        .parse::<chrono::DateTime<chrono::Utc>>()
        .ok()?;
    let expires_at = get_str(image, "expires_at")
        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .map(Timestamp::from_datetime);

    Some(Share {
        id: ShareId(id),
        family_id: FamilyId(family_id),
        requester_id: IdentityId(requester_id),
        accepter_email,
        accepter_id,
        permission_scope,
        status,
        created_at: Timestamp::from_datetime(created_at),
        updated_at: Timestamp::from_datetime(updated_at),
        expires_at,
    })
}

/// Determine the record type from the SK prefix.
/// Returns `"Share"` for share records, `"Family"` / `"Dependent"` / `"Activity"` for
/// resource records, or `None` for anything we don't care about.
fn record_type_from_sk(sk: &str) -> Option<&'static str> {
    if sk.starts_with("SHARE#") {
        Some("Share")
    } else if sk.starts_with("FAMILY#") {
        Some("Family")
    } else if sk.starts_with("DEPENDENT#") {
        Some("Dependent")
    } else if sk.starts_with("ACTIVITY#") {
        Some("Activity")
    } else {
        None
    }
}

/// Determine whether a PK belongs to the owner partition (as opposed to the
/// email-index partition `SHARE_EMAIL#...`). We only process owner-partition
/// share records to avoid double-processing.
fn is_owner_partition(pk: &str) -> bool {
    pk.starts_with("OWNER#")
}

/// Classify a single DynamoDB Stream record into a `RecordChange` variant.
///
/// Classification rules:
/// - Share records (SK starts with `SHARE#`) in the owner partition:
///   - INSERT/MODIFY where new status is `active` and old status was not `active`
///     → `ShareActivated`
///   - MODIFY where `permission_scope` changed but status did not transition to active
///     → `SharePermissionUpdated`
///   - REMOVE → `ShareRevoked`
///   - Everything else (e.g. pending creation) → `Ignored`
/// - Resource records (Family / Dependent / Activity):
///   - Any INSERT / MODIFY / REMOVE → `ResourceChanged`
/// - Anything else → `Ignored`
pub fn classify_record(record: &EventRecord) -> RecordChange {
    let keys = &record.change.keys;

    // Extract PK and SK from the key attributes.
    let pk = match get_str(keys, "PK") {
        Some(v) => v,
        None => return RecordChange::Ignored,
    };
    let sk = match get_str(keys, "SK") {
        Some(v) => v,
        None => return RecordChange::Ignored,
    };

    let record_type = match record_type_from_sk(&sk) {
        Some(t) => t,
        None => return RecordChange::Ignored,
    };

    // Determine the operation type from the event_name field.
    let op = match record.event_name.as_str() {
        "INSERT" => OperationType::Insert,
        "MODIFY" => OperationType::Modify,
        "REMOVE" => OperationType::Remove,
        _ => return RecordChange::Ignored,
    };

    match record_type {
        "Share" => classify_share_record(&pk, &sk, &op, record),
        "Family" | "Dependent" | "Activity" => {
            let change_op = match op {
                OperationType::Insert => ChangeOperation::Insert,
                OperationType::Modify => ChangeOperation::Modify,
                OperationType::Remove => ChangeOperation::Remove,
            };
            let new_image = convert_image(&record.change.new_image);
            let old_image = convert_image(&record.change.old_image);
            RecordChange::ResourceChanged(ResourceChange {
                pk,
                sk,
                operation: change_op,
                new_image,
                old_image,
            })
        }
        _ => RecordChange::Ignored,
    }
}

/// Classify a share-specific stream record.
fn classify_share_record(
    pk: &str,
    sk: &str,
    op: &OperationType,
    record: &EventRecord,
) -> RecordChange {
    // Only process share records from the owner partition to avoid
    // double-processing the email-index copy.
    if !is_owner_partition(pk) {
        return RecordChange::Ignored;
    }

    match op {
        OperationType::Remove => {
            // Extract the share_id from the SK: "SHARE#{uuid}"
            let share_id_str = sk.strip_prefix("SHARE#").unwrap_or("");
            match share_id_str.parse::<uuid::Uuid>() {
                Ok(id) => RecordChange::ShareRevoked(ShareId(id)),
                Err(_) => RecordChange::Ignored,
            }
        }
        OperationType::Insert | OperationType::Modify => {
            let new_image = &record.change.new_image;
            let new_share = match parse_share_from_image(new_image) {
                Some(s) => s,
                None => return RecordChange::Ignored,
            };

            let new_status = &new_share.status;

            // Check if this is an activation (status transitioned to Active).
            if *new_status == ShareStatus::Active {
                let old_was_active = get_str(&record.change.old_image, "status")
                    .map(|s| s == "active")
                    .unwrap_or(false);

                if !old_was_active {
                    return RecordChange::ShareActivated(new_share);
                }
            }

            // Check if permission_scope changed (but not an activation).
            if *op == OperationType::Modify {
                let old_scope = get_str(&record.change.old_image, "permission_scope");
                let new_scope = get_str(new_image, "permission_scope");
                if old_scope != new_scope {
                    return RecordChange::SharePermissionUpdated(new_share);
                }
            }

            // Other share changes (e.g. pending creation) — not actionable.
            RecordChange::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lambda_events::event::dynamodb::StreamRecord;
    use chrono::Utc;
    use serde_dynamo::AttributeValue;
    use std::collections::HashMap;

    /// Helper: build a minimal `EventRecord` with the given event name, keys,
    /// old image, and new image.
    fn make_event_record(
        event_name: &str,
        keys: HashMap<String, AttributeValue>,
        old_image: HashMap<String, AttributeValue>,
        new_image: HashMap<String, AttributeValue>,
    ) -> EventRecord {
        EventRecord {
            aws_region: "us-east-1".to_string(),
            change: StreamRecord {
                approximate_creation_date_time: Utc::now(),
                keys: serde_dynamo::Item::from(keys),
                new_image: serde_dynamo::Item::from(new_image),
                old_image: serde_dynamo::Item::from(old_image),
                sequence_number: Some("1".to_string()),
                size_bytes: 0,
                stream_view_type: None,
            },
            event_id: "1".to_string(),
            event_name: event_name.to_string(),
            event_source: Some("aws:dynamodb".to_string()),
            event_version: Some("1.1".to_string()),
            event_source_arn: None,
            user_identity: None,
            record_format: None,
            table_name: None,
        }
    }

    fn s(val: &str) -> AttributeValue {
        AttributeValue::S(val.to_string())
    }

    fn share_keys(requester_id: &str, share_id: &str) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert("PK".to_string(), s(&format!("OWNER#{}", requester_id)));
        m.insert("SK".to_string(), s(&format!("SHARE#{}", share_id)));
        m
    }

    fn share_image(
        share_id: &str,
        family_id: &str,
        requester_id: &str,
        email: &str,
        status: &str,
        scope_json: &str,
    ) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert("id".to_string(), s(share_id));
        m.insert("family_id".to_string(), s(family_id));
        m.insert("requester_id".to_string(), s(requester_id));
        m.insert("accepter_email".to_string(), s(email));
        m.insert("status".to_string(), s(status));
        m.insert("permission_scope".to_string(), s(scope_json));
        m.insert("created_at".to_string(), s("2025-01-01T00:00:00Z"));
        m.insert("updated_at".to_string(), s("2025-01-01T00:00:00Z"));
        m
    }

    const SCOPE_JSON: &str = r#"{"actions":["family_read"]}"#;

    #[test]
    fn test_share_activated_on_insert() {
        let sid = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let new_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", SCOPE_JSON);
        let record = make_event_record("INSERT", keys, HashMap::new(), new_img);

        match classify_record(&record) {
            RecordChange::ShareActivated(share) => {
                assert_eq!(share.status, ShareStatus::Active);
                assert_eq!(share.id.0.to_string(), sid);
            }
            other => panic!("Expected ShareActivated, got {:?}", other),
        }
    }

    #[test]
    fn test_share_activated_on_modify_pending_to_active() {
        let sid = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let old_img = share_image(&sid, &fid, "owner1", "a@b.com", "pending", SCOPE_JSON);
        let new_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", SCOPE_JSON);
        let record = make_event_record("MODIFY", keys, old_img, new_img);

        match classify_record(&record) {
            RecordChange::ShareActivated(share) => {
                assert_eq!(share.status, ShareStatus::Active);
            }
            other => panic!("Expected ShareActivated, got {:?}", other),
        }
    }

    #[test]
    fn test_share_permission_updated() {
        let sid = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let old_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", SCOPE_JSON);
        let new_scope = r#"{"actions":["family_read","dependent_read"]}"#;
        let new_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", new_scope);
        let record = make_event_record("MODIFY", keys, old_img, new_img);

        match classify_record(&record) {
            RecordChange::SharePermissionUpdated(share) => {
                assert_eq!(share.permission_scope.actions.len(), 2);
            }
            other => panic!("Expected SharePermissionUpdated, got {:?}", other),
        }
    }

    #[test]
    fn test_share_revoked() {
        let sid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let old_img = share_image(
            &sid,
            &uuid::Uuid::new_v4().to_string(),
            "owner1",
            "a@b.com",
            "active",
            SCOPE_JSON,
        );
        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());

        match classify_record(&record) {
            RecordChange::ShareRevoked(share_id) => {
                assert_eq!(share_id.0.to_string(), sid);
            }
            other => panic!("Expected ShareRevoked, got {:?}", other),
        }
    }

    #[test]
    fn test_pending_share_creation_ignored() {
        let sid = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let new_img = share_image(&sid, &fid, "owner1", "a@b.com", "pending", SCOPE_JSON);
        let record = make_event_record("INSERT", keys, HashMap::new(), new_img);

        assert!(matches!(classify_record(&record), RecordChange::Ignored));
    }

    #[test]
    fn test_email_partition_share_ignored() {
        let sid = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s("SHARE_EMAIL#a@b.com"));
        keys.insert("SK".to_string(), s(&format!("SHARE#{}", sid)));
        let new_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", SCOPE_JSON);
        let record = make_event_record("INSERT", keys, HashMap::new(), new_img);

        assert!(matches!(classify_record(&record), RecordChange::Ignored));
    }

    #[test]
    fn test_family_insert_is_resource_changed() {
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s("OWNER#user1"));
        keys.insert(
            "SK".to_string(),
            s(&format!("FAMILY#{}", uuid::Uuid::new_v4())),
        );
        let record = make_event_record("INSERT", keys, HashMap::new(), HashMap::new());

        assert!(matches!(
            classify_record(&record),
            RecordChange::ResourceChanged(_)
        ));
    }

    #[test]
    fn test_dependent_modify_is_resource_changed() {
        let mut keys = HashMap::new();
        keys.insert(
            "PK".to_string(),
            s(&format!("FAMILY#{}", uuid::Uuid::new_v4())),
        );
        keys.insert(
            "SK".to_string(),
            s(&format!("DEPENDENT#{}", uuid::Uuid::new_v4())),
        );
        let record = make_event_record("MODIFY", keys, HashMap::new(), HashMap::new());

        assert!(matches!(
            classify_record(&record),
            RecordChange::ResourceChanged(_)
        ));
    }

    #[test]
    fn test_activity_remove_is_resource_changed() {
        let mut keys = HashMap::new();
        keys.insert(
            "PK".to_string(),
            s(&format!(
                "FAMILY#{}#DEPENDENT#{}",
                uuid::Uuid::new_v4(),
                uuid::Uuid::new_v4()
            )),
        );
        keys.insert(
            "SK".to_string(),
            s(&format!("ACTIVITY#{}", uuid::Uuid::new_v4())),
        );
        let record = make_event_record("REMOVE", keys, HashMap::new(), HashMap::new());

        assert!(matches!(
            classify_record(&record),
            RecordChange::ResourceChanged(_)
        ));
    }

    #[test]
    fn test_unknown_sk_prefix_ignored() {
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s("OWNER#user1"));
        keys.insert("SK".to_string(), s("UNKNOWN#something"));
        let record = make_event_record("INSERT", keys, HashMap::new(), HashMap::new());

        assert!(matches!(classify_record(&record), RecordChange::Ignored));
    }

    #[test]
    fn test_active_to_active_modify_no_scope_change_ignored() {
        let sid = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let old_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", SCOPE_JSON);
        let new_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", SCOPE_JSON);
        let record = make_event_record("MODIFY", keys, old_img, new_img);

        assert!(matches!(classify_record(&record), RecordChange::Ignored));
    }

    // -----------------------------------------------------------------------
    // Tests for propagate_change helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_family_id_from_family_record() {
        let pk = "OWNER#user1";
        let fid = uuid::Uuid::new_v4().to_string();
        let sk = format!("FAMILY#{}", fid);
        let mut image = HashMap::new();
        image.insert("id".to_string(), DdbAttributeValue::S(fid.clone()));
        assert_eq!(extract_family_id(pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_family_id_from_dependent_record() {
        let fid = uuid::Uuid::new_v4().to_string();
        let pk = format!("FAMILY#{}", fid);
        let sk = format!("DEPENDENT#{}", uuid::Uuid::new_v4());
        let mut image = HashMap::new();
        image.insert("family_id".to_string(), DdbAttributeValue::S(fid.clone()));
        assert_eq!(extract_family_id(&pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_family_id_from_activity_record() {
        let fid = uuid::Uuid::new_v4().to_string();
        let did = uuid::Uuid::new_v4().to_string();
        let pk = format!("FAMILY#{}#DEPENDENT#{}", fid, did);
        let sk = format!("ACTIVITY#{}", uuid::Uuid::new_v4());
        let mut image = HashMap::new();
        image.insert("family_id".to_string(), DdbAttributeValue::S(fid.clone()));
        assert_eq!(extract_family_id(&pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_family_id_from_pk_fallback() {
        let fid = uuid::Uuid::new_v4().to_string();
        let pk = format!("FAMILY#{}", fid);
        let sk = format!("DEPENDENT#{}", uuid::Uuid::new_v4());
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert_eq!(extract_family_id(&pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_family_id_from_activity_pk_fallback() {
        let fid = uuid::Uuid::new_v4().to_string();
        let did = uuid::Uuid::new_v4().to_string();
        let pk = format!("FAMILY#{}#DEPENDENT#{}", fid, did);
        let sk = format!("ACTIVITY#{}", uuid::Uuid::new_v4());
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert_eq!(extract_family_id(&pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_owner_id_from_family_pk() {
        let pk = "OWNER#user1";
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert_eq!(extract_owner_id(pk, &image), Some("user1".to_string()));
    }

    #[test]
    fn test_extract_owner_id_from_image_attribute() {
        let pk = "FAMILY#some-fid";
        let mut image = HashMap::new();
        image.insert(
            "owner_id".to_string(),
            DdbAttributeValue::S("user2".to_string()),
        );
        assert_eq!(extract_owner_id(pk, &image), Some("user2".to_string()));
    }

    #[test]
    fn test_extract_owner_id_none_for_dependent_without_attr() {
        let pk = "FAMILY#some-fid";
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert_eq!(extract_owner_id(pk, &image), None);
    }

    #[test]
    fn test_is_mirrored_resource_true() {
        let mut image = HashMap::new();
        image.insert(
            "share_id".to_string(),
            DdbAttributeValue::S("some-share-id".to_string()),
        );
        assert!(is_mirrored_resource(&image));
    }

    #[test]
    fn test_is_mirrored_resource_false() {
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert!(!is_mirrored_resource(&image));
    }

    #[test]
    fn test_resource_changed_carries_context() {
        let fid = uuid::Uuid::new_v4().to_string();
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s("OWNER#user1"));
        keys.insert("SK".to_string(), s(&format!("FAMILY#{}", fid)));

        let mut new_img = HashMap::new();
        new_img.insert("id".to_string(), s(&fid));
        new_img.insert("name".to_string(), s("Test Family"));

        let record = make_event_record("INSERT", keys, HashMap::new(), new_img);

        match classify_record(&record) {
            RecordChange::ResourceChanged(change) => {
                assert_eq!(change.pk, "OWNER#user1");
                assert!(change.sk.starts_with("FAMILY#"));
                assert_eq!(change.operation, ChangeOperation::Insert);
                assert!(!change.new_image.is_empty());
            }
            other => panic!("Expected ResourceChanged, got {:?}", other),
        }
    }

    #[test]
    fn test_resource_changed_remove_carries_old_image() {
        let fid = uuid::Uuid::new_v4().to_string();
        let did = uuid::Uuid::new_v4().to_string();
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s(&format!("FAMILY#{}", fid)));
        keys.insert("SK".to_string(), s(&format!("DEPENDENT#{}", did)));

        let mut old_img = HashMap::new();
        old_img.insert("id".to_string(), s(&did));
        old_img.insert("family_id".to_string(), s(&fid));

        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());

        match classify_record(&record) {
            RecordChange::ResourceChanged(change) => {
                assert_eq!(change.operation, ChangeOperation::Remove);
                assert!(!change.old_image.is_empty());
                assert!(change.new_image.is_empty());
            }
            other => panic!("Expected ResourceChanged, got {:?}", other),
        }
    }

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

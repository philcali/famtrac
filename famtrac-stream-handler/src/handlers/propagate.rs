use aws_sdk_dynamodb::types::AttributeValue as DdbAttributeValue;
use aws_sdk_dynamodb::Client;
use lambda_runtime::Error;

use crate::classify::{strip_sync_token, ChangeOperation, ResourceChange};
use crate::dynamo_util::{
    delete_item, extract_family_id, find_active_shares_by_family_id, get_item,
    is_mirrored_resource, put_item, rekey_item,
};

/// Handle a resource change event by propagating it to mirrored copies or
/// writing it back to the owner partition.
///
/// Before any propagation, checks if the new image contains a `sync_token`
/// attribute. If present, the write was handler-originated and propagation
/// is skipped to break infinite cycles (Requirement 6.4, 6.6).
pub async fn handle_resource_changed(
    client: &Client,
    table_name: &str,
    change: &ResourceChange,
    sync_token: &str,
) -> Result<(), Error> {
    // Skip handler-originated writes (Requirement 6.4, 6.6)
    if change.new_image.contains_key("sync_token") {
        return Ok(());
    }

    // Determine which image to inspect (new for insert/modify, old for remove)
    let image = match change.operation {
        ChangeOperation::Remove => &change.old_image,
        _ => &change.new_image,
    };

    // If this is a mirrored resource, propagate back to the original (write-back)
    if is_mirrored_resource(image) {
        return propagate_writeback(client, table_name, change, sync_token).await;
    }

    // This is an owned resource — propagate to all active share mirrors
    propagate_to_mirrors(client, table_name, change, sync_token).await
}

/// Propagate an owned resource change to all mirrored copies in accepter partitions.
/// Stamps `sync_token` on every item written (Requirement 6.1, 6.2).
async fn propagate_to_mirrors(
    client: &Client,
    table_name: &str,
    change: &ResourceChange,
    sync_token: &str,
) -> Result<(), Error> {
    let image = match change.operation {
        ChangeOperation::Remove => &change.old_image,
        _ => &change.new_image,
    };

    let family_id = match extract_family_id(&change.pk, &change.sk, image) {
        Some(fid) => fid,
        None => return Ok(()), // Can't determine family — skip
    };

    // Use GSI query to find all active shares for this family (Requirement 2.1)
    let active_shares = find_active_shares_by_family_id(client, table_name, &family_id).await?;

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
                    let mut mirrored = rekey_item(
                        change.new_image.clone(),
                        &format!("OWNER#{}", accepter_id),
                        &share_id_str,
                        &scope_json,
                    );
                    mirrored.insert(
                        "sync_token".to_string(),
                        DdbAttributeValue::S(sync_token.to_string()),
                    );
                    put_item(client, table_name, mirrored).await?;
                } else {
                    // Dependent/Activity records keep the same PK/SK.
                    // The original item is shared, so we don't need to create
                    // separate copies. The share metadata is on the mirrored
                    // Family record, and permission checks use that context.
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

/// Propagate a write-back from a mirrored resource to the original owner's partition.
///
/// When an accepter modifies a mirrored resource (e.g., creates an Activity),
/// the stream handler detects the `share_id` on the record and propagates the
/// change back to the original family's partition.
///
/// For Family record write-backs, performs a semantic diff against the existing
/// owner record before writing — skips if identical (Requirement 6.7, 6.8).
///
/// Stamps `sync_token` on every item written (Requirement 6.1, 6.2).
async fn propagate_writeback(
    client: &Client,
    table_name: &str,
    change: &ResourceChange,
    sync_token: &str,
) -> Result<(), Error> {
    let image = match change.operation {
        ChangeOperation::Remove => &change.old_image,
        _ => &change.new_image,
    };

    let is_family_record = change.sk.starts_with("FAMILY#");

    if is_family_record {
        // Mirrored Family records have PK=OWNER#{accepter_id}.
        // We need to find the original owner and write back to their partition.
        let family_id = match extract_family_id(&change.pk, &change.sk, image) {
            Some(fid) => fid,
            None => return Ok(()),
        };

        // Query active shares by family_id via GSI, derive owner from requester_id
        let active_shares = find_active_shares_by_family_id(client, table_name, &family_id).await?;
        let original_owner = match active_shares.first().map(|s| s.requester_id.0.clone()) {
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

                // Semantic diff before write-back (Requirement 6.7, 6.8):
                // compare stripped new image against existing owner record, skip if identical
                let existing = get_item(
                    client,
                    table_name,
                    &format!("OWNER#{}", original_owner),
                    &change.sk,
                )
                .await?;
                if let Some(existing_item) = existing {
                    let stripped_new = strip_sync_token(&original_item);
                    let stripped_existing = strip_sync_token(&existing_item);
                    if stripped_new == stripped_existing {
                        return Ok(()); // No meaningful change — skip write-back
                    }
                }

                // Stamp sync_token on the write-back item
                original_item.insert(
                    "sync_token".to_string(),
                    DdbAttributeValue::S(sync_token.to_string()),
                );
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

            // Semantic diff: compare against existing to avoid no-op writes (Requirement 6.8)
            let existing_pk = match item.get("PK").and_then(|v| v.as_s().ok()) {
                Some(pk) => pk.clone(),
                None => return Ok(()),
            };
            let existing_sk = match item.get("SK").and_then(|v| v.as_s().ok()) {
                Some(sk) => sk.clone(),
                None => return Ok(()),
            };
            let existing = get_item(client, table_name, &existing_pk, &existing_sk).await?;
            if let Some(existing_item) = existing {
                let stripped_new = strip_sync_token(&item);
                let stripped_existing = strip_sync_token(&existing_item);
                if stripped_new == stripped_existing {
                    return Ok(()); // No meaningful change — skip write-back
                }
            }

            // Stamp sync_token on the write-back item
            item.insert(
                "sync_token".to_string(),
                DdbAttributeValue::S(sync_token.to_string()),
            );
            put_item(client, table_name, item).await?;
        }
    }

    Ok(())
}

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
///
/// # Errors
///
/// Returns an error if any `DynamoDB` operation fails.
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
    if is_mirrored_resource(&change.pk, image) {
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

    let Some(family_id) = extract_family_id(&change.pk, &change.sk, image) else {
        return Ok(()); // Can't determine family — skip
    };

    // Use GSI query to find all active shares for this family (Requirement 2.1)
    let active_shares = find_active_shares_by_family_id(client, table_name, &family_id).await?;

    if active_shares.is_empty() {
        return Ok(());
    }

    let is_family_record = change.sk.starts_with("FAMILY#");
    let is_recipe_record = change.sk.starts_with("RECIPE#");

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
                if is_family_record || is_recipe_record {
                    // Family and Recipe records are rekeyed into accepter's OWNER partition
                    let mut mirrored = rekey_item(
                        change.new_image.clone(),
                        &format!("OWNER#{accepter_id}"),
                        &share_id_str,
                        &scope_json,
                    );
                    mirrored.insert(
                        "sync_token".to_string(),
                        DdbAttributeValue::S(sync_token.to_string()),
                    );
                    put_item(client, table_name, mirrored).await?;
                } else {
                    // Dependent/MealSlot/FeedingLog/Activity records keep the same PK/SK.
                    // The original item is shared, so we don't need to create
                    // separate copies. The share metadata is on the mirrored
                    // Family record, and permission checks use that context.
                }
            }
            ChangeOperation::Remove => {
                if is_family_record || is_recipe_record {
                    // Delete the mirrored Family/Recipe from the accepter's partition
                    delete_item(
                        client,
                        table_name,
                        &format!("OWNER#{accepter_id}"),
                        &change.sk,
                    )
                    .await?;
                }
                // For Dependent/MealSlot/FeedingLog/Activity removes: the original item is deleted,
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
/// For Family and Recipe record write-backs, performs a semantic diff against the existing
/// owner record before writing — skips if identical (Requirement 6.7, 6.8).
///
/// Stamps `sync_token` on every item written (Requirement 6.1, 6.2).
#[allow(clippy::similar_names)]
#[allow(clippy::too_many_lines)]
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
    let is_recipe_record = change.sk.starts_with("RECIPE#");

    if is_family_record || is_recipe_record {
        // Mirrored Family/Recipe records have PK=OWNER#{accepter_id}.
        // We need to find the original owner and write back to their partition.
        let Some(family_id) = extract_family_id(&change.pk, &change.sk, image) else {
            return Ok(());
        };

        // Query active shares by family_id via GSI, derive owner from requester_id
        let active_shares = find_active_shares_by_family_id(client, table_name, &family_id).await?;
        let Some(original_owner) = active_shares.first().map(|s| s.requester_id.0.clone()) else {
            return Ok(());
        };

        match change.operation {
            ChangeOperation::Insert | ChangeOperation::Modify => {
                // Write back to the original owner's partition, stripping share metadata
                let mut original_item = change.new_image.clone();
                original_item.insert(
                    "PK".to_string(),
                    DdbAttributeValue::S(format!("OWNER#{original_owner}")),
                );
                original_item.remove("share_id");
                original_item.remove("permission_scope");

                // Semantic diff before write-back (Requirement 6.7, 6.8):
                // compare stripped new image against existing owner record, skip if identical
                let existing = get_item(
                    client,
                    table_name,
                    &format!("OWNER#{original_owner}"),
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
                    &format!("OWNER#{original_owner}"),
                    &change.sk,
                )
                .await?;
            }
        }
    } else if change.sk.starts_with("MEAL_SLOT#") || change.sk.starts_with("FEEDING_LOG#") {
        // MealSlot/FeedingLog mirrored records share the same PK/SK as originals.
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

#[cfg(test)]
mod tests {
    use crate::classify::{classify_record, ChangeOperation, RecordChange, ResourceChange};
    use aws_lambda_events::event::dynamodb::{EventRecord, StreamRecord};
    use chrono::Utc;
    use serde_dynamo::AttributeValue;
    use std::collections::HashMap;

    fn s(val: &str) -> AttributeValue {
        AttributeValue::S(val.to_string())
    }

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

    fn recipe_keys(family_id: &str, recipe_id: &str) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert("PK".to_string(), s(&format!("FAMILY#{family_id}")));
        m.insert("SK".to_string(), s(&format!("RECIPE#{recipe_id}")));
        m
    }

    fn mirrored_recipe_keys(
        accepter_id: &str,
        recipe_id: &str,
    ) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert("PK".to_string(), s(&format!("OWNER#{accepter_id}")));
        m.insert("SK".to_string(), s(&format!("RECIPE#{recipe_id}")));
        m
    }

    fn recipe_image(recipe_id: &str, family_id: &str, name: &str) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert("id".to_string(), s(recipe_id));
        m.insert("family_id".to_string(), s(family_id));
        m.insert("name".to_string(), s(name));
        m.insert("created_at".to_string(), s("2025-01-01T00:00:00Z"));
        m.insert("updated_at".to_string(), s("2025-01-01T00:00:00Z"));
        m
    }

    fn mirrored_recipe_image(
        recipe_id: &str,
        family_id: &str,
        name: &str,
    ) -> HashMap<String, AttributeValue> {
        let mut m = recipe_image(recipe_id, family_id, name);
        m.insert("share_id".to_string(), s("share-123"));
        m.insert(
            "permission_scope".to_string(),
            s(r#"{"actions":["dependent_read","dependent_write"]}"#),
        );
        m
    }

    // -------------------------------------------------------------------
    // Critical bug regression tests: recipe deletes must not trigger write-back
    // -------------------------------------------------------------------

    #[test]
    fn test_delete_original_recipe_classified_as_resource_changed_not_writeback() {
        // When a recipe is deleted from the ORIGINAL partition (PK = FAMILY#{fid}),
        // the old image contains share_id (because it was shared). This MUST be
        // classified as ResourceChanged (propagate_to_mirrors), NOT as a mirrored
        // resource write-back.
        let fid = uuid::Uuid::new_v4().to_string();
        let rid = uuid::Uuid::new_v4().to_string();
        let keys = recipe_keys(&fid, &rid);
        let old_img = mirrored_recipe_image(&rid, &fid, "Test Recipe");

        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());
        let change = classify_record(&record);

        match change {
            RecordChange::ResourceChanged(ResourceChange {
                pk,
                operation: ChangeOperation::Remove,
                ..
            }) => {
                // PK must be the FAMILY partition, NOT an OWNER partition
                assert!(
                    pk.starts_with("FAMILY#"),
                    "Original recipe delete PK must be FAMILY#{fid}, got {pk}"
                );
                assert!(
                    !pk.starts_with("OWNER#"),
                    "Original recipe delete PK must NOT be OWNER#, got {pk}"
                );
            }
            other => panic!(
                "Expected ResourceChanged with Remove, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_delete_mirrored_recipe_classified_as_resource_changed() {
        // When a recipe is deleted from a MIRRORED partition (PK = OWNER#{accepter_id}),
        // it should also be ResourceChanged, but the PK will be OWNER#{accepter_id}.
        // The is_mirrored_resource check in handle_resource_changed will route this
        // to propagate_writeback (which is correct for mirrored records).
        let accepter_id = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let rid = uuid::Uuid::new_v4().to_string();
        let keys = mirrored_recipe_keys(&accepter_id, &rid);
        let old_img = mirrored_recipe_image(&rid, &fid, "Test Recipe");

        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());
        let change = classify_record(&record);

        match change {
            RecordChange::ResourceChanged(ResourceChange { pk, .. }) => {
                assert!(
                    pk.starts_with("OWNER#"),
                    "Mirrored recipe delete PK must be OWNER#{accepter_id}, got {pk}"
                );
            }
            other => panic!(
                "Expected ResourceChanged, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_delete_original_family_with_share_id_not_mirrored() {
        // Regression: a Family record in the original OWNER partition with share_id
        // is the accepter's mirrored copy, NOT the original. But a Family record in
        // the original owner's partition (OWNER#{owner_id}) with share_id is the
        // mirrored copy that was created by the share activation.
        let fid = uuid::Uuid::new_v4().to_string();
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s(&format!("OWNER#original-owner")));
        keys.insert("SK".to_string(), s(&format!("FAMILY#{fid}")));
        let old_img = {
            let mut m = HashMap::new();
            m.insert("id".to_string(), s(&fid));
            m.insert("name".to_string(), s("Test Family"));
            m.insert(
                "owner_id".to_string(),
                s("original-owner"),
            );
            m.insert("share_id".to_string(), s("share-123"));
            m
        };

        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());
        let change = classify_record(&record);

        match change {
            RecordChange::ResourceChanged(ResourceChange { pk, .. }) => {
                // The PK is OWNER#{original-owner} which is the accepter's mirrored copy
                assert!(pk.starts_with("OWNER#"));
            }
            other => panic!(
                "Expected ResourceChanged, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_delete_recipe_without_share_id_from_family_partition_not_mirrored() {
        // A recipe in the FAMILY partition without share_id is clearly not mirrored.
        let fid = uuid::Uuid::new_v4().to_string();
        let rid = uuid::Uuid::new_v4().to_string();
        let keys = recipe_keys(&fid, &rid);
        let old_img = recipe_image(&rid, &fid, "Test Recipe");

        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());
        let change = classify_record(&record);

        match change {
            RecordChange::ResourceChanged(ResourceChange { pk, .. }) => {
                assert_eq!(pk, format!("FAMILY#{fid}"));
            }
            other => panic!(
                "Expected ResourceChanged, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_insert_recipe_from_family_partition_not_mirrored() {
        // A recipe INSERT from the FAMILY partition is always the original.
        let fid = uuid::Uuid::new_v4().to_string();
        let rid = uuid::Uuid::new_v4().to_string();
        let keys = recipe_keys(&fid, &rid);
        let new_img = recipe_image(&rid, &fid, "New Recipe");

        let record = make_event_record("INSERT", keys, HashMap::new(), new_img);
        let change = classify_record(&record);

        match change {
            RecordChange::ResourceChanged(ResourceChange { pk, operation, .. }) => {
                assert_eq!(pk, format!("FAMILY#{fid}"));
                assert_eq!(operation, ChangeOperation::Insert);
            }
            other => panic!(
                "Expected ResourceChanged, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_modify_recipe_from_family_partition_not_mirrored() {
        // A recipe MODIFY from the FAMILY partition is always the original.
        let fid = uuid::Uuid::new_v4().to_string();
        let rid = uuid::Uuid::new_v4().to_string();
        let keys = recipe_keys(&fid, &rid);
        let old_img = recipe_image(&rid, &fid, "Old Recipe");
        let mut new_img = recipe_image(&rid, &fid, "New Recipe");
        new_img.insert("name".to_string(), s("New Recipe"));

        let record = make_event_record("MODIFY", keys, old_img, new_img);
        let change = classify_record(&record);

        match change {
            RecordChange::ResourceChanged(ResourceChange { pk, operation, .. }) => {
                assert_eq!(pk, format!("FAMILY#{fid}"));
                assert_eq!(operation, ChangeOperation::Modify);
            }
            other => panic!(
                "Expected ResourceChanged, got {:?}",
                other
            ),
        }
    }
}

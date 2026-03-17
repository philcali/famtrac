use aws_lambda_events::event::dynamodb::{Event as DynamoDbEvent, EventRecord, OperationType};
use lambda_runtime::{service_fn, Error, LambdaEvent};

use famtrac_backend::domain::{
    FamilyId, IdentityId, PermissionScope, Share, ShareId, ShareStatus, Timestamp,
};

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
    ResourceChanged,
    /// Record change is not relevant to stream processing.
    Ignored,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = service_fn(handle_stream_event);
    lambda_runtime::run(handler).await?;
    Ok(())
}

/// Main stream event handler — classifies each record and dispatches to the appropriate action.
async fn handle_stream_event(event: LambdaEvent<DynamoDbEvent>) -> Result<(), Error> {
    let dynamo_event = event.payload;

    for record in &dynamo_event.records {
        match classify_record(record) {
            RecordChange::ShareActivated(_share) => {
                // TODO (task 13.3): mirror_resources(share).await?
            }
            RecordChange::ShareRevoked(_share_id) => {
                // TODO (task 13.8): cleanup_mirrored(share_id).await?
            }
            RecordChange::SharePermissionUpdated(_share) => {
                // TODO (task 13.10): update_mirrored_permissions(share).await?
            }
            RecordChange::ResourceChanged => {
                // TODO (task 13.5): propagate_change(change).await?
            }
            RecordChange::Ignored => {}
        }
    }

    Ok(())
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
        "Family" | "Dependent" | "Activity" => RecordChange::ResourceChanged,
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
            RecordChange::ResourceChanged
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
            RecordChange::ResourceChanged
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
            RecordChange::ResourceChanged
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
}

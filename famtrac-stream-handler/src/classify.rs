use aws_lambda_events::event::dynamodb::{EventRecord, OperationType};
use aws_sdk_dynamodb::types::AttributeValue as DdbAttributeValue;
use std::collections::HashMap;

use famtrac_backend::domain::{FamilyId, IdentityId, Share, ShareId, ShareStatus};

use crate::dynamo_util::convert_image;
use crate::parser::{get_str, parse_share};

/// The type of `DynamoDB` Stream operation.
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

/// Classification of a `DynamoDB` Stream record change.
#[derive(Debug)]
pub enum RecordChange {
    /// A share transitioned to active status — mirror resources into accepter's partition.
    ShareActivated(Share),
    /// A share was deleted — clean up mirrored records.
    ShareRevoked {
        share_id: ShareId,
        family_id: FamilyId,
        accepter_id: IdentityId,
    },
    /// A share's permission scope was updated — update mirrored record metadata.
    SharePermissionUpdated(Share),
    /// A family/dependent/activity was created, updated, or deleted — propagate to mirrored copies.
    ResourceChanged(ResourceChange),
    /// Record change is not relevant to stream processing.
    Ignored,
}

/// Discriminant used as the routing key for the Router dispatch table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    ShareActivated,
    ShareRevoked,
    SharePermissionUpdated,
    ResourceChanged,
}

/// Map a `RecordChange` to its `ChangeKind` routing key.
/// Returns `None` for `Ignored` variants (the router skips these).
#[must_use]
pub const fn change_kind(rc: &RecordChange) -> Option<ChangeKind> {
    match rc {
        RecordChange::ShareActivated(_) => Some(ChangeKind::ShareActivated),
        RecordChange::ShareRevoked { .. } => Some(ChangeKind::ShareRevoked),
        RecordChange::SharePermissionUpdated(_) => Some(ChangeKind::SharePermissionUpdated),
        RecordChange::ResourceChanged(_) => Some(ChangeKind::ResourceChanged),
        RecordChange::Ignored => None,
    }
}

/// Strip the `sync_token` attribute from a `DynamoDB` image for semantic comparison.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn strip_sync_token(
    image: &HashMap<String, DdbAttributeValue>,
) -> HashMap<String, DdbAttributeValue> {
    image
        .iter()
        .filter(|(k, _)| k.as_str() != "sync_token")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Determine the record type from the SK prefix.
/// Returns `"Share"` for share records, `"Family"` / `"Dependent"` / `"Activity"` for
/// resource records, `"Recipe"` / `"MealSlot"` / `"FeedingLog"` for meal planning records,
/// or `None` for anything we don't care about.
fn record_type_from_sk(sk: &str) -> Option<&'static str> {
    if sk.starts_with("SHARE#") {
        Some("Share")
    } else if sk.starts_with("RECIPE#") {
        Some("Recipe")
    } else if sk.starts_with("MEAL_SLOT#") {
        Some("MealSlot")
    } else if sk.starts_with("FEEDING_LOG#") {
        Some("FeedingLog")
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

/// Classify a single `DynamoDB` Stream record into a `RecordChange` variant.
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
///   - Semantic no-op (old == new after stripping `sync_token`) → `Ignored`
/// - Anything else → `Ignored`
#[must_use]
pub fn classify_record(record: &EventRecord) -> RecordChange {
    let keys = &record.change.keys;

    // Extract PK and SK from the key attributes.
    let Some(pk) = get_str(keys, "PK") else {
        return RecordChange::Ignored;
    };
    let Some(sk) = get_str(keys, "SK") else {
        return RecordChange::Ignored;
    };

    let Some(record_type) = record_type_from_sk(&sk) else {
        return RecordChange::Ignored;
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
        "Family" | "Dependent" | "Activity" | "Recipe" | "MealSlot" | "FeedingLog" => {
            let change_op = match op {
                OperationType::Insert => ChangeOperation::Insert,
                OperationType::Modify => ChangeOperation::Modify,
                OperationType::Remove => ChangeOperation::Remove,
            };
            let new_image = convert_image(&record.change.new_image);
            let old_image = convert_image(&record.change.old_image);

            // Semantic no-op detection: if old and new images are identical
            // after stripping sync_token, classify as Ignored.
            if change_op == ChangeOperation::Modify {
                let stripped_new = strip_sync_token(&new_image);
                let stripped_old = strip_sync_token(&old_image);
                if stripped_new == stripped_old {
                    return RecordChange::Ignored;
                }
            }

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
            let share_id = match share_id_str.parse::<uuid::Uuid>() {
                Ok(id) => ShareId(id),
                Err(_) => return RecordChange::Ignored,
            };

            // Extract family_id and accepter_id from the old image
            let old_image = convert_image(&record.change.old_image);
            let family_id = match old_image
                .get("family_id")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| s.parse::<uuid::Uuid>().ok())
            {
                Some(id) => FamilyId(id),
                None => return RecordChange::Ignored,
            };
            let accepter_id = match old_image.get("accepter_id").and_then(|v| v.as_s().ok()) {
                Some(id) => IdentityId(id.clone()),
                None => return RecordChange::Ignored,
            };

            RecordChange::ShareRevoked {
                share_id,
                family_id,
                accepter_id,
            }
        }
        OperationType::Insert | OperationType::Modify => {
            let new_image = &record.change.new_image;
            let Some(new_share) = parse_share(new_image) else {
                return RecordChange::Ignored;
            };

            let new_status = &new_share.status;

            // Check if this is an activation (status transitioned to Active).
            if *new_status == ShareStatus::Active {
                let old_was_active =
                    get_str(&record.change.old_image, "status").is_some_and(|s| s == "active");

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
        username: &str,
        status: &str,
        scope_json: &str,
    ) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert("id".to_string(), s(share_id));
        m.insert("family_id".to_string(), s(family_id));
        m.insert("requester_id".to_string(), s(requester_id));
        m.insert("accepter_username".to_string(), s(username));
        m.insert("status".to_string(), s(status));
        m.insert("permission_scope".to_string(), s(scope_json));
        m.insert("created_at".to_string(), s("2025-01-01T00:00:00Z"));
        m.insert("updated_at".to_string(), s("2025-01-01T00:00:00Z"));
        m
    }

    fn share_image_with_accepter(
        share_id: &str,
        family_id: &str,
        requester_id: &str,
        username: &str,
        accepter_id: &str,
        status: &str,
        scope_json: &str,
    ) -> HashMap<String, AttributeValue> {
        let mut m = share_image(
            share_id,
            family_id,
            requester_id,
            username,
            status,
            scope_json,
        );
        m.insert("accepter_id".to_string(), s(accepter_id));
        m
    }

    const SCOPE_JSON: &str = r#"{"actions":["family_read"]}"#;

    // -------------------------------------------------------------------
    // ChangeKind tests
    // -------------------------------------------------------------------

    #[test]
    fn test_change_kind_share_activated() {
        // Use a minimal Share — we only care about the variant discriminant
        let sid = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let new_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", SCOPE_JSON);
        let record = make_event_record("INSERT", keys, HashMap::new(), new_img);
        let rc = classify_record(&record);
        assert_eq!(change_kind(&rc), Some(ChangeKind::ShareActivated));
    }

    #[test]
    fn test_change_kind_ignored() {
        let rc = RecordChange::Ignored;
        assert_eq!(change_kind(&rc), None);
    }

    // -------------------------------------------------------------------
    // Classifier tests (relocated from main.rs)
    // -------------------------------------------------------------------

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
        let fid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let old_img = share_image_with_accepter(
            &sid,
            &fid,
            "owner1",
            "a@b.com",
            "accepter1",
            "active",
            SCOPE_JSON,
        );
        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());

        match classify_record(&record) {
            RecordChange::ShareRevoked {
                share_id,
                family_id,
                accepter_id,
            } => {
                assert_eq!(share_id.0.to_string(), sid);
                assert_eq!(family_id.0.to_string(), fid);
                assert_eq!(accepter_id.0, "accepter1");
            }
            other => panic!("Expected ShareRevoked, got {:?}", other),
        }
    }

    #[test]
    fn test_share_revoked_missing_accepter_id_ignored() {
        let sid = uuid::Uuid::new_v4().to_string();
        let fid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        // Old image without accepter_id
        let old_img = share_image(&sid, &fid, "owner1", "a@b.com", "active", SCOPE_JSON);
        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());

        assert!(matches!(classify_record(&record), RecordChange::Ignored));
    }

    #[test]
    fn test_share_revoked_missing_family_id_ignored() {
        let sid = uuid::Uuid::new_v4().to_string();
        let keys = share_keys("owner1", &sid);
        let mut old_img = HashMap::new();
        old_img.insert("id".to_string(), s(&sid));
        old_img.insert("accepter_id".to_string(), s("accepter1"));
        // No family_id
        let record = make_event_record("REMOVE", keys, old_img, HashMap::new());

        assert!(matches!(classify_record(&record), RecordChange::Ignored));
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
        let mut old_img = HashMap::new();
        old_img.insert("name".to_string(), s("Old Name"));
        let mut new_img = HashMap::new();
        new_img.insert("name".to_string(), s("New Name"));
        let record = make_event_record("MODIFY", keys, old_img, new_img);

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
    fn test_recipe_insert_is_resource_changed() {
        let mut keys = HashMap::new();
        keys.insert(
            "PK".to_string(),
            s(&format!("FAMILY#{}", uuid::Uuid::new_v4())),
        );
        keys.insert(
            "SK".to_string(),
            s(&format!("RECIPE#{}", uuid::Uuid::new_v4())),
        );
        let record = make_event_record("INSERT", keys, HashMap::new(), HashMap::new());

        assert!(matches!(
            classify_record(&record),
            RecordChange::ResourceChanged(_)
        ));
    }

    #[test]
    fn test_meal_slot_modify_is_resource_changed() {
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
            s(&format!("MEAL_SLOT#{}", uuid::Uuid::new_v4())),
        );
        let mut old_img = HashMap::new();
        old_img.insert("time".to_string(), s("08:00"));
        let mut new_img = HashMap::new();
        new_img.insert("time".to_string(), s("09:00"));
        let record = make_event_record("MODIFY", keys, old_img, new_img);

        assert!(matches!(
            classify_record(&record),
            RecordChange::ResourceChanged(_)
        ));
    }

    #[test]
    fn test_feeding_log_remove_is_resource_changed() {
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
            s(&format!("FEEDING_LOG#{}", uuid::Uuid::new_v4())),
        );
        let record = make_event_record("REMOVE", keys, HashMap::new(), HashMap::new());

        assert!(matches!(
            classify_record(&record),
            RecordChange::ResourceChanged(_)
        ));
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

    // -------------------------------------------------------------------
    // Resource change context tests (relocated from main.rs)
    // -------------------------------------------------------------------

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

    // -------------------------------------------------------------------
    // Semantic no-op detection tests
    // -------------------------------------------------------------------

    #[test]
    fn test_semantic_noop_modify_only_sync_token_diff_ignored() {
        let fid = uuid::Uuid::new_v4().to_string();
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s("OWNER#user1"));
        keys.insert("SK".to_string(), s(&format!("FAMILY#{}", fid)));

        let mut old_img = HashMap::new();
        old_img.insert("id".to_string(), s(&fid));
        old_img.insert("name".to_string(), s("Test Family"));

        let mut new_img = old_img.clone();
        new_img.insert("sync_token".to_string(), s("some-request-id"));

        let record = make_event_record("MODIFY", keys, old_img, new_img);

        assert!(matches!(classify_record(&record), RecordChange::Ignored));
    }

    #[test]
    fn test_semantic_noop_both_have_different_sync_tokens_ignored() {
        let fid = uuid::Uuid::new_v4().to_string();
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s("OWNER#user1"));
        keys.insert("SK".to_string(), s(&format!("FAMILY#{}", fid)));

        let mut old_img = HashMap::new();
        old_img.insert("id".to_string(), s(&fid));
        old_img.insert("name".to_string(), s("Test Family"));
        old_img.insert("sync_token".to_string(), s("old-token"));

        let mut new_img = HashMap::new();
        new_img.insert("id".to_string(), s(&fid));
        new_img.insert("name".to_string(), s("Test Family"));
        new_img.insert("sync_token".to_string(), s("new-token"));

        let record = make_event_record("MODIFY", keys, old_img, new_img);

        assert!(matches!(classify_record(&record), RecordChange::Ignored));
    }

    #[test]
    fn test_real_modify_with_sync_token_not_ignored() {
        let fid = uuid::Uuid::new_v4().to_string();
        let mut keys = HashMap::new();
        keys.insert("PK".to_string(), s("OWNER#user1"));
        keys.insert("SK".to_string(), s(&format!("FAMILY#{}", fid)));

        let mut old_img = HashMap::new();
        old_img.insert("id".to_string(), s(&fid));
        old_img.insert("name".to_string(), s("Old Name"));

        let mut new_img = HashMap::new();
        new_img.insert("id".to_string(), s(&fid));
        new_img.insert("name".to_string(), s("New Name"));
        new_img.insert("sync_token".to_string(), s("some-token"));

        let record = make_event_record("MODIFY", keys, old_img, new_img);

        assert!(matches!(
            classify_record(&record),
            RecordChange::ResourceChanged(_)
        ));
    }

    // -------------------------------------------------------------------
    // strip_sync_token tests
    // -------------------------------------------------------------------

    #[test]
    fn test_strip_sync_token_removes_only_sync_token() {
        let mut image = HashMap::new();
        image.insert(
            "PK".to_string(),
            DdbAttributeValue::S("OWNER#user1".to_string()),
        );
        image.insert("name".to_string(), DdbAttributeValue::S("Test".to_string()));
        image.insert(
            "sync_token".to_string(),
            DdbAttributeValue::S("token-123".to_string()),
        );

        let stripped = strip_sync_token(&image);
        assert_eq!(stripped.len(), 2);
        assert!(stripped.contains_key("PK"));
        assert!(stripped.contains_key("name"));
        assert!(!stripped.contains_key("sync_token"));
    }

    #[test]
    fn test_strip_sync_token_noop_when_absent() {
        let mut image = HashMap::new();
        image.insert(
            "PK".to_string(),
            DdbAttributeValue::S("OWNER#user1".to_string()),
        );
        image.insert("name".to_string(), DdbAttributeValue::S("Test".to_string()));

        let stripped = strip_sync_token(&image);
        assert_eq!(stripped.len(), 2);
        assert_eq!(stripped, image);
    }
}

//! Integration tests for `handle_permission_updated`.
//!
//! These tests verify that updating a share's permission scope correctly updates
//! mirrored records in the accepter's partition WITHOUT modifying original records.
//!
//! Critical regression: mirrored Recipe records live at `OWNER#{accepter_id}` (rekeyed),
//! NOT at `FAMILY#{family_id}`. The permission handler must target the correct partition.

mod common;

use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use famtrac_backend::domain::*;
use famtrac_stream_handler::handlers::permission::handle_permission_updated;

fn s(val: &str) -> AttributeValue {
    AttributeValue::S(val.to_string())
}

fn make_item(pk: &str, sk: &str, attrs: &[(&str, &str)]) -> HashMap<String, AttributeValue> {
    let mut item = HashMap::new();
    item.insert("PK".to_string(), s(pk));
    item.insert("SK".to_string(), s(sk));
    for (k, v) in attrs {
        item.insert(k.to_string(), s(v));
    }
    item
}

fn make_test_share(
    share_id: uuid::Uuid,
    family_id: uuid::Uuid,
    requester_id: &str,
    accepter_id: &str,
    scope: PermissionScope,
) -> Share {
    Share {
        id: ShareId(share_id),
        family_id: FamilyId(family_id),
        requester_id: IdentityId::new(requester_id.to_string()),
        accepter_id: Some(IdentityId::new(accepter_id.to_string())),
        accepter_username: "test@example.com".to_string(),
        permission_scope: scope,
        status: ShareStatus::Active,
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
        expires_at: None,
    }
}

#[tokio::test]
#[ignore] // Requires DynamoDB Local (java)
async fn test_permission_update_targets_mirrored_recipes_in_accepter_partition() {
    let db = common::DynamoDbLocalInstance::start("perm-recipe-test")
        .await
        .expect("DynamoDB Local required for this test");

    let family_id = uuid::Uuid::new_v4();
    let share_id = uuid::Uuid::new_v4();
    let accepter_id = "perm-accepter-1";
    let requester_id = "perm-requester-1";
    let recipe_id = uuid::Uuid::new_v4();

    let share_id_str = share_id.to_string();
    let fid = family_id.to_string();
    let old_scope = r#"{"actions":["family_read"]}"#;
    let new_scope = PermissionScope {
        actions: vec![
            PermissionAction::FamilyRead,
            PermissionAction::DependentRead,
        ],
    };

    // Seed: mirrored Family in accepter's partition
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("FAMILY#{fid}"),
        &[
            ("id", &fid),
            ("share_id", &share_id_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Seed: mirrored Recipe in accepter's OWNER partition (rekeyed by mirror handler)
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("RECIPE#{}", recipe_id),
        &[
            ("id", &recipe_id.to_string()),
            ("family_id", &fid),
            ("name", "Mirrored Recipe"),
            ("share_id", &share_id_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Seed: original Recipe in FAMILY partition (should NOT be updated)
    db.put_item(make_item(
        &format!("FAMILY#{fid}"),
        &format!("RECIPE#{}", recipe_id),
        &[
            ("id", &recipe_id.to_string()),
            ("family_id", &fid),
            ("name", "Original Recipe"),
        ],
    ))
    .await;

    // Act: update permission
    let share = make_test_share(share_id, family_id, requester_id, accepter_id, new_scope);
    handle_permission_updated(&db.client, &db.table_name, &share)
        .await
        .expect("handle_permission_updated should succeed");

    // Assert: mirrored Family record has updated scope
    let family = db
        .get_item(&format!("OWNER#{accepter_id}"), &format!("FAMILY#{fid}"))
        .await
        .expect("Mirrored Family should still exist");
    let family_scope = family
        .get("permission_scope")
        .and_then(|v| v.as_s().ok())
        .unwrap();
    assert!(
        family_scope.contains("dependent_read"),
        "Mirrored Family permission_scope should be updated, got: {}",
        family_scope
    );

    // Assert: mirrored Recipe in OWNER partition has updated scope
    let recipe = db
        .get_item(
            &format!("OWNER#{accepter_id}"),
            &format!("RECIPE#{}", recipe_id),
        )
        .await
        .expect("Mirrored Recipe should still exist");
    let recipe_scope = recipe
        .get("permission_scope")
        .and_then(|v| v.as_s().ok())
        .unwrap();
    assert!(
        recipe_scope.contains("dependent_read"),
        "Mirrored Recipe permission_scope should be updated, got: {}",
        recipe_scope
    );

    // Assert: original Recipe in FAMILY partition is UNTOUCHED
    let original = db
        .get_item(&format!("FAMILY#{fid}"), &format!("RECIPE#{}", recipe_id))
        .await
        .expect("Original Recipe should still exist");
    assert!(
        !original.contains_key("permission_scope"),
        "Original Recipe should not have permission_scope attribute"
    );
}

#[tokio::test]
#[ignore] // Requires DynamoDB Local (java)
async fn test_permission_update_targets_mirrored_meal_slots_and_feeding_logs() {
    let db = common::DynamoDbLocalInstance::start("perm-meal-test")
        .await
        .expect("DynamoDB Local required for this test");

    let family_id = uuid::Uuid::new_v4();
    let share_id = uuid::Uuid::new_v4();
    let accepter_id = "perm-accepter-2";
    let requester_id = "perm-requester-2";
    let dep_id = uuid::Uuid::new_v4();
    let meal_slot_id = uuid::Uuid::new_v4();
    let feeding_log_id = uuid::Uuid::new_v4();

    let share_id_str = share_id.to_string();
    let fid = family_id.to_string();
    let did = dep_id.to_string();
    let old_scope = r#"{"actions":["family_read"]}"#;
    let new_scope = PermissionScope {
        actions: vec![
            PermissionAction::FamilyRead,
            PermissionAction::DependentWrite,
        ],
    };

    // Seed: mirrored Family
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("FAMILY#{fid}"),
        &[
            ("id", &fid),
            ("share_id", &share_id_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Seed: mirrored Dependent
    db.put_item(make_item(
        &format!("FAMILY#{fid}"),
        &format!("DEPENDENT#{did}"),
        &[
            ("id", &did),
            ("family_id", &fid),
            ("share_id", &share_id_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Seed: mirrored MealSlot
    db.put_item(make_item(
        &format!("FAMILY#{fid}#DEPENDENT#{did}"),
        &format!("MEAL_SLOT#{}", meal_slot_id),
        &[
            ("id", &meal_slot_id.to_string()),
            ("family_id", &fid),
            ("share_id", &share_id_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Seed: mirrored FeedingLog
    db.put_item(make_item(
        &format!("FAMILY#{fid}#DEPENDENT#{did}"),
        &format!("FEEDING_LOG#{}", feeding_log_id),
        &[
            ("id", &feeding_log_id.to_string()),
            ("family_id", &fid),
            ("share_id", &share_id_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Act
    let share = make_test_share(share_id, family_id, requester_id, accepter_id, new_scope);
    handle_permission_updated(&db.client, &db.table_name, &share)
        .await
        .expect("handle_permission_updated should succeed");

    // Assert: MealSlot has updated scope
    let ms = db
        .get_item(
            &format!("FAMILY#{fid}#DEPENDENT#{did}"),
            &format!("MEAL_SLOT#{}", meal_slot_id),
        )
        .await
        .expect("MealSlot should still exist");
    let ms_scope = ms
        .get("permission_scope")
        .and_then(|v| v.as_s().ok())
        .unwrap();
    assert!(
        ms_scope.contains("dependent_write"),
        "MealSlot permission_scope should be updated, got: {}",
        ms_scope
    );

    // Assert: FeedingLog has updated scope
    let fl = db
        .get_item(
            &format!("FAMILY#{fid}#DEPENDENT#{did}"),
            &format!("FEEDING_LOG#{}", feeding_log_id),
        )
        .await
        .expect("FeedingLog should still exist");
    let fl_scope = fl
        .get("permission_scope")
        .and_then(|v| v.as_s().ok())
        .unwrap();
    assert!(
        fl_scope.contains("dependent_write"),
        "FeedingLog permission_scope should be updated, got: {}",
        fl_scope
    );
}

#[tokio::test]
#[ignore] // Requires DynamoDB Local (java)
async fn test_permission_update_does_not_affect_recipes_from_different_share() {
    let db = common::DynamoDbLocalInstance::start("perm-other-share-test")
        .await
        .expect("DynamoDB Local required for this test");

    let family_id = uuid::Uuid::new_v4();
    let share_id = uuid::Uuid::new_v4();
    let other_share_id = uuid::Uuid::new_v4();
    let accepter_id = "perm-accepter-3";
    let requester_id = "perm-requester-3";
    let recipe_id_ours = uuid::Uuid::new_v4();
    let recipe_id_theirs = uuid::Uuid::new_v4();

    let share_id_str = share_id.to_string();
    let other_share_str = other_share_id.to_string();
    let fid = family_id.to_string();
    let old_scope = r#"{"actions":["family_read"]}"#;
    let new_scope = PermissionScope {
        actions: vec![
            PermissionAction::FamilyRead,
            PermissionAction::DependentRead,
        ],
    };

    // Seed: mirrored Family
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("FAMILY#{fid}"),
        &[
            ("id", &fid),
            ("share_id", &share_id_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Seed: Recipe from our share
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("RECIPE#{}", recipe_id_ours),
        &[
            ("id", &recipe_id_ours.to_string()),
            ("family_id", &fid),
            ("share_id", &share_id_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Seed: Recipe from a different share (should NOT be updated)
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("RECIPE#{}", recipe_id_theirs),
        &[
            ("id", &recipe_id_theirs.to_string()),
            ("family_id", &fid),
            ("share_id", &other_share_str),
            ("permission_scope", old_scope),
        ],
    ))
    .await;

    // Act
    let share = make_test_share(share_id, family_id, requester_id, accepter_id, new_scope);
    handle_permission_updated(&db.client, &db.table_name, &share)
        .await
        .expect("handle_permission_updated should succeed");

    // Assert: our share's recipe was updated
    let ours = db
        .get_item(
            &format!("OWNER#{accepter_id}"),
            &format!("RECIPE#{}", recipe_id_ours),
        )
        .await
        .expect("Our recipe should still exist");
    let ours_scope = ours
        .get("permission_scope")
        .and_then(|v| v.as_s().ok())
        .unwrap();
    assert!(
        ours_scope.contains("dependent_read"),
        "Our recipe's scope should be updated"
    );

    // Assert: other share's recipe was NOT updated
    let theirs = db
        .get_item(
            &format!("OWNER#{accepter_id}"),
            &format!("RECIPE#{}", recipe_id_theirs),
        )
        .await
        .expect("Other recipe should still exist");
    let theirs_scope = theirs
        .get("permission_scope")
        .and_then(|v| v.as_s().ok())
        .unwrap();
    assert_eq!(
        theirs_scope, old_scope,
        "Other share's recipe scope should be unchanged"
    );
}

//! Integration tests for `handle_share_revoked`.
//!
//! These tests verify that revoking a share correctly deletes mirrored records
//! from the accepter's partition WITHOUT touching original records.
//!
//! Critical regression: mirrored Recipe records live at `OWNER#{accepter_id}` (rekeyed),
//! NOT at `FAMILY#{family_id}`. The revoke handler must query the correct partition.

mod common;

use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

use famtrac_backend::domain::{FamilyId, IdentityId, ShareId};
use famtrac_stream_handler::handlers::revoke::handle_share_revoked;

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

#[tokio::test]
#[ignore] // Requires DynamoDB Local (java)
async fn test_revoke_deletes_mirrored_recipes_from_accepter_partition() {
    let db = common::DynamoDbLocalInstance::start("revoke-recipe-test")
        .await
        .expect("DynamoDB Local required for this test");

    let family_id = uuid::Uuid::new_v4();
    let accepter_id = "accepter-user-1";
    let share_id = uuid::Uuid::new_v4();
    let recipe_id_1 = uuid::Uuid::new_v4();
    let recipe_id_2 = uuid::Uuid::new_v4();

    let share_id_str = share_id.to_string();
    let fid = family_id.to_string();

    // Seed: mirrored Family in accepter's partition
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("FAMILY#{fid}"),
        &[
            ("id", &fid),
            ("name", "Test Family"),
            ("share_id", &share_id_str),
        ],
    ))
    .await;

    // Seed: mirrored Recipes in accepter's OWNER partition (rekeyed by mirror handler)
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("RECIPE#{}", recipe_id_1),
        &[
            ("id", &recipe_id_1.to_string()),
            ("family_id", &fid),
            ("name", "Mirrored Recipe 1"),
            ("share_id", &share_id_str),
        ],
    ))
    .await;

    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("RECIPE#{}", recipe_id_2),
        &[
            ("id", &recipe_id_2.to_string()),
            ("family_id", &fid),
            ("name", "Mirrored Recipe 2"),
            ("share_id", &share_id_str),
        ],
    ))
    .await;

    // Seed: original Recipes in FAMILY partition (should NOT be deleted)
    db.put_item(make_item(
        &format!("FAMILY#{fid}"),
        &format!("RECIPE#{}", recipe_id_1),
        &[
            ("id", &recipe_id_1.to_string()),
            ("family_id", &fid),
            ("name", "Original Recipe 1"),
        ],
    ))
    .await;

    db.put_item(make_item(
        &format!("FAMILY#{fid}"),
        &format!("RECIPE#{}", recipe_id_2),
        &[
            ("id", &recipe_id_2.to_string()),
            ("family_id", &fid),
            ("name", "Original Recipe 2"),
        ],
    ))
    .await;

    // Act: revoke the share
    handle_share_revoked(
        &db.client,
        &db.table_name,
        &ShareId(share_id),
        &FamilyId(family_id),
        &IdentityId(accepter_id.to_string()),
    )
    .await
    .expect("handle_share_revoked should succeed");

    // Assert: mirrored Family record deleted
    let family = db
        .get_item(&format!("OWNER#{accepter_id}"), &format!("FAMILY#{fid}"))
        .await;
    assert!(family.is_none(), "Mirrored Family should be deleted");

    // Assert: mirrored Recipes deleted from OWNER#{accepter_id}
    let mirrored_recipes = db
        .query_items(&format!("OWNER#{accepter_id}"), "RECIPE#")
        .await;
    assert!(
        mirrored_recipes.is_empty(),
        "All mirrored Recipes should be deleted from accepter's partition, found: {:?}",
        mirrored_recipes.len()
    );

    // Assert: original Recipes in FAMILY#{fid} are UNTOUCHED
    let original_recipes = db.query_items(&format!("FAMILY#{fid}"), "RECIPE#").await;
    assert_eq!(
        original_recipes.len(),
        2,
        "Original Recipes must NOT be deleted; found {}",
        original_recipes.len()
    );
}

#[tokio::test]
#[ignore] // Requires DynamoDB Local (java)
async fn test_revoke_deletes_mirrored_dependents_and_activities() {
    let db = common::DynamoDbLocalInstance::start("revoke-dep-test")
        .await
        .expect("DynamoDB Local required for this test");

    let family_id = uuid::Uuid::new_v4();
    let accepter_id = "accepter-user-2";
    let share_id = uuid::Uuid::new_v4();
    let dep_id = uuid::Uuid::new_v4();
    let activity_id = uuid::Uuid::new_v4();
    let meal_slot_id = uuid::Uuid::new_v4();
    let feeding_log_id = uuid::Uuid::new_v4();

    let share_id_str = share_id.to_string();
    let fid = family_id.to_string();
    let did = dep_id.to_string();

    // Seed: mirrored Family
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("FAMILY#{fid}"),
        &[("id", &fid), ("share_id", &share_id_str)],
    ))
    .await;

    // Seed: mirrored Dependent (annotated, same PK as original)
    db.put_item(make_item(
        &format!("FAMILY#{fid}"),
        &format!("DEPENDENT#{did}"),
        &[
            ("id", &did),
            ("family_id", &fid),
            ("name", "Child"),
            ("share_id", &share_id_str),
        ],
    ))
    .await;

    // Seed: mirrored Activity
    db.put_item(make_item(
        &format!("FAMILY#{fid}#DEPENDENT#{did}"),
        &format!("ACTIVITY#{}", activity_id),
        &[
            ("id", &activity_id.to_string()),
            ("family_id", &fid),
            ("share_id", &share_id_str),
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
        ],
    ))
    .await;

    // Seed: NON-mirrored Activity (no share_id — should survive)
    let other_activity_id = uuid::Uuid::new_v4();
    db.put_item(make_item(
        &format!("FAMILY#{fid}#DEPENDENT#{did}"),
        &format!("ACTIVITY#{}", other_activity_id),
        &[("id", &other_activity_id.to_string()), ("family_id", &fid)],
    ))
    .await;

    // Act
    handle_share_revoked(
        &db.client,
        &db.table_name,
        &ShareId(share_id),
        &FamilyId(family_id),
        &IdentityId(accepter_id.to_string()),
    )
    .await
    .expect("handle_share_revoked should succeed");

    // Assert: mirrored Dependent deleted
    let deps = db.query_items(&format!("FAMILY#{fid}"), "DEPENDENT#").await;
    assert!(deps.is_empty(), "Mirrored Dependent should be deleted");

    // Assert: mirrored Activity deleted, non-mirrored survives
    let activities = db
        .query_items(&format!("FAMILY#{fid}#DEPENDENT#{did}"), "ACTIVITY#")
        .await;
    assert_eq!(
        activities.len(),
        1,
        "Only non-mirrored Activity should survive"
    );
    let surviving_id = activities[0].get("id").and_then(|v| v.as_s().ok()).unwrap();
    assert_eq!(surviving_id, &other_activity_id.to_string());

    // Assert: mirrored MealSlot deleted
    let meal_slots = db
        .query_items(&format!("FAMILY#{fid}#DEPENDENT#{did}"), "MEAL_SLOT#")
        .await;
    assert!(meal_slots.is_empty(), "Mirrored MealSlot should be deleted");

    // Assert: mirrored FeedingLog deleted
    let feeding_logs = db
        .query_items(&format!("FAMILY#{fid}#DEPENDENT#{did}"), "FEEDING_LOG#")
        .await;
    assert!(
        feeding_logs.is_empty(),
        "Mirrored FeedingLog should be deleted"
    );
}

#[tokio::test]
#[ignore] // Requires DynamoDB Local (java)
async fn test_revoke_does_not_delete_recipes_belonging_to_different_share() {
    let db = common::DynamoDbLocalInstance::start("revoke-other-share-test")
        .await
        .expect("DynamoDB Local required for this test");

    let family_id = uuid::Uuid::new_v4();
    let accepter_id = "accepter-user-3";
    let share_id = uuid::Uuid::new_v4();
    let other_share_id = uuid::Uuid::new_v4();
    let recipe_id_ours = uuid::Uuid::new_v4();
    let recipe_id_theirs = uuid::Uuid::new_v4();

    let share_id_str = share_id.to_string();
    let other_share_str = other_share_id.to_string();
    let fid = family_id.to_string();

    // Seed: mirrored Family
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("FAMILY#{fid}"),
        &[("id", &fid), ("share_id", &share_id_str)],
    ))
    .await;

    // Seed: Recipe belonging to OUR share (should be deleted)
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("RECIPE#{}", recipe_id_ours),
        &[
            ("id", &recipe_id_ours.to_string()),
            ("family_id", &fid),
            ("share_id", &share_id_str),
        ],
    ))
    .await;

    // Seed: Recipe belonging to a DIFFERENT share (should survive)
    db.put_item(make_item(
        &format!("OWNER#{accepter_id}"),
        &format!("RECIPE#{}", recipe_id_theirs),
        &[
            ("id", &recipe_id_theirs.to_string()),
            ("family_id", &fid),
            ("share_id", &other_share_str),
        ],
    ))
    .await;

    // Act
    handle_share_revoked(
        &db.client,
        &db.table_name,
        &ShareId(share_id),
        &FamilyId(family_id),
        &IdentityId(accepter_id.to_string()),
    )
    .await
    .expect("handle_share_revoked should succeed");

    // Assert: only the recipe belonging to our share was deleted
    let remaining = db
        .query_items(&format!("OWNER#{accepter_id}"), "RECIPE#")
        .await;
    assert_eq!(
        remaining.len(),
        1,
        "Only the other share's recipe should remain"
    );
    let remaining_id = remaining[0].get("id").and_then(|v| v.as_s().ok()).unwrap();
    assert_eq!(remaining_id, &recipe_id_theirs.to_string());
}

use aws_sdk_dynamodb::Client;
use lambda_runtime::Error;

use famtrac_backend::domain::{FamilyId, IdentityId, ShareId};

use crate::dynamo_util::{delete_item, query_items};

/// Handle a share revocation by deleting all mirrored records associated with
/// the revoked share.
///
/// This includes:
/// 1. The mirrored Family record in the accepter's OWNER partition
/// 2. All Recipe records for the family annotated with the share's `share_id`
/// 3. All Dependent records annotated with the share's `share_id`
/// 4. All Activity records for each dependent annotated with the share's `share_id`
/// 5. All `MealSlot` records for each dependent annotated with the share's `share_id`
/// 6. All `FeedingLog` records for each dependent annotated with the share's `share_id`
///
/// All deletes are idempotent — `delete_item` on a non-existent key succeeds
/// silently (Requirement 3.5).
///
/// # Errors
///
/// Returns an error if any `DynamoDB` operation fails.
#[allow(clippy::too_many_lines)]
#[allow(clippy::similar_names)]
pub async fn handle_share_revoked(
    client: &Client,
    table_name: &str,
    share_id: &ShareId,
    family_id: &FamilyId,
    accepter_id: &IdentityId,
) -> Result<(), Error> {
    let share_id_str = share_id.0.to_string();

    // 1. Delete the mirrored Family record from the accepter's partition.
    //    PK = OWNER#{accepter_id}, SK = FAMILY#{family_id}
    delete_item(
        client,
        table_name,
        &format!("OWNER#{}", accepter_id.0),
        &format!("FAMILY#{}", family_id.0),
    )
    .await?;

    // 1b. Delete all mirrored Recipe records for this family.
    //     Query recipes under FAMILY#{family_id} and delete those matching share_id.
    let recipes = query_items(
        client,
        table_name,
        &format!("FAMILY#{}", family_id.0),
        "RECIPE#",
    )
    .await?;

    for recipe_item in &recipes {
        let matches_share = recipe_item
            .get("share_id")
            .and_then(|v| v.as_s().ok())
            .is_some_and(|s| s == &share_id_str);

        if !matches_share {
            continue;
        }

        let recipe_pk = match recipe_item.get("PK").and_then(|v| v.as_s().ok()) {
            Some(pk) => pk.clone(),
            None => continue,
        };
        let recipe_sk = match recipe_item.get("SK").and_then(|v| v.as_s().ok()) {
            Some(sk) => sk.clone(),
            None => continue,
        };

        delete_item(client, table_name, &recipe_pk, &recipe_sk).await?;
    }

    // 2. Query all Dependents under FAMILY#{family_id} and delete those
    //    matching the revoked share_id.
    let dependents = query_items(
        client,
        table_name,
        &format!("FAMILY#{}", family_id.0),
        "DEPENDENT#",
    )
    .await?;

    let matching_dependents: Vec<_> = dependents
        .iter()
        .filter(|item| {
            item.get("share_id")
                .and_then(|v| v.as_s().ok())
                .is_some_and(|s| s == &share_id_str)
        })
        .collect();

    for dep_item in &matching_dependents {
        let dep_pk = match dep_item.get("PK").and_then(|v| v.as_s().ok()) {
            Some(pk) => pk.clone(),
            None => continue,
        };
        let dep_sk = match dep_item.get("SK").and_then(|v| v.as_s().ok()) {
            Some(sk) => sk.clone(),
            None => continue,
        };

        delete_item(client, table_name, &dep_pk, &dep_sk).await?;
    }

    // 3. For each matching dependent, query and delete all Activities with
    //    the same share_id.
    for dep_item in &matching_dependents {
        let dep_id = match dep_item.get("id").and_then(|v| v.as_s().ok()) {
            Some(id) => id.clone(),
            None => continue,
        };

        let activities = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", family_id.0, dep_id),
            "ACTIVITY#",
        )
        .await?;

        for act_item in &activities {
            let matches_share = act_item
                .get("share_id")
                .and_then(|v| v.as_s().ok())
                .is_some_and(|s| s == &share_id_str);

            if !matches_share {
                continue;
            }

            let act_pk = match act_item.get("PK").and_then(|v| v.as_s().ok()) {
                Some(pk) => pk.clone(),
                None => continue,
            };
            let act_sk = match act_item.get("SK").and_then(|v| v.as_s().ok()) {
                Some(sk) => sk.clone(),
                None => continue,
            };

            delete_item(client, table_name, &act_pk, &act_sk).await?;
        }
    }

    // 4. Delete all mirrored MealSlot records for each dependent.
    for dep_item in &matching_dependents {
        let dep_id = match dep_item.get("id").and_then(|v| v.as_s().ok()) {
            Some(id) => id.clone(),
            None => continue,
        };

        let meal_slots = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", family_id.0, dep_id),
            "MEAL_SLOT#",
        )
        .await?;

        for ms_item in &meal_slots {
            let matches_share = ms_item
                .get("share_id")
                .and_then(|v| v.as_s().ok())
                .is_some_and(|s| s == &share_id_str);

            if !matches_share {
                continue;
            }

            let ms_pk = match ms_item.get("PK").and_then(|v| v.as_s().ok()) {
                Some(pk) => pk.clone(),
                None => continue,
            };
            let ms_sk = match ms_item.get("SK").and_then(|v| v.as_s().ok()) {
                Some(sk) => sk.clone(),
                None => continue,
            };

            delete_item(client, table_name, &ms_pk, &ms_sk).await?;
        }
    }

    // 5. Delete all mirrored FeedingLog records for each dependent.
    for dep_item in &matching_dependents {
        let dep_id = match dep_item.get("id").and_then(|v| v.as_s().ok()) {
            Some(id) => id.clone(),
            None => continue,
        };

        let feeding_logs = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", family_id.0, dep_id),
            "FEEDING_LOG#",
        )
        .await?;

        for fl_item in &feeding_logs {
            let matches_share = fl_item
                .get("share_id")
                .and_then(|v| v.as_s().ok())
                .is_some_and(|s| s == &share_id_str);

            if !matches_share {
                continue;
            }

            let fl_pk = match fl_item.get("PK").and_then(|v| v.as_s().ok()) {
                Some(pk) => pk.clone(),
                None => continue,
            };
            let fl_sk = match fl_item.get("SK").and_then(|v| v.as_s().ok()) {
                Some(sk) => sk.clone(),
                None => continue,
            };

            delete_item(client, table_name, &fl_pk, &fl_sk).await?;
        }
    }

    Ok(())
}

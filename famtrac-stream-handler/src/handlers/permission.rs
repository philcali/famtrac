use aws_sdk_dynamodb::Client;
use lambda_runtime::Error;

use famtrac_backend::domain::Share;

use crate::dynamo_util::{conditional_update_permission, query_items};

/// Update the `permission_scope` attribute on all mirrored records that carry
/// the given share's `share_id`. This is triggered when a requester updates
/// the permission scope on an active share.
///
/// Updates:
/// 1. The mirrored Family record in the accepter's `OWNER#` partition
/// 2. All mirrored Recipe records under `FAMILY#{family_id}`
/// 3. All mirrored Dependent records under `FAMILY#{family_id}`
/// 4. All mirrored Activity records under `FAMILY#{family_id}#DEPENDENT#{dep_id}`
/// 5. All mirrored `MealSlot` records under `FAMILY#{family_id}#DEPENDENT#{dep_id}`
/// 6. All mirrored `FeedingLog` records under `FAMILY#{family_id}#DEPENDENT#{dep_id}`
///
/// Each update uses a condition expression `share_id = :sid` so that only
/// mirrored copies (not originals) are affected.
/// `ConditionalCheckFailedExceptions` are silently ignored (the record may not
/// exist or may not be mirrored).
///
/// # Errors
///
/// Returns an error if any `DynamoDB` operation fails.
#[allow(clippy::too_many_lines)]
pub async fn handle_permission_updated(
    client: &Client,
    table_name: &str,
    share: &Share,
) -> Result<(), Error> {
    let Some(accepter_id) = &share.accepter_id else {
        return Ok(()); // No accepter identity yet — nothing to update
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

    // 2. Update all mirrored Recipe records for this family
    let recipes = query_items(
        client,
        table_name,
        &format!("FAMILY#{}", share.family_id.0),
        "RECIPE#",
    )
    .await?;

    for recipe_item in &recipes {
        let recipe_sk = match recipe_item.get("SK").and_then(|v| v.as_s().ok()) {
            Some(sk) => sk.clone(),
            None => continue,
        };

        conditional_update_permission(
            client,
            table_name,
            &format!("FAMILY#{}", share.family_id.0),
            &recipe_sk,
            &share_id_str,
            &scope_json,
        )
        .await?;
    }

    // 3. Query all Dependents for this family and update mirrored ones
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

        // Update mirrored MealSlot records for this dependent
        let dep_id = match dep_item.get("id").and_then(|v| v.as_s().ok()) {
            Some(id) => id.clone(),
            None => continue,
        };

        let meal_slots = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
            "MEAL_SLOT#",
        )
        .await?;

        for ms_item in &meal_slots {
            let ms_sk = match ms_item.get("SK").and_then(|v| v.as_s().ok()) {
                Some(sk) => sk.clone(),
                None => continue,
            };

            conditional_update_permission(
                client,
                table_name,
                &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
                &ms_sk,
                &share_id_str,
                &scope_json,
            )
            .await?;
        }

        // Update mirrored FeedingLog records for this dependent
        let feeding_logs = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
            "FEEDING_LOG#",
        )
        .await?;

        for fl_item in &feeding_logs {
            let fl_sk = match fl_item.get("SK").and_then(|v| v.as_s().ok()) {
                Some(sk) => sk.clone(),
                None => continue,
            };

            conditional_update_permission(
                client,
                table_name,
                &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
                &fl_sk,
                &share_id_str,
                &scope_json,
            )
            .await?;
        }
    }

    Ok(())
}

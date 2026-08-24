use aws_sdk_dynamodb::types::AttributeValue as DdbAttributeValue;
use aws_sdk_dynamodb::Client;
use lambda_runtime::Error;

use famtrac_backend::domain::Share;

use crate::dynamo_util::{annotate_item, conditional_put, get_item, query_items, rekey_item};

/// Mirror the shared family, all its dependents, and all their activities into
/// the accepter's partition.
///
/// Each mirrored record is annotated with the share's `share_id` and
/// `permission_scope`. Conditional writes (`attribute_not_exists`) ensure
/// idempotency — duplicate stream deliveries are safely ignored.
///
/// Every mirrored item written is stamped with `sync_token` to identify it as
/// handler-originated and break infinite write-back cycles.
///
/// # Errors
///
/// Returns an error if any `DynamoDB` operation fails.
pub async fn handle_share_activated(
    client: &Client,
    table_name: &str,
    share: &Share,
    sync_token: &str,
) -> Result<(), Error> {
    let Some(accepter_id) = &share.accepter_id else {
        return Ok(()); // No accepter identity yet — nothing to mirror
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

    let Some(family_item) = family_item else {
        return Ok(()); // Family not found — nothing to mirror
    };

    // Mirror the family into the accepter's partition with rekeyed PK
    let mut mirrored_family = rekey_item(
        family_item,
        &format!("OWNER#{}", accepter_id.0),
        &share_id_str,
        &scope_json,
    );
    stamp_sync_token(&mut mirrored_family, sync_token);
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
        let mut mirrored_dep = annotate_item(dep_item.clone(), &share_id_str, &scope_json);
        stamp_sync_token(&mut mirrored_dep, sync_token);
        conditional_put(client, table_name, mirrored_dep).await?;
    }

    // 3. Mirror all recipes for this family (rekeyed into accepter's partition)
    let recipes = query_items(
        client,
        table_name,
        &format!("FAMILY#{}", share.family_id.0),
        "RECIPE#",
    )
    .await?;

    for recipe_item in &recipes {
        let mut mirrored_recipe = rekey_item(
            recipe_item.clone(),
            &format!("OWNER#{}", accepter_id.0),
            &share_id_str,
            &scope_json,
        );
        stamp_sync_token(&mut mirrored_recipe, sync_token);
        conditional_put(client, table_name, mirrored_recipe).await?;
    }

    // 4. For each dependent, fetch and mirror all activities
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
            let mut mirrored_act = annotate_item(act_item, &share_id_str, &scope_json);
            stamp_sync_token(&mut mirrored_act, sync_token);
            conditional_put(client, table_name, mirrored_act).await?;
        }

        // 5. Mirror all MealSlots for this dependent
        let meal_slots = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
            "MEAL_SLOT#",
        )
        .await?;

        for ms_item in &meal_slots {
            let mut mirrored_ms = annotate_item(ms_item.clone(), &share_id_str, &scope_json);
            stamp_sync_token(&mut mirrored_ms, sync_token);
            conditional_put(client, table_name, mirrored_ms).await?;
        }

        // 6. Mirror all FeedingLogs for this dependent
        let feeding_logs = query_items(
            client,
            table_name,
            &format!("FAMILY#{}#DEPENDENT#{}", share.family_id.0, dep_id),
            "FEEDING_LOG#",
        )
        .await?;

        for fl_item in &feeding_logs {
            let mut mirrored_fl = annotate_item(fl_item.clone(), &share_id_str, &scope_json);
            stamp_sync_token(&mut mirrored_fl, sync_token);
            conditional_put(client, table_name, mirrored_fl).await?;
        }
    }

    Ok(())
}

/// Stamp the `sync_token` attribute on a `DynamoDB` item.
fn stamp_sync_token(
    item: &mut std::collections::HashMap<String, DdbAttributeValue>,
    sync_token: &str,
) {
    item.insert(
        "sync_token".to_string(),
        DdbAttributeValue::S(sync_token.to_string()),
    );
}

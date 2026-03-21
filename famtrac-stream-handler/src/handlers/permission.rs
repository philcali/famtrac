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
/// 2. All mirrored Dependent records under `FAMILY#{family_id}`
/// 3. All mirrored Activity records under `FAMILY#{family_id}#DEPENDENT#{dep_id}`
///
/// Each update uses a condition expression `share_id = :sid` so that only
/// mirrored copies (not originals) are affected. ConditionalCheckFailedExceptions
/// are silently ignored (the record may not exist or may not be mirrored).
pub async fn handle_permission_updated(
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

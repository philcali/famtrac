use aws_sdk_dynamodb::Client;
use lambda_runtime::Error;

use famtrac_backend::domain::{FamilyId, IdentityId, ShareId};

use crate::dynamo_util::{delete_item, query_items};

/// Handle a share revocation by deleting all mirrored records associated with
/// the revoked share. This includes:
/// 1. The mirrored Family record in the accepter's OWNER partition
/// 2. All Dependent records annotated with the share's `share_id`
/// 3. All Activity records for each dependent annotated with the share's `share_id`
///
/// All deletes are idempotent — `delete_item` on a non-existent key succeeds
/// silently (Requirement 3.5).
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
                .map(|s| s == &share_id_str)
                .unwrap_or(false)
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
                .map(|s| s == &share_id_str)
                .unwrap_or(false);

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

    Ok(())
}

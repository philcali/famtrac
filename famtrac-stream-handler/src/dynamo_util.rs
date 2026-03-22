use aws_sdk_dynamodb::types::AttributeValue as DdbAttributeValue;
use aws_sdk_dynamodb::Client;
use lambda_runtime::Error;
use std::collections::HashMap;

use famtrac_backend::domain::Share;

use crate::parser::parse_share_from_attrs;

/// Fetch a single item by PK and SK.
pub async fn get_item(
    client: &Client,
    table_name: &str,
    pk: &str,
    sk: &str,
) -> Result<Option<HashMap<String, DdbAttributeValue>>, Error> {
    let result = client
        .get_item()
        .table_name(table_name)
        .key("PK", DdbAttributeValue::S(pk.to_string()))
        .key("SK", DdbAttributeValue::S(sk.to_string()))
        .send()
        .await?;

    Ok(result.item)
}

/// Query all items matching a PK and SK prefix.
pub async fn query_items(
    client: &Client,
    table_name: &str,
    pk: &str,
    sk_prefix: &str,
) -> Result<Vec<HashMap<String, DdbAttributeValue>>, Error> {
    let result = client
        .query()
        .table_name(table_name)
        .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
        .expression_attribute_values(":pk", DdbAttributeValue::S(pk.to_string()))
        .expression_attribute_values(":sk_prefix", DdbAttributeValue::S(sk_prefix.to_string()))
        .send()
        .await?;

    Ok(result.items.unwrap_or_default())
}

/// Put an item, overwriting any existing item at the same PK/SK.
pub async fn put_item(
    client: &Client,
    table_name: &str,
    item: HashMap<String, DdbAttributeValue>,
) -> Result<(), Error> {
    client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await?;
    Ok(())
}

/// Delete an item by PK and SK.
pub async fn delete_item(
    client: &Client,
    table_name: &str,
    pk: &str,
    sk: &str,
) -> Result<(), Error> {
    client
        .delete_item()
        .table_name(table_name)
        .key("PK", DdbAttributeValue::S(pk.to_string()))
        .key("SK", DdbAttributeValue::S(sk.to_string()))
        .send()
        .await?;
    Ok(())
}

/// Put an item with a condition that it doesn't already exist (idempotent write).
/// If the item already exists, the ConditionalCheckFailedException is silently ignored.
pub async fn conditional_put(
    client: &Client,
    table_name: &str,
    item: HashMap<String, DdbAttributeValue>,
) -> Result<(), Error> {
    let result = client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .condition_expression("attribute_not_exists(PK) AND attribute_not_exists(SK)")
        .send()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            if err
                .as_service_error()
                .map(|e| e.is_conditional_check_failed_exception())
                .unwrap_or(false)
            {
                Ok(())
            } else {
                Err(Box::new(err))
            }
        }
    }
}

/// Update the `permission_scope` attribute on a single item, conditioned on
/// `share_id = :sid`. Silently ignores ConditionalCheckFailedException (the
/// item may not exist or may not be a mirrored copy for this share).
pub async fn conditional_update_permission(
    client: &Client,
    table_name: &str,
    pk: &str,
    sk: &str,
    share_id: &str,
    scope_json: &str,
) -> Result<(), Error> {
    let result = client
        .update_item()
        .table_name(table_name)
        .key("PK", DdbAttributeValue::S(pk.to_string()))
        .key("SK", DdbAttributeValue::S(sk.to_string()))
        .update_expression("SET permission_scope = :new_scope")
        .condition_expression("share_id = :sid")
        .expression_attribute_values(":sid", DdbAttributeValue::S(share_id.to_string()))
        .expression_attribute_values(":new_scope", DdbAttributeValue::S(scope_json.to_string()))
        .send()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            if err
                .as_service_error()
                .map(|e| e.is_conditional_check_failed_exception())
                .unwrap_or(false)
            {
                Ok(())
            } else {
                Err(Box::new(err))
            }
        }
    }
}

/// Rekey an item's PK to a new value and annotate with share metadata.
/// Used for Family records that need to move into the accepter's OWNER partition.
pub fn rekey_item(
    mut item: HashMap<String, DdbAttributeValue>,
    new_pk: &str,
    share_id: &str,
    scope_json: &str,
) -> HashMap<String, DdbAttributeValue> {
    item.insert("PK".to_string(), DdbAttributeValue::S(new_pk.to_string()));
    item.insert(
        "share_id".to_string(),
        DdbAttributeValue::S(share_id.to_string()),
    );
    item.insert(
        "permission_scope".to_string(),
        DdbAttributeValue::S(scope_json.to_string()),
    );
    item
}

/// Annotate an item with share metadata without changing its PK/SK.
/// Used for Dependent and Activity records that keep their original keys.
pub fn annotate_item(
    mut item: HashMap<String, DdbAttributeValue>,
    share_id: &str,
    scope_json: &str,
) -> HashMap<String, DdbAttributeValue> {
    item.insert(
        "share_id".to_string(),
        DdbAttributeValue::S(share_id.to_string()),
    );
    item.insert(
        "permission_scope".to_string(),
        DdbAttributeValue::S(scope_json.to_string()),
    );
    item
}

/// Convert a `serde_dynamo::Item` to a `HashMap<String, DdbAttributeValue>` for use
/// with the AWS SDK DynamoDB client.
pub fn convert_image(image: &serde_dynamo::Item) -> HashMap<String, DdbAttributeValue> {
    image
        .inner()
        .iter()
        .filter_map(|(k, v)| {
            let ddb_val = match v {
                serde_dynamo::AttributeValue::S(s) => Some(DdbAttributeValue::S(s.clone())),
                serde_dynamo::AttributeValue::N(n) => Some(DdbAttributeValue::N(n.clone())),
                serde_dynamo::AttributeValue::Bool(b) => Some(DdbAttributeValue::Bool(*b)),
                serde_dynamo::AttributeValue::Null(true) => Some(DdbAttributeValue::Null(true)),
                _ => None,
            };
            ddb_val.map(|v| (k.clone(), v))
        })
        .collect()
}

/// Extract the family_id from a resource's DynamoDB image or PK.
///
/// For Family records: read the `id` attribute (the family_id IS the record id).
/// For Dependent records: read the `family_id` attribute, or parse from PK `FAMILY#{fid}`.
/// For Activity records: read the `family_id` attribute, or parse from PK `FAMILY#{fid}#DEPENDENT#{did}`.
pub fn extract_family_id(
    pk: &str,
    sk: &str,
    image: &HashMap<String, DdbAttributeValue>,
) -> Option<String> {
    // Try the image's family_id attribute first (present on Dependent/Activity)
    if let Some(fid) = image.get("family_id").and_then(|v| v.as_s().ok()) {
        return Some(fid.clone());
    }

    // For Family records, the id attribute IS the family_id
    if sk.starts_with("FAMILY#") {
        if let Some(id) = image.get("id").and_then(|v| v.as_s().ok()) {
            return Some(id.clone());
        }
        // Fallback: parse from SK
        return sk.strip_prefix("FAMILY#").map(|s| s.to_string());
    }

    // For Dependent records: PK = FAMILY#{fid}
    if pk.starts_with("FAMILY#") && !pk.contains("#DEPENDENT#") {
        return pk.strip_prefix("FAMILY#").map(|s| s.to_string());
    }

    // For Activity records: PK = FAMILY#{fid}#DEPENDENT#{did}
    if pk.starts_with("FAMILY#") && pk.contains("#DEPENDENT#") {
        let after_family = pk.strip_prefix("FAMILY#")?;
        let fid = after_family.split("#DEPENDENT#").next()?;
        return Some(fid.to_string());
    }

    None
}

/// Extract the owner_id from a resource's DynamoDB image or PK.
/// For Family records: PK = OWNER#{owner_id}, or read `owner_id` attribute.
/// For Dependent/Activity records: read `owner_id` attribute if present, otherwise
/// we need to look up the family to find the owner.
pub fn extract_owner_id(pk: &str, image: &HashMap<String, DdbAttributeValue>) -> Option<String> {
    // Try the image's owner_id attribute first
    if let Some(oid) = image.get("owner_id").and_then(|v| v.as_s().ok()) {
        return Some(oid.clone());
    }

    // For Family records: PK = OWNER#{owner_id}
    if pk.starts_with("OWNER#") {
        return pk.strip_prefix("OWNER#").map(|s| s.to_string());
    }

    None
}

/// Check if a resource record is a mirrored copy (has a share_id attribute).
pub fn is_mirrored_resource(image: &HashMap<String, DdbAttributeValue>) -> bool {
    image.get("share_id").and_then(|v| v.as_s().ok()).is_some()
}

/// Query the `GSI-family_id` GSI to find all active shares for a given family.
/// This replaces both `find_owner_for_family` (table scan) and
/// `find_active_shares_for_family` (owner-partition query).
pub async fn find_active_shares_by_family_id(
    client: &Client,
    table_name: &str,
    family_id: &str,
) -> Result<Vec<Share>, Error> {
    let result = client
        .query()
        .table_name(table_name)
        .index_name("GSI-family_id")
        .key_condition_expression("family_id = :fid AND begins_with(SK, :sk_prefix)")
        .filter_expression("#st = :active")
        .expression_attribute_names("#st", "status")
        .expression_attribute_values(":fid", DdbAttributeValue::S(family_id.to_string()))
        .expression_attribute_values(":sk_prefix", DdbAttributeValue::S("SHARE#".to_string()))
        .expression_attribute_values(":active", DdbAttributeValue::S("active".to_string()))
        .send()
        .await?;
    let items = result.items.unwrap_or_default();
    Ok(items.iter().filter_map(parse_share_from_attrs).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Tests for extract_family_id
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_family_id_from_family_record() {
        let pk = "OWNER#user1";
        let fid = uuid::Uuid::new_v4().to_string();
        let sk = format!("FAMILY#{}", fid);
        let mut image = HashMap::new();
        image.insert("id".to_string(), DdbAttributeValue::S(fid.clone()));
        assert_eq!(extract_family_id(pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_family_id_from_dependent_record() {
        let fid = uuid::Uuid::new_v4().to_string();
        let pk = format!("FAMILY#{}", fid);
        let sk = format!("DEPENDENT#{}", uuid::Uuid::new_v4());
        let mut image = HashMap::new();
        image.insert("family_id".to_string(), DdbAttributeValue::S(fid.clone()));
        assert_eq!(extract_family_id(&pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_family_id_from_activity_record() {
        let fid = uuid::Uuid::new_v4().to_string();
        let did = uuid::Uuid::new_v4().to_string();
        let pk = format!("FAMILY#{}#DEPENDENT#{}", fid, did);
        let sk = format!("ACTIVITY#{}", uuid::Uuid::new_v4());
        let mut image = HashMap::new();
        image.insert("family_id".to_string(), DdbAttributeValue::S(fid.clone()));
        assert_eq!(extract_family_id(&pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_family_id_from_pk_fallback() {
        let fid = uuid::Uuid::new_v4().to_string();
        let pk = format!("FAMILY#{}", fid);
        let sk = format!("DEPENDENT#{}", uuid::Uuid::new_v4());
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert_eq!(extract_family_id(&pk, &sk, &image), Some(fid));
    }

    #[test]
    fn test_extract_family_id_from_activity_pk_fallback() {
        let fid = uuid::Uuid::new_v4().to_string();
        let did = uuid::Uuid::new_v4().to_string();
        let pk = format!("FAMILY#{}#DEPENDENT#{}", fid, did);
        let sk = format!("ACTIVITY#{}", uuid::Uuid::new_v4());
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert_eq!(extract_family_id(&pk, &sk, &image), Some(fid));
    }

    // -----------------------------------------------------------------------
    // Tests for extract_owner_id
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_owner_id_from_family_pk() {
        let pk = "OWNER#user1";
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert_eq!(extract_owner_id(pk, &image), Some("user1".to_string()));
    }

    #[test]
    fn test_extract_owner_id_from_image_attribute() {
        let pk = "FAMILY#some-fid";
        let mut image = HashMap::new();
        image.insert(
            "owner_id".to_string(),
            DdbAttributeValue::S("user2".to_string()),
        );
        assert_eq!(extract_owner_id(pk, &image), Some("user2".to_string()));
    }

    #[test]
    fn test_extract_owner_id_none_for_dependent_without_attr() {
        let pk = "FAMILY#some-fid";
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert_eq!(extract_owner_id(pk, &image), None);
    }

    // -----------------------------------------------------------------------
    // Tests for is_mirrored_resource
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_mirrored_resource_true() {
        let mut image = HashMap::new();
        image.insert(
            "share_id".to_string(),
            DdbAttributeValue::S("some-share-id".to_string()),
        );
        assert!(is_mirrored_resource(&image));
    }

    #[test]
    fn test_is_mirrored_resource_false() {
        let image: HashMap<String, DdbAttributeValue> = HashMap::new();
        assert!(!is_mirrored_resource(&image));
    }

    // -----------------------------------------------------------------------
    // Tests for rekey_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_rekey_item_sets_pk_and_share_metadata() {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            DdbAttributeValue::S("OWNER#original".to_string()),
        );
        item.insert(
            "SK".to_string(),
            DdbAttributeValue::S("FAMILY#fid".to_string()),
        );
        item.insert(
            "name".to_string(),
            DdbAttributeValue::S("Test Family".to_string()),
        );

        let result = rekey_item(
            item,
            "OWNER#accepter",
            "share-123",
            r#"{"actions":["family_read"]}"#,
        );

        assert_eq!(
            result.get("PK").and_then(|v| v.as_s().ok()),
            Some(&"OWNER#accepter".to_string())
        );
        assert_eq!(
            result.get("SK").and_then(|v| v.as_s().ok()),
            Some(&"FAMILY#fid".to_string())
        );
        assert_eq!(
            result.get("share_id").and_then(|v| v.as_s().ok()),
            Some(&"share-123".to_string())
        );
        assert_eq!(
            result.get("permission_scope").and_then(|v| v.as_s().ok()),
            Some(&r#"{"actions":["family_read"]}"#.to_string())
        );
        // Original data preserved
        assert_eq!(
            result.get("name").and_then(|v| v.as_s().ok()),
            Some(&"Test Family".to_string())
        );
    }

    // -----------------------------------------------------------------------
    // Tests for annotate_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_annotate_item_adds_share_metadata() {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            DdbAttributeValue::S("FAMILY#fid".to_string()),
        );
        item.insert(
            "SK".to_string(),
            DdbAttributeValue::S("DEPENDENT#did".to_string()),
        );
        item.insert(
            "name".to_string(),
            DdbAttributeValue::S("Child".to_string()),
        );

        let result = annotate_item(item, "share-456", r#"{"actions":["family_read"]}"#);

        // PK/SK unchanged
        assert_eq!(
            result.get("PK").and_then(|v| v.as_s().ok()),
            Some(&"FAMILY#fid".to_string())
        );
        assert_eq!(
            result.get("SK").and_then(|v| v.as_s().ok()),
            Some(&"DEPENDENT#did".to_string())
        );
        // Share metadata added
        assert_eq!(
            result.get("share_id").and_then(|v| v.as_s().ok()),
            Some(&"share-456".to_string())
        );
        assert_eq!(
            result.get("permission_scope").and_then(|v| v.as_s().ok()),
            Some(&r#"{"actions":["family_read"]}"#.to_string())
        );
        // Original data preserved
        assert_eq!(
            result.get("name").and_then(|v| v.as_s().ok()),
            Some(&"Child".to_string())
        );
    }

    // -----------------------------------------------------------------------
    // Tests for convert_image
    // -----------------------------------------------------------------------

    #[test]
    fn test_convert_image_string_values() {
        let mut m = HashMap::new();
        m.insert(
            "PK".to_string(),
            serde_dynamo::AttributeValue::S("OWNER#user1".to_string()),
        );
        m.insert(
            "SK".to_string(),
            serde_dynamo::AttributeValue::S("FAMILY#fid".to_string()),
        );
        let image = serde_dynamo::Item::from(m);
        let result = convert_image(&image);
        assert_eq!(
            result.get("PK").and_then(|v| v.as_s().ok()),
            Some(&"OWNER#user1".to_string())
        );
        assert_eq!(
            result.get("SK").and_then(|v| v.as_s().ok()),
            Some(&"FAMILY#fid".to_string())
        );
    }

    #[test]
    fn test_convert_image_bool_and_number() {
        let mut m = HashMap::new();
        m.insert(
            "count".to_string(),
            serde_dynamo::AttributeValue::N("42".to_string()),
        );
        m.insert(
            "active".to_string(),
            serde_dynamo::AttributeValue::Bool(true),
        );
        let image = serde_dynamo::Item::from(m);
        let result = convert_image(&image);
        assert_eq!(
            result.get("count").and_then(|v| v.as_n().ok()),
            Some(&"42".to_string())
        );
        assert_eq!(
            result.get("active").and_then(|v| v.as_bool().ok()),
            Some(&true)
        );
    }
}

use aws_sdk_dynamodb::types::AttributeValue as DdbAttributeValue;
use std::collections::HashMap;

use famtrac_backend::domain::{
    FamilyId, IdentityId, PermissionScope, Share, ShareId, ShareStatus, Timestamp,
};

/// Extract a string value from a `serde_dynamo::Item` by attribute name.
#[must_use]
pub fn get_str(item: &serde_dynamo::Item, key: &str) -> Option<String> {
    match item.inner().get(key)? {
        serde_dynamo::AttributeValue::S(s) => Some(s.clone()),
        _ => None,
    }
}

/// Parse a `Share` from a `serde_dynamo::Item` (stream image).
#[must_use]
pub fn parse_share(image: &serde_dynamo::Item) -> Option<Share> {
    let mut map = HashMap::new();
    for (k, v) in image.inner() {
        if let serde_dynamo::AttributeValue::S(s) = v {
            map.insert(k.clone(), s.clone());
        }
    }
    parse_share_from_strings(&map)
}

/// Parse a `Share` from a raw `DynamoDB` SDK `HashMap<String, DdbAttributeValue>`.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn parse_share_from_attrs(item: &HashMap<String, DdbAttributeValue>) -> Option<Share> {
    let mut map = HashMap::new();
    for (k, v) in item {
        if let Ok(s) = v.as_s() {
            map.insert(k.clone(), s.clone());
        }
    }
    parse_share_from_strings(&map)
}

/// Shared parsing logic: parse a `Share` from a `HashMap<String, String>`.
fn parse_share_from_strings(map: &HashMap<String, String>) -> Option<Share> {
    let id = map.get("id")?.parse().ok()?;
    let family_id = map.get("family_id")?.parse().ok()?;
    let requester_id = map.get("requester_id")?.clone();
    let accepter_username = map.get("accepter_username")?.clone();
    let accepter_id = map.get("accepter_id").cloned().map(IdentityId);

    let scope_json = map.get("permission_scope")?;
    let permission_scope: PermissionScope = serde_json::from_str(scope_json).ok()?;

    let status_str = map.get("status")?;
    let status: ShareStatus = serde_json::from_str(&format!("\"{status_str}\"")).ok()?;

    let created_at = map
        .get("created_at")?
        .parse::<chrono::DateTime<chrono::Utc>>()
        .ok()?;
    let updated_at = map
        .get("updated_at")?
        .parse::<chrono::DateTime<chrono::Utc>>()
        .ok()?;
    let expires_at = map
        .get("expires_at")
        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .map(Timestamp::from_datetime);

    Some(Share {
        id: ShareId(id),
        family_id: FamilyId(family_id),
        requester_id: IdentityId(requester_id),
        accepter_username,
        accepter_id,
        permission_scope,
        status,
        created_at: Timestamp::from_datetime(created_at),
        updated_at: Timestamp::from_datetime(updated_at),
        expires_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_dynamo::AttributeValue;

    fn s_ddb(val: &str) -> DdbAttributeValue {
        DdbAttributeValue::S(val.to_string())
    }

    fn s_serde(val: &str) -> AttributeValue {
        AttributeValue::S(val.to_string())
    }

    const SCOPE_JSON: &str = r#"{"actions":["family_read"]}"#;

    fn valid_share_attrs() -> HashMap<String, DdbAttributeValue> {
        let mut m = HashMap::new();
        m.insert("id".to_string(), s_ddb(&uuid::Uuid::new_v4().to_string()));
        m.insert(
            "family_id".to_string(),
            s_ddb(&uuid::Uuid::new_v4().to_string()),
        );
        m.insert("requester_id".to_string(), s_ddb("owner1"));
        m.insert("accepter_username".to_string(), s_ddb("a@b.com"));
        m.insert("status".to_string(), s_ddb("active"));
        m.insert("permission_scope".to_string(), s_ddb(SCOPE_JSON));
        m.insert("created_at".to_string(), s_ddb("2025-01-01T00:00:00Z"));
        m.insert("updated_at".to_string(), s_ddb("2025-01-01T00:00:00Z"));
        m
    }

    fn valid_share_serde_image() -> serde_dynamo::Item {
        let mut m = HashMap::new();
        m.insert("id".to_string(), s_serde(&uuid::Uuid::new_v4().to_string()));
        m.insert(
            "family_id".to_string(),
            s_serde(&uuid::Uuid::new_v4().to_string()),
        );
        m.insert("requester_id".to_string(), s_serde("owner1"));
        m.insert("accepter_username".to_string(), s_serde("a@b.com"));
        m.insert("status".to_string(), s_serde("active"));
        m.insert("permission_scope".to_string(), s_serde(SCOPE_JSON));
        m.insert("created_at".to_string(), s_serde("2025-01-01T00:00:00Z"));
        m.insert("updated_at".to_string(), s_serde("2025-01-01T00:00:00Z"));
        serde_dynamo::Item::from(m)
    }

    #[test]
    fn test_parse_share_from_attrs_valid() {
        let attrs = valid_share_attrs();
        let share = parse_share_from_attrs(&attrs);
        assert!(share.is_some());
        let share = share.unwrap();
        assert_eq!(share.status, ShareStatus::Active);
    }

    #[test]
    fn test_parse_share_valid() {
        let image = valid_share_serde_image();
        let share = parse_share(&image);
        assert!(share.is_some());
        let share = share.unwrap();
        assert_eq!(share.status, ShareStatus::Active);
    }

    #[test]
    fn test_parse_share_from_attrs_missing_id() {
        let mut attrs = valid_share_attrs();
        attrs.remove("id");
        assert!(parse_share_from_attrs(&attrs).is_none());
    }

    #[test]
    fn test_parse_share_from_attrs_missing_family_id() {
        let mut attrs = valid_share_attrs();
        attrs.remove("family_id");
        assert!(parse_share_from_attrs(&attrs).is_none());
    }

    #[test]
    fn test_parse_share_from_attrs_missing_status() {
        let mut attrs = valid_share_attrs();
        attrs.remove("status");
        assert!(parse_share_from_attrs(&attrs).is_none());
    }

    #[test]
    fn test_parse_share_missing_requester_id() {
        let mut m: HashMap<String, AttributeValue> = HashMap::new();
        m.insert("id".to_string(), s_serde(&uuid::Uuid::new_v4().to_string()));
        m.insert(
            "family_id".to_string(),
            s_serde(&uuid::Uuid::new_v4().to_string()),
        );
        m.insert("accepter_username".to_string(), s_serde("a@b.com"));
        m.insert("status".to_string(), s_serde("active"));
        m.insert("permission_scope".to_string(), s_serde(SCOPE_JSON));
        m.insert("created_at".to_string(), s_serde("2025-01-01T00:00:00Z"));
        m.insert("updated_at".to_string(), s_serde("2025-01-01T00:00:00Z"));
        let image = serde_dynamo::Item::from(m);
        assert!(parse_share(&image).is_none());
    }

    #[test]
    fn test_parse_share_optional_accepter_id() {
        let attrs = valid_share_attrs();
        let share = parse_share_from_attrs(&attrs).unwrap();
        assert!(share.accepter_id.is_none());

        let mut attrs_with_accepter = valid_share_attrs();
        attrs_with_accepter.insert("accepter_id".to_string(), s_ddb("accepter1"));
        let share = parse_share_from_attrs(&attrs_with_accepter).unwrap();
        assert_eq!(share.accepter_id, Some(IdentityId("accepter1".to_string())));
    }

    #[test]
    fn test_get_str_extracts_string() {
        let mut m = HashMap::new();
        m.insert("key".to_string(), s_serde("value"));
        let item = serde_dynamo::Item::from(m);
        assert_eq!(get_str(&item, "key"), Some("value".to_string()));
    }

    #[test]
    fn test_get_str_returns_none_for_missing() {
        let m: HashMap<String, AttributeValue> = HashMap::new();
        let item = serde_dynamo::Item::from(m);
        assert_eq!(get_str(&item, "key"), None);
    }
}

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use std::collections::HashMap;

use crate::domain::{
    Activity, ActivityId, ActivityType, Date, Dependent, DependentId, Family, FamilyId, IdentityId,
    PermissionScope, Share, ShareId, ShareStatus, Timestamp,
};
use crate::errors::StoreError;
use crate::handlers::{PaginatedResponse, PaginationParams};

use super::traits::{
    ActivityQueryParams, ActivityRepository, DependentRepository, FamilyRepository, ShareRepository,
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

/// Decode a pagination `next_token` (base64-encoded JSON) into a DynamoDB
/// `ExclusiveStartKey` map.  Returns `Ok(None)` when the token is absent.
fn decode_next_token(
    next_token: &Option<String>,
) -> Result<Option<HashMap<String, AttributeValue>>, StoreError> {
    let token = match next_token {
        Some(t) => t,
        None => return Ok(None),
    };

    let json_bytes = BASE64.decode(token).map_err(|e| {
        StoreError::QueryError(format!("Invalid pagination token (base64 decode): {}", e))
    })?;

    let map: HashMap<String, serde_json::Value> =
        serde_json::from_slice(&json_bytes).map_err(|e| {
            StoreError::QueryError(format!("Invalid pagination token (JSON decode): {}", e))
        })?;

    let dynamo_map: HashMap<String, AttributeValue> = map
        .into_iter()
        .map(|(k, v)| {
            let av = match v {
                serde_json::Value::String(s) => AttributeValue::S(s),
                serde_json::Value::Number(n) => AttributeValue::N(n.to_string()),
                other => AttributeValue::S(other.to_string()),
            };
            (k, av)
        })
        .collect();

    Ok(Some(dynamo_map))
}

/// Encode a DynamoDB `LastEvaluatedKey` map into a base64-encoded JSON string
/// suitable for use as a pagination `next_token`.  Returns `Ok(None)` when
/// there is no key (i.e. last page).
fn encode_last_evaluated_key(
    last_evaluated_key: &Option<HashMap<String, AttributeValue>>,
) -> Result<Option<String>, StoreError> {
    let key = match last_evaluated_key {
        Some(k) => k,
        None => return Ok(None),
    };

    let json_map: HashMap<String, serde_json::Value> = key
        .iter()
        .map(|(k, v)| {
            let json_val = match v.as_s() {
                Ok(s) => serde_json::Value::String(s.clone()),
                Err(_) => match v.as_n() {
                    Ok(n) => serde_json::json!(n.parse::<f64>().unwrap_or(0.0)),
                    Err(_) => serde_json::Value::String(format!("{:?}", v)),
                },
            };
            (k.clone(), json_val)
        })
        .collect();

    let json_bytes = serde_json::to_vec(&json_map)
        .map_err(|e| StoreError::QueryError(format!("Failed to encode pagination token: {}", e)))?;

    Ok(Some(BASE64.encode(&json_bytes)))
}

/// DynamoDB implementation of FamilyRepository
#[derive(Clone)]
pub struct DynamoDbFamilyRepository {
    client: Client,
    table_name: String,
}

impl DynamoDbFamilyRepository {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    /// Convert Family to DynamoDB item
    fn to_item(&self, family: &Family) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("OWNER#{}", family.owner_id.0)),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("FAMILY#{}", family.id.0)),
        );
        item.insert("Type".to_string(), AttributeValue::S("Family".to_string()));
        item.insert("id".to_string(), AttributeValue::S(family.id.0.to_string()));
        item.insert("name".to_string(), AttributeValue::S(family.name.clone()));
        item.insert(
            "owner_id".to_string(),
            AttributeValue::S(family.owner_id.0.clone()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(family.created_at.0.to_rfc3339()),
        );
        item.insert(
            "updated_at".to_string(),
            AttributeValue::S(family.updated_at.0.to_rfc3339()),
        );
        if let Some(ref share_id) = family.share_id {
            item.insert(
                "share_id".to_string(),
                AttributeValue::S(share_id.0.to_string()),
            );
        }
        if let Some(ref permission_scope) = family.permission_scope {
            let scope_json =
                serde_json::to_string(permission_scope).unwrap_or_else(|_| "{}".to_string());
            item.insert(
                "permission_scope".to_string(),
                AttributeValue::S(scope_json),
            );
        }
        item
    }

    /// Convert DynamoDB item to Family
    fn parse_item(&self, item: &HashMap<String, AttributeValue>) -> Result<Family, StoreError> {
        let id = item
            .get("id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid id".to_string()))?;

        let name = item
            .get("name")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| StoreError::QueryError("Missing name".to_string()))?
            .clone();

        let owner_id = item
            .get("owner_id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| StoreError::QueryError("Missing owner_id".to_string()))?
            .clone();

        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid created_at".to_string()))?;

        let updated_at = item
            .get("updated_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid updated_at".to_string()))?;

        let share_id = item
            .get("share_id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .map(ShareId);

        let permission_scope = item
            .get("permission_scope")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| serde_json::from_str::<PermissionScope>(s).ok());

        Ok(Family {
            id: FamilyId(id),
            name,
            owner_id: IdentityId(owner_id),
            created_at: Timestamp::from_datetime(created_at),
            updated_at: Timestamp::from_datetime(updated_at),
            share_id,
            permission_scope,
        })
    }
}

#[async_trait]
impl FamilyRepository for DynamoDbFamilyRepository {
    async fn create(&self, family: Family) -> Result<Family, StoreError> {
        let item = self.to_item(&family);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to create family: {}", e)))?;

        Ok(family)
    }

    async fn get(&self, owner_id: IdentityId, id: FamilyId) -> Result<Option<Family>, StoreError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("OWNER#{}", owner_id.0)))
            .key("SK", AttributeValue::S(format!("FAMILY#{}", id.0)))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to get family: {}", e)))?;

        match result.item {
            Some(item) => Ok(Some(self.parse_item(&item)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, family: Family) -> Result<Family, StoreError> {
        let item = self.to_item(&family);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to update family: {}", e)))?;

        Ok(family)
    }

    async fn get_by_owner(
        &self,
        owner_id: IdentityId,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Family>, StoreError> {
        let exclusive_start_key = decode_next_token(&pagination.next_token)?;

        let mut query = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(":pk", AttributeValue::S(format!("OWNER#{}", owner_id.0)))
            .expression_attribute_values(":sk_prefix", AttributeValue::S("FAMILY#".to_string()))
            .limit(pagination.effective_limit() as i32);

        if let Some(start_key) = exclusive_start_key {
            query = query.set_exclusive_start_key(Some(start_key));
        }

        let result = query.send().await.map_err(|e| {
            StoreError::QueryError(format!("Failed to query families by owner: {}", e))
        })?;

        let families = result
            .items
            .unwrap_or_default()
            .iter()
            .map(|item| self.parse_item(item))
            .collect::<Result<Vec<_>, _>>()?;

        let next_token = encode_last_evaluated_key(&result.last_evaluated_key)?;

        Ok(PaginatedResponse::with_next_token(families, next_token))
    }

    async fn delete(&self, owner_id: IdentityId, id: FamilyId) -> Result<(), StoreError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("OWNER#{}", owner_id.0)))
            .key("SK", AttributeValue::S(format!("FAMILY#{}", id.0)))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to delete family: {}", e)))?;

        Ok(())
    }
}

/// DynamoDB implementation of DependentRepository
#[derive(Clone)]
pub struct DynamoDbDependentRepository {
    client: Client,
    table_name: String,
}

impl DynamoDbDependentRepository {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    /// Convert Dependent to DynamoDB item
    fn to_item(&self, dependent: &Dependent) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("FAMILY#{}", dependent.family_id.0)),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("DEPENDENT#{}", dependent.id.0)),
        );
        item.insert(
            "Type".to_string(),
            AttributeValue::S("Dependent".to_string()),
        );
        item.insert(
            "id".to_string(),
            AttributeValue::S(dependent.id.0.to_string()),
        );
        item.insert(
            "family_id".to_string(),
            AttributeValue::S(dependent.family_id.0.to_string()),
        );
        item.insert(
            "name".to_string(),
            AttributeValue::S(dependent.name.clone()),
        );
        item.insert(
            "date_of_birth".to_string(),
            AttributeValue::S(dependent.date_of_birth.0.to_string()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(dependent.created_at.0.to_rfc3339()),
        );
        item.insert(
            "updated_at".to_string(),
            AttributeValue::S(dependent.updated_at.0.to_rfc3339()),
        );
        if let Some(ref share_id) = dependent.share_id {
            item.insert(
                "share_id".to_string(),
                AttributeValue::S(share_id.0.to_string()),
            );
        }
        if let Some(ref permission_scope) = dependent.permission_scope {
            let scope_json =
                serde_json::to_string(permission_scope).unwrap_or_else(|_| "{}".to_string());
            item.insert(
                "permission_scope".to_string(),
                AttributeValue::S(scope_json),
            );
        }
        item
    }

    /// Convert DynamoDB item to Dependent
    fn parse_item(&self, item: &HashMap<String, AttributeValue>) -> Result<Dependent, StoreError> {
        let id = item
            .get("id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid id".to_string()))?;

        let family_id = item
            .get("family_id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid family_id".to_string()))?;

        let name = item
            .get("name")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| StoreError::QueryError("Missing name".to_string()))?
            .clone();

        let date_of_birth = item
            .get("date_of_birth")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| {
                StoreError::QueryError("Missing or invalid date_of_birth".to_string())
            })?;

        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid created_at".to_string()))?;

        let updated_at = item
            .get("updated_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid updated_at".to_string()))?;

        let share_id = item
            .get("share_id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .map(ShareId);

        let permission_scope = item
            .get("permission_scope")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| serde_json::from_str::<PermissionScope>(s).ok());

        Ok(Dependent {
            id: DependentId(id),
            family_id: FamilyId(family_id),
            name,
            date_of_birth: Date::from_naive_date(date_of_birth),
            created_at: Timestamp::from_datetime(created_at),
            updated_at: Timestamp::from_datetime(updated_at),
            share_id,
            permission_scope,
        })
    }
}

#[async_trait]
impl DependentRepository for DynamoDbDependentRepository {
    async fn create(&self, dependent: Dependent) -> Result<Dependent, StoreError> {
        let item = self.to_item(&dependent);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to create dependent: {}", e)))?;

        Ok(dependent)
    }

    async fn get(
        &self,
        family_id: FamilyId,
        id: DependentId,
    ) -> Result<Option<Dependent>, StoreError> {
        let pk = format!("FAMILY#{}", family_id.0);
        let sk = format!("DEPENDENT#{}", id.0);

        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(pk))
            .key("SK", AttributeValue::S(sk))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to get dependent: {}", e)))?;

        match result.item {
            Some(item) => Ok(Some(self.parse_item(&item)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, dependent: Dependent) -> Result<Dependent, StoreError> {
        let item = self.to_item(&dependent);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to update dependent: {}", e)))?;

        Ok(dependent)
    }

    async fn list_by_family(
        &self,
        family_id: FamilyId,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Dependent>, StoreError> {
        let exclusive_start_key = decode_next_token(&pagination.next_token)?;

        let mut query = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("FAMILY#{}", family_id.0)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("DEPENDENT#".to_string()))
            .limit(pagination.effective_limit() as i32);

        if let Some(start_key) = exclusive_start_key {
            query = query.set_exclusive_start_key(Some(start_key));
        }

        let result = query.send().await.map_err(|e| {
            StoreError::QueryError(format!("Failed to list dependents by family: {}", e))
        })?;

        let dependents = result
            .items
            .unwrap_or_default()
            .iter()
            .map(|item| self.parse_item(item))
            .collect::<Result<Vec<_>, _>>()?;

        let next_token = encode_last_evaluated_key(&result.last_evaluated_key)?;

        Ok(PaginatedResponse::with_next_token(dependents, next_token))
    }

    async fn delete(&self, family_id: FamilyId, id: DependentId) -> Result<(), StoreError> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("FAMILY#{}", family_id.0)))
            .key("SK", AttributeValue::S(format!("DEPENDENT#{}", id.0)))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to delete dependent: {}", e)))?;

        Ok(())
    }
}

/// DynamoDB implementation of ActivityRepository
#[derive(Clone)]
pub struct DynamoDbActivityRepository {
    client: Client,
    table_name: String,
}

impl DynamoDbActivityRepository {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    /// Convert Activity to DynamoDB item
    fn to_item(&self, activity: &Activity) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!(
                "FAMILY#{}#DEPENDENT#{}",
                activity.family_id.0, activity.dependent_id.0
            )),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("ACTIVITY#{}", activity.id.0)),
        );
        item.insert(
            "Type".to_string(),
            AttributeValue::S("Activity".to_string()),
        );
        item.insert(
            "id".to_string(),
            AttributeValue::S(activity.id.0.to_string()),
        );
        item.insert(
            "dependent_id".to_string(),
            AttributeValue::S(activity.dependent_id.0.to_string()),
        );
        item.insert(
            "family_id".to_string(),
            AttributeValue::S(activity.family_id.0.to_string()),
        );
        item.insert(
            "timestamp".to_string(),
            AttributeValue::S(activity.timestamp.0.to_rfc3339()),
        );
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(activity.created_at.0.to_rfc3339()),
        );
        item.insert(
            "updated_at".to_string(),
            AttributeValue::S(activity.updated_at.0.to_rfc3339()),
        );

        // Serialize activity_type as JSON
        let activity_type_json =
            serde_json::to_string(&activity.activity_type).unwrap_or_else(|_| "{}".to_string());
        item.insert(
            "activity_type".to_string(),
            AttributeValue::S(activity_type_json),
        );

        // Add activity type discriminator for filtering
        let type_name = match &activity.activity_type {
            ActivityType::Feeding { .. } => "feeding",
            ActivityType::DiaperChange { .. } => "diaper_change",
            ActivityType::Sleep { .. } => "sleep",
            ActivityType::Pumping { .. } => "pumping",
            ActivityType::ActivityTime { .. } => "activity_time",
            ActivityType::TummyTime { .. } => "tummy_time",
            ActivityType::WakeWindow { .. } => "wake_window",
        };
        item.insert(
            "activity_type_name".to_string(),
            AttributeValue::S(type_name.to_string()),
        );

        if let Some(ref share_id) = activity.share_id {
            item.insert(
                "share_id".to_string(),
                AttributeValue::S(share_id.0.to_string()),
            );
        }
        if let Some(ref permission_scope) = activity.permission_scope {
            let scope_json =
                serde_json::to_string(permission_scope).unwrap_or_else(|_| "{}".to_string());
            item.insert(
                "permission_scope".to_string(),
                AttributeValue::S(scope_json),
            );
        }

        item
    }

    /// Convert DynamoDB item to Activity
    fn parse_item(&self, item: &HashMap<String, AttributeValue>) -> Result<Activity, StoreError> {
        let id = item
            .get("id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid id".to_string()))?;

        let dependent_id = item
            .get("dependent_id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid dependent_id".to_string()))?;

        let family_id = item
            .get("family_id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid family_id".to_string()))?;

        let timestamp = item
            .get("timestamp")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid timestamp".to_string()))?;

        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid created_at".to_string()))?;

        let updated_at = item
            .get("updated_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid updated_at".to_string()))?;

        let activity_type_json = item
            .get("activity_type")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| StoreError::QueryError("Missing activity_type".to_string()))?;

        let activity_type: ActivityType = serde_json::from_str(activity_type_json)
            .map_err(|e| StoreError::QueryError(format!("Invalid activity_type JSON: {}", e)))?;

        let share_id = item
            .get("share_id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .map(ShareId);

        let permission_scope = item
            .get("permission_scope")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| serde_json::from_str::<PermissionScope>(s).ok());

        Ok(Activity {
            id: ActivityId(id),
            family_id: FamilyId(family_id),
            dependent_id: DependentId(dependent_id),
            timestamp: Timestamp::from_datetime(timestamp),
            activity_type,
            created_at: Timestamp::from_datetime(created_at),
            updated_at: Timestamp::from_datetime(updated_at),
            share_id,
            permission_scope,
        })
    }
}

#[async_trait]
impl ActivityRepository for DynamoDbActivityRepository {
    async fn create(&self, activity: Activity) -> Result<Activity, StoreError> {
        let item = self.to_item(&activity);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to create activity: {}", e)))?;

        Ok(activity)
    }

    async fn get(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        id: ActivityId,
    ) -> Result<Option<Activity>, StoreError> {
        let pk = format!("FAMILY#{}#DEPENDENT#{}", family_id.0, dependent_id.0);
        let sk = format!("ACTIVITY#{}", id.0);

        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(pk))
            .key("SK", AttributeValue::S(sk))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to get activity: {}", e)))?;

        match result.item {
            Some(item) => Ok(Some(self.parse_item(&item)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, activity: Activity) -> Result<Activity, StoreError> {
        let item = self.to_item(&activity);

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to update activity: {}", e)))?;

        Ok(activity)
    }

    async fn delete(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        id: ActivityId,
    ) -> Result<(), StoreError> {
        let pk = format!("FAMILY#{}#DEPENDENT#{}", family_id.0, dependent_id.0);
        let sk = format!("ACTIVITY#{}", id.0);

        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(pk))
            .key("SK", AttributeValue::S(sk))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to delete activity: {}", e)))?;

        Ok(())
    }

    async fn query(
        &self,
        params: ActivityQueryParams,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Activity>, StoreError> {
        let pk = format!(
            "FAMILY#{}#DEPENDENT#{}",
            params.family_id.0, params.dependent_id.0
        );

        // Build key condition for GSI: PK + optional timestamp range
        let mut key_condition = "PK = :pk".to_string();
        let mut filter_expressions = Vec::new();

        if params.start_date.is_some() && params.end_date.is_some() {
            key_condition.push_str(" AND #timestamp BETWEEN :start_date AND :end_date");
        } else if params.start_date.is_some() {
            key_condition.push_str(" AND #timestamp >= :start_date");
        } else if params.end_date.is_some() {
            key_condition.push_str(" AND #timestamp <= :end_date");
        }

        // Activity type filter stays as a filter expression
        if params.activity_type.is_some() {
            filter_expressions.push("activity_type_name = :activity_type");
        }

        let exclusive_start_key = decode_next_token(&pagination.next_token)?;

        let mut query_builder = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI-1")
            .key_condition_expression(&key_condition)
            .expression_attribute_values(":pk", AttributeValue::S(pk))
            .scan_index_forward(false) // Sort descending by timestamp
            .limit(pagination.effective_limit() as i32);

        if let Some(start_key) = exclusive_start_key {
            query_builder = query_builder.set_exclusive_start_key(Some(start_key));
        }

        // Add timestamp attribute name alias (reserved word)
        if params.start_date.is_some() || params.end_date.is_some() {
            query_builder = query_builder.expression_attribute_names("#timestamp", "timestamp");

            if let Some(start_ts) = params.start_date {
                query_builder = query_builder.expression_attribute_values(
                    ":start_date",
                    AttributeValue::S(start_ts.to_iso8601()),
                );
            }

            if let Some(end_ts) = params.end_date {
                query_builder = query_builder.expression_attribute_values(
                    ":end_date",
                    AttributeValue::S(end_ts.to_iso8601()),
                );
            }
        }

        if !filter_expressions.is_empty() {
            query_builder = query_builder.filter_expression(filter_expressions.join(" AND "));

            if let Some(activity_type) = &params.activity_type {
                let type_name = match activity_type {
                    ActivityType::Feeding { .. } => "feeding",
                    ActivityType::DiaperChange { .. } => "diaper_change",
                    ActivityType::Sleep { .. } => "sleep",
                    ActivityType::Pumping { .. } => "pumping",
                    ActivityType::ActivityTime { .. } => "activity_time",
                    ActivityType::TummyTime { .. } => "tummy_time",
                    ActivityType::WakeWindow { .. } => "wake_window",
                };
                query_builder = query_builder.expression_attribute_values(
                    ":activity_type",
                    AttributeValue::S(type_name.to_string()),
                );
            }
        }

        let result = query_builder
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to query activities: {}", e)))?;

        let activities = result
            .items
            .unwrap_or_default()
            .iter()
            .map(|item| self.parse_item(item))
            .collect::<Result<Vec<_>, _>>()?;

        let next_token = encode_last_evaluated_key(&result.last_evaluated_key)?;

        Ok(PaginatedResponse::with_next_token(activities, next_token))
    }
}

/// DynamoDB implementation of ShareRepository
///
/// Uses a dual-write pattern: each Share is stored in both the owner partition
/// (`OWNER#{requester_id}/SHARE#{share_id}`) and the email partition
/// (`SHARE_EMAIL#{accepter_email}/SHARE#{share_id}`). All create/update/delete
/// operations use TransactWriteItems for consistency.
#[derive(Clone)]
pub struct DynamoDbShareRepository {
    client: Client,
    table_name: String,
}

impl DynamoDbShareRepository {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    /// Build the common set of share attributes (shared between owner and email items)
    fn share_attributes(&self, share: &Share) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S(share.id.0.to_string()));
        item.insert(
            "family_id".to_string(),
            AttributeValue::S(share.family_id.0.to_string()),
        );
        item.insert(
            "requester_id".to_string(),
            AttributeValue::S(share.requester_id.0.clone()),
        );
        item.insert(
            "accepter_username".to_string(),
            AttributeValue::S(share.accepter_username.clone()),
        );
        if let Some(ref accepter_id) = share.accepter_id {
            item.insert(
                "accepter_id".to_string(),
                AttributeValue::S(accepter_id.0.clone()),
            );
        }
        let scope_json =
            serde_json::to_string(&share.permission_scope).unwrap_or_else(|_| "{}".to_string());
        item.insert(
            "permission_scope".to_string(),
            AttributeValue::S(scope_json),
        );
        let status_str = serde_json::to_string(&share.status)
            .unwrap_or_else(|_| "\"pending\"".to_string())
            .trim_matches('"')
            .to_string();
        item.insert("status".to_string(), AttributeValue::S(status_str));
        item.insert(
            "created_at".to_string(),
            AttributeValue::S(share.created_at.0.to_rfc3339()),
        );
        item.insert(
            "updated_at".to_string(),
            AttributeValue::S(share.updated_at.0.to_rfc3339()),
        );
        if let Some(ref expires_at) = share.expires_at {
            item.insert(
                "expires_at".to_string(),
                AttributeValue::S(expires_at.0.to_rfc3339()),
            );
            // Set expires_in as Unix epoch seconds for DynamoDB TTL auto-cleanup
            item.insert(
                "expires_in".to_string(),
                AttributeValue::N(expires_at.0.timestamp().to_string()),
            );
        }
        item
    }

    /// Build the owner partition item for a Share
    fn to_owner_item(&self, share: &Share) -> HashMap<String, AttributeValue> {
        let mut item = self.share_attributes(share);
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("OWNER#{}", share.requester_id.0)),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("SHARE#{}", share.id.0)),
        );
        item.insert("Type".to_string(), AttributeValue::S("Share".to_string()));
        item
    }

    /// Build the email partition item for a Share
    fn to_email_item(&self, share: &Share) -> HashMap<String, AttributeValue> {
        let mut item = self.share_attributes(share);
        item.insert(
            "PK".to_string(),
            AttributeValue::S(format!("SHARE_USERNAME#{}", share.accepter_username)),
        );
        item.insert(
            "SK".to_string(),
            AttributeValue::S(format!("SHARE#{}", share.id.0)),
        );
        item.insert(
            "Type".to_string(),
            AttributeValue::S("ShareEmailIndex".to_string()),
        );
        item
    }

    /// Parse a DynamoDB item into a Share
    fn parse_share_item(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<Share, StoreError> {
        let id = item
            .get("id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| StoreError::QueryError("Missing or invalid share id".to_string()))?;

        let family_id = item
            .get("family_id")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| {
                StoreError::QueryError("Missing or invalid share family_id".to_string())
            })?;

        let requester_id = item
            .get("requester_id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| StoreError::QueryError("Missing share requester_id".to_string()))?
            .clone();

        let accepter_username = item
            .get("accepter_username")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| StoreError::QueryError("Missing share accepter_email".to_string()))?
            .clone();

        let accepter_id = item
            .get("accepter_id")
            .and_then(|v| v.as_s().ok())
            .map(|s| IdentityId(s.clone()));

        let permission_scope_json = item
            .get("permission_scope")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| StoreError::QueryError("Missing share permission_scope".to_string()))?;
        let permission_scope: PermissionScope = serde_json::from_str(permission_scope_json)
            .map_err(|e| StoreError::QueryError(format!("Invalid permission_scope JSON: {}", e)))?;

        let status_str = item
            .get("status")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| StoreError::QueryError("Missing share status".to_string()))?;
        let status: ShareStatus =
            serde_json::from_str(&format!("\"{}\"", status_str)).map_err(|e| {
                StoreError::QueryError(format!("Invalid share status '{}': {}", status_str, e))
            })?;

        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| {
                StoreError::QueryError("Missing or invalid share created_at".to_string())
            })?;

        let updated_at = item
            .get("updated_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| {
                StoreError::QueryError("Missing or invalid share updated_at".to_string())
            })?;

        let expires_at = item
            .get("expires_in")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse().ok())
            .map(Timestamp::from_datetime);

        Ok(Share {
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
}

#[async_trait]
impl ShareRepository for DynamoDbShareRepository {
    async fn create(&self, share: Share) -> Result<Share, StoreError> {
        use aws_sdk_dynamodb::types::{Put, TransactWriteItem};

        let owner_item = self.to_owner_item(&share);
        let email_item = self.to_email_item(&share);

        let owner_put = Put::builder()
            .table_name(&self.table_name)
            .set_item(Some(owner_item))
            .condition_expression("attribute_not_exists(PK) AND attribute_not_exists(SK)")
            .build()
            .map_err(|e| StoreError::QueryError(format!("Failed to build owner put: {}", e)))?;

        let email_put = Put::builder()
            .table_name(&self.table_name)
            .set_item(Some(email_item))
            .condition_expression("attribute_not_exists(PK) AND attribute_not_exists(SK)")
            .build()
            .map_err(|e| StoreError::QueryError(format!("Failed to build email put: {}", e)))?;

        self.client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(owner_put).build())
            .transact_items(TransactWriteItem::builder().put(email_put).build())
            .send()
            .await
            .map_err(|e| {
                let err_str = format!("{}", e);
                if err_str.contains("ConditionalCheckFailed")
                    || err_str.contains("TransactionCanceledException")
                {
                    StoreError::ConflictError(
                        "Share already exists for this family and email".to_string(),
                    )
                } else {
                    StoreError::QueryError(format!("Failed to create share: {}", e))
                }
            })?;

        Ok(share)
    }

    async fn get(
        &self,
        requester_id: IdentityId,
        share_id: ShareId,
    ) -> Result<Option<Share>, StoreError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("OWNER#{}", requester_id.0)))
            .key("SK", AttributeValue::S(format!("SHARE#{}", share_id.0)))
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to get share: {}", e)))?;

        match result.item {
            Some(item) => Ok(Some(self.parse_share_item(&item)?)),
            None => Ok(None),
        }
    }

    async fn get_by_username_and_share_id(
        &self,
        accepter_username: &str,
        share_id: ShareId,
    ) -> Result<Option<Share>, StoreError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key(
                "PK",
                AttributeValue::S(format!("SHARE_USERNAME#{}", accepter_username)),
            )
            .key("SK", AttributeValue::S(format!("SHARE#{}", share_id.0)))
            .send()
            .await
            .map_err(|e| {
                StoreError::QueryError(format!("Failed to get share by email and id: {}", e))
            })?;

        match result.item {
            Some(item) => Ok(Some(self.parse_share_item(&item)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, share: Share) -> Result<Share, StoreError> {
        use aws_sdk_dynamodb::types::{Put, TransactWriteItem};

        let owner_item = self.to_owner_item(&share);
        let email_item = self.to_email_item(&share);

        let owner_put = Put::builder()
            .table_name(&self.table_name)
            .set_item(Some(owner_item))
            .build()
            .map_err(|e| StoreError::QueryError(format!("Failed to build owner put: {}", e)))?;

        let email_put = Put::builder()
            .table_name(&self.table_name)
            .set_item(Some(email_item))
            .build()
            .map_err(|e| StoreError::QueryError(format!("Failed to build email put: {}", e)))?;

        self.client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(owner_put).build())
            .transact_items(TransactWriteItem::builder().put(email_put).build())
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to update share: {}", e)))?;

        Ok(share)
    }

    async fn delete(&self, requester_id: IdentityId, share_id: ShareId) -> Result<(), StoreError> {
        use aws_sdk_dynamodb::types::{Delete, TransactWriteItem};

        // First retrieve the share to get the accepter_email for the email partition key
        let share = self
            .get(requester_id.clone(), share_id)
            .await?
            .ok_or_else(|| StoreError::NotFound("Share not found".to_string()))?;

        let owner_delete = Delete::builder()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("OWNER#{}", requester_id.0)))
            .key("SK", AttributeValue::S(format!("SHARE#{}", share_id.0)))
            .build()
            .map_err(|e| StoreError::QueryError(format!("Failed to build owner delete: {}", e)))?;

        let email_delete = Delete::builder()
            .table_name(&self.table_name)
            .key(
                "PK",
                AttributeValue::S(format!("SHARE_USERNAME#{}", share.accepter_username)),
            )
            .key("SK", AttributeValue::S(format!("SHARE#{}", share_id.0)))
            .build()
            .map_err(|e| StoreError::QueryError(format!("Failed to build email delete: {}", e)))?;

        self.client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().delete(owner_delete).build())
            .transact_items(TransactWriteItem::builder().delete(email_delete).build())
            .send()
            .await
            .map_err(|e| StoreError::QueryError(format!("Failed to delete share: {}", e)))?;

        Ok(())
    }

    async fn list_by_family(
        &self,
        requester_id: IdentityId,
        family_id: FamilyId,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Share>, StoreError> {
        let exclusive_start_key = decode_next_token(&pagination.next_token)?;

        let mut query_builder = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .filter_expression("family_id = :fid")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("OWNER#{}", requester_id.0)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("SHARE#".to_string()))
            .expression_attribute_values(":fid", AttributeValue::S(family_id.0.to_string()))
            .limit(pagination.effective_limit() as i32);

        query_builder = query_builder.set_exclusive_start_key(exclusive_start_key);

        let result = query_builder.send().await.map_err(|e| {
            StoreError::QueryError(format!("Failed to list shares by family: {}", e))
        })?;

        let shares = result
            .items
            .unwrap_or_default()
            .iter()
            .map(|item| self.parse_share_item(item))
            .collect::<Result<Vec<_>, _>>()?;

        let next_token = encode_last_evaluated_key(&result.last_evaluated_key)?;

        Ok(PaginatedResponse::with_next_token(shares, next_token))
    }

    async fn list_by_accepter_username(
        &self,
        username: &str,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Share>, StoreError> {
        let exclusive_start_key = decode_next_token(&pagination.next_token)?;

        let mut query_builder = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("SHARE_USERNAME#{}", username)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("SHARE#".to_string()))
            .limit(pagination.effective_limit() as i32);

        query_builder = query_builder.set_exclusive_start_key(exclusive_start_key);

        let result = query_builder.send().await.map_err(|e| {
            StoreError::QueryError(format!("Failed to list shares by accepter email: {}", e))
        })?;

        let shares = result
            .items
            .unwrap_or_default()
            .iter()
            .map(|item| self.parse_share_item(item))
            .collect::<Result<Vec<_>, _>>()?;

        let next_token = encode_last_evaluated_key(&result.last_evaluated_key)?;

        Ok(PaginatedResponse::with_next_token(shares, next_token))
    }

    async fn get_by_family_and_username(
        &self,
        family_id: FamilyId,
        accepter_username: &str,
    ) -> Result<Option<Share>, StoreError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .filter_expression("family_id = :fid")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("SHARE_USERNAME#{}", accepter_username)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("SHARE#".to_string()))
            .expression_attribute_values(":fid", AttributeValue::S(family_id.0.to_string()))
            .send()
            .await
            .map_err(|e| {
                StoreError::QueryError(format!("Failed to get share by family and email: {}", e))
            })?;

        let items = result.items.unwrap_or_default();
        match items.first() {
            Some(item) => Ok(Some(self.parse_share_item(item)?)),
            None => Ok(None),
        }
    }
}

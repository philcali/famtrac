use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use std::collections::HashMap;

use crate::domain::{
    Activity, ActivityId, ActivityType, Date, Dependent, DependentId, Family, FamilyId, IdentityId,
    PermissionScope, ShareId, Timestamp,
};
use crate::errors::StoreError;

use super::traits::{
    ActivityQueryParams, ActivityRepository, DependentRepository, FamilyRepository,
};

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

    async fn get_by_owner(&self, owner_id: IdentityId) -> Result<Vec<Family>, StoreError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(":pk", AttributeValue::S(format!("OWNER#{}", owner_id.0)))
            .expression_attribute_values(":sk_prefix", AttributeValue::S("FAMILY#".to_string()))
            .send()
            .await
            .map_err(|e| {
                StoreError::QueryError(format!("Failed to query families by owner: {}", e))
            })?;

        let families = result
            .items
            .unwrap_or_default()
            .iter()
            .map(|item| self.parse_item(item))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(families)
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

    async fn list_by_family(&self, family_id: FamilyId) -> Result<Vec<Dependent>, StoreError> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("PK = :pk AND begins_with(SK, :sk_prefix)")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("FAMILY#{}", family_id.0)),
            )
            .expression_attribute_values(":sk_prefix", AttributeValue::S("DEPENDENT#".to_string()))
            .send()
            .await
            .map_err(|e| {
                StoreError::QueryError(format!("Failed to list dependents by family: {}", e))
            })?;

        let dependents = result
            .items
            .unwrap_or_default()
            .iter()
            .map(|item| self.parse_item(item))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(dependents)
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

    async fn query(&self, params: ActivityQueryParams) -> Result<Vec<Activity>, StoreError> {
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

        let mut query_builder = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("GSI-1")
            .key_condition_expression(&key_condition)
            .expression_attribute_values(":pk", AttributeValue::S(pk))
            .scan_index_forward(false); // Sort descending by timestamp

        // Add timestamp attribute name alias (reserved word)
        if params.start_date.is_some() || params.end_date.is_some() {
            query_builder = query_builder.expression_attribute_names("#timestamp", "timestamp");

            if let Some(start_date) = params.start_date {
                let start_datetime = start_date
                    .0
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .to_rfc3339();
                query_builder = query_builder
                    .expression_attribute_values(":start_date", AttributeValue::S(start_datetime));
            }

            if let Some(end_date) = params.end_date {
                let end_datetime = end_date
                    .0
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_utc()
                    .to_rfc3339();
                query_builder = query_builder
                    .expression_attribute_values(":end_date", AttributeValue::S(end_datetime));
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

        Ok(activities)
    }
}

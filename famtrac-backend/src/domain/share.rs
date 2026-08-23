use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{FamilyId, IdentityId, Timestamp};
use crate::errors::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShareId(pub Uuid);

impl ShareId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ShareId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareStatus {
    Pending,
    Active,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionAction {
    FamilyRead,
    DependentRead,
    DependentWrite,
    ActivityRead,
    ActivityWrite,
    RecipeRead,
    RecipeWrite,
    MealSlotRead,
    MealSlotWrite,
    FeedingLogRead,
    FeedingLogWrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionScope {
    pub actions: Vec<PermissionAction>,
}

impl PermissionScope {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.actions.is_empty() {
            return Err(ValidationError {
                field: "permission_scope".to_string(),
                message: "Permission scope must contain at least one action".to_string(),
                constraint: Some("must contain at least one action".to_string()),
            });
        }

        if !self.actions.contains(&PermissionAction::FamilyRead) {
            return Err(ValidationError {
                field: "permission_scope".to_string(),
                message: "Permission scope must include family_read".to_string(),
                constraint: Some("must include family_read".to_string()),
            });
        }

        if self.actions.contains(&PermissionAction::DependentWrite)
            && !self.actions.contains(&PermissionAction::DependentRead)
        {
            return Err(ValidationError {
                field: "permission_scope".to_string(),
                message: "dependent_write requires dependent_read".to_string(),
                constraint: Some("dependent_write requires dependent_read".to_string()),
            });
        }

        if self.actions.contains(&PermissionAction::ActivityWrite) {
            if !self.actions.contains(&PermissionAction::ActivityRead) {
                return Err(ValidationError {
                    field: "permission_scope".to_string(),
                    message: "activity_write requires activity_read".to_string(),
                    constraint: Some("activity_write requires activity_read".to_string()),
                });
            }
            if !self.actions.contains(&PermissionAction::DependentRead) {
                return Err(ValidationError {
                    field: "permission_scope".to_string(),
                    message: "activity_write requires dependent_read".to_string(),
                    constraint: Some("activity_write requires dependent_read".to_string()),
                });
            }
        }

        if self.actions.contains(&PermissionAction::RecipeWrite)
            && !self.actions.contains(&PermissionAction::RecipeRead)
        {
            return Err(ValidationError {
                field: "permission_scope".to_string(),
                message: "recipe_write requires recipe_read".to_string(),
                constraint: Some("recipe_write requires recipe_read".to_string()),
            });
        }

        if self.actions.contains(&PermissionAction::MealSlotWrite)
            && !self.actions.contains(&PermissionAction::MealSlotRead)
        {
            return Err(ValidationError {
                field: "permission_scope".to_string(),
                message: "meal_slot_write requires meal_slot_read".to_string(),
                constraint: Some("meal_slot_write requires meal_slot_read".to_string()),
            });
        }

        if self.actions.contains(&PermissionAction::FeedingLogWrite)
            && !self.actions.contains(&PermissionAction::FeedingLogRead)
        {
            return Err(ValidationError {
                field: "permission_scope".to_string(),
                message: "feeding_log_write requires feeding_log_read".to_string(),
                constraint: Some("feeding_log_write requires feeding_log_read".to_string()),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Share {
    pub id: ShareId,
    pub family_id: FamilyId,
    pub requester_id: IdentityId,
    pub accepter_username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepter_id: Option<IdentityId>,
    pub permission_scope: PermissionScope,
    pub status: ShareStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
}

use serde::{Deserialize, Serialize};

use super::{FamilyId, MealSlotId, RecipeId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealSlot {
    pub id: MealSlotId,
    pub family_id: FamilyId,
    pub dependent_id: super::DependentId,
    pub day: String,
    pub time: String,
    pub recipe_id: Option<RecipeId>,
    pub notes: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_id: Option<super::ShareId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_scope: Option<super::PermissionScope>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateMealSlotRequest {
    pub dependent_id: super::DependentId,
    pub day: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateMealSlotRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl MealSlot {
    pub fn new(id: MealSlotId, family_id: FamilyId) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            family_id,
            dependent_id: super::DependentId::new(),
            day: String::new(),
            time: String::new(),
            recipe_id: None,
            notes: None,
            created_at: now,
            updated_at: now,
            share_id: None,
            permission_scope: None,
        }
    }

    pub fn apply_update(&mut self, update: UpdateMealSlotRequest) {
        if let Some(day) = update.day {
            self.day = day;
        }
        if let Some(time) = update.time {
            self.time = time;
        }
        if let Some(recipe_id_str) = update.recipe_id {
            self.recipe_id = recipe_id_str.parse().ok().map(RecipeId);
        }
        if let Some(notes) = update.notes {
            self.notes = Some(notes);
        }
        self.updated_at = Timestamp::now();
    }
}

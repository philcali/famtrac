use serde::{Deserialize, Serialize};

use super::{FamilyId, FeedingLogId, RecipeId, Timestamp};

/// FeedingLog records a single feeding event (e.g. a baby ate a certain amount of a recipe).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedingLog {
    pub id: FeedingLogId,
    pub family_id: FamilyId,
    pub dependent_id: super::DependentId,
    pub date: String,
    pub time: String,
    pub recipe_id: Option<RecipeId>,
    pub amount: f64,
    pub reaction: Option<String>,
    pub notes: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_id: Option<super::ShareId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_scope: Option<super::PermissionScope>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeedingLogRequest {
    pub family_id: FamilyId,
    pub dependent_id: super::DependentId,
    pub date: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reaction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeedingLogRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reaction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl FeedingLog {
    pub fn new(id: FeedingLogId, family_id: FamilyId) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            family_id,
            dependent_id: super::DependentId::new(),
            date: String::new(),
            time: String::new(),
            recipe_id: None,
            amount: 0.0,
            reaction: None,
            notes: None,
            created_at: now,
            updated_at: now,
            share_id: None,
            permission_scope: None,
        }
    }

    pub fn apply_update(&mut self, update: UpdateFeedingLogRequest) {
        if let Some(date) = update.date {
            self.date = date;
        }
        if let Some(time) = update.time {
            self.time = time;
        }
        if let Some(recipe_id_str) = update.recipe_id {
            self.recipe_id = recipe_id_str.parse().ok().map(RecipeId);
        }
        if let Some(amount) = update.amount {
            self.amount = amount;
        }
        if let Some(reaction) = update.reaction {
            self.reaction = Some(reaction);
        }
        if let Some(notes) = update.notes {
            self.notes = Some(notes);
        }
        self.updated_at = Timestamp::now();
    }
}

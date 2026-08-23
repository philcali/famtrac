use serde::{Deserialize, Serialize};

use super::{FamilyId, PermissionScope, RecipeId, ShareId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub family_id: FamilyId,
    pub name: String,
    pub emoji: Option<String>,
    pub ingredients: Vec<String>,
    pub age_min: Option<u32>,
    pub texture: Option<String>,
    pub allergens: Vec<String>,
    pub prep_notes: Option<String>,
    pub safe: Option<bool>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_id: Option<ShareId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_scope: Option<PermissionScope>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateRecipeRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(skip_serializing_if = "default_vec")]
    pub ingredients: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
    #[serde(skip_serializing_if = "default_vec")]
    pub allergens: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prep_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe: Option<bool>,
}

#[allow(dead_code)]
fn default_vec(v: &Option<Vec<String>>) -> bool {
    v.as_ref().is_none_or(|vec| vec.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRecipeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingredients: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allergens: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prep_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe: Option<bool>,
}

impl Recipe {
    pub fn new(id: RecipeId, family_id: FamilyId) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            family_id,
            name: String::new(),
            emoji: None,
            ingredients: Vec::new(),
            age_min: None,
            texture: None,
            allergens: Vec::new(),
            prep_notes: None,
            safe: None,
            created_at: now,
            updated_at: now,
            share_id: None,
            permission_scope: None,
        }
    }

    pub fn apply_update(&mut self, update: UpdateRecipeRequest) {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(emoji) = update.emoji {
            self.emoji = Some(emoji);
        }
        if let Some(ingredients) = update.ingredients {
            self.ingredients = ingredients;
        }
        if let Some(age_min) = update.age_min {
            self.age_min = Some(age_min);
        }
        if let Some(texture) = update.texture {
            self.texture = Some(texture);
        }
        if let Some(allergens) = update.allergens {
            self.allergens = allergens;
        }
        if let Some(prep_notes) = update.prep_notes {
            self.prep_notes = Some(prep_notes);
        }
        if let Some(safe) = update.safe {
            self.safe = Some(safe);
        }
        self.updated_at = Timestamp::now();
    }
}

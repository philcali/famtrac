use super::permission::check_permission;
use crate::context::RequestContext;
use crate::domain::{FamilyId, PermissionAction, Recipe, RecipeId, Timestamp};
use crate::errors::HandlerError;
use crate::handlers::pagination::PaginationParams;
use crate::repository::{FamilyRepository, RecipeRepository};
use serde::{Deserialize, Serialize};

/// Request body for creating a new recipe
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateRecipeRequest {
    pub family_id: FamilyId,
    pub name: String,
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

/// Request body for updating a recipe
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

/// Response body for recipe operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeResponse {
    pub id: RecipeId,
    pub family_id: FamilyId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ingredients: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allergens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prep_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe: Option<bool>,
    pub created_at: String,
    pub updated_at: String,
}

/// Response body for listing recipes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeListResponse {
    pub recipes: Vec<RecipeResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

impl From<Recipe> for RecipeResponse {
    fn from(recipe: Recipe) -> Self {
        RecipeResponse {
            id: recipe.id,
            family_id: recipe.family_id,
            name: recipe.name,
            emoji: recipe.emoji,
            ingredients: recipe.ingredients,
            age_min: recipe.age_min,
            texture: recipe.texture,
            allergens: recipe.allergens,
            prep_notes: recipe.prep_notes,
            safe: recipe.safe,
            created_at: recipe.created_at.to_iso8601(),
            updated_at: recipe.updated_at.to_iso8601(),
        }
    }
}

/// Handler for POST /families/{family_id}/recipes
pub async fn create_recipe<F: FamilyRepository, R: RecipeRepository>(
    family_id: FamilyId,
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    recipe_repository: &R,
) -> Result<(u16, String), HandlerError> {
    let request: CreateRecipeRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate recipe name
    if request.name.trim().is_empty() {
        return Err(HandlerError::Validation(crate::errors::ValidationError {
            field: "name".to_string(),
            message: "Recipe name cannot be empty".to_string(),
            constraint: Some("must be a non-empty string".to_string()),
        }));
    }

    // Verify Family ownership
    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Enforce permission
    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    // Create recipe
    let now = Timestamp::now();
    let recipe = Recipe {
        id: RecipeId::new(),
        family_id: request.family_id,
        name: request.name,
        emoji: request.emoji,
        ingredients: request.ingredients.unwrap_or_default(),
        age_min: request.age_min,
        texture: request.texture,
        allergens: request.allergens.unwrap_or_default(),
        prep_notes: request.prep_notes,
        safe: request.safe,
        created_at: now,
        updated_at: now,
        share_id: None,
        permission_scope: None,
    };

    let created_recipe = recipe_repository.create(recipe).await?;
    let response = RecipeResponse::from(created_recipe);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((201, response_json))
}

/// Handler for GET /families/{family_id}/recipes/{recipe_id}
pub async fn get_recipe<F: FamilyRepository, R: RecipeRepository>(
    family_id: FamilyId,
    recipe_id: RecipeId,
    context: &RequestContext,
    family_repository: &F,
    recipe_repository: &R,
) -> Result<(u16, String), HandlerError> {
    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentRead,
    )?;

    let recipe = recipe_repository.get(family_id, recipe_id).await?;
    let recipe = recipe.ok_or(HandlerError::NotFound(format!(
        "Recipe with id {:?} not found",
        recipe_id
    )))?;

    let response = RecipeResponse::from(recipe);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for PUT /families/{family_id}/recipes/{recipe_id}
pub async fn update_recipe<F: FamilyRepository, R: RecipeRepository>(
    family_id: FamilyId,
    recipe_id: RecipeId,
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    recipe_repository: &R,
) -> Result<(u16, String), HandlerError> {
    let request: UpdateRecipeRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    let recipe = recipe_repository.get(family_id, recipe_id).await?;
    let mut recipe = recipe.ok_or(HandlerError::NotFound(format!(
        "Recipe with id {:?} not found",
        recipe_id
    )))?;

    if let Some(name) = &request.name {
        if name.trim().is_empty() {
            return Err(HandlerError::Validation(crate::errors::ValidationError {
                field: "name".to_string(),
                message: "Recipe name cannot be empty".to_string(),
                constraint: Some("must be a non-empty string".to_string()),
            }));
        }
        recipe.name = name.clone();
    }
    if let Some(emoji) = &request.emoji {
        recipe.emoji = Some(emoji.clone());
    }
    if let Some(ingredients) = &request.ingredients {
        recipe.ingredients = ingredients.clone();
    }
    if let Some(age_min) = request.age_min {
        recipe.age_min = Some(age_min);
    }
    if let Some(texture) = &request.texture {
        recipe.texture = Some(texture.clone());
    }
    if let Some(allergens) = &request.allergens {
        recipe.allergens = allergens.clone();
    }
    if let Some(prep_notes) = &request.prep_notes {
        recipe.prep_notes = Some(prep_notes.clone());
    }
    if let Some(safe) = request.safe {
        recipe.safe = Some(safe);
    }
    recipe.updated_at = Timestamp::now();

    let updated_recipe = recipe_repository.update(recipe).await?;
    let response = RecipeResponse::from(updated_recipe);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for DELETE /families/{family_id}/recipes/{recipe_id}
pub async fn delete_recipe<F: FamilyRepository, R: RecipeRepository>(
    family_id: FamilyId,
    recipe_id: RecipeId,
    context: &RequestContext,
    family_repository: &F,
    recipe_repository: &R,
) -> Result<(u16, String), HandlerError> {
    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    recipe_repository.delete(family_id, recipe_id).await?;

    Ok((204, String::new()))
}

/// Handler for GET /families/{family_id}/recipes
pub async fn list_recipes<F: FamilyRepository, R: RecipeRepository>(
    family_id: FamilyId,
    context: &RequestContext,
    family_repository: &F,
    recipe_repository: &R,
    pagination: PaginationParams,
) -> Result<(u16, String), HandlerError> {
    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentRead,
    )?;

    let paginated_result = recipe_repository
        .list_by_family(family_id, pagination)
        .await?;

    let recipes_response: Vec<RecipeResponse> = paginated_result
        .items
        .into_iter()
        .map(RecipeResponse::from)
        .collect();

    let response = RecipeListResponse {
        recipes: recipes_response,
        next_token: paginated_result.next_token,
    };

    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Family, IdentityId};
    use crate::test_utils::mocks::{MockFamilyRepository, MockRecipeRepository};

    fn create_test_context(identity_id: &str) -> RequestContext {
        RequestContext {
            identity_id: IdentityId::new(identity_id.to_string()),
            username: None,
        }
    }

    fn create_test_family(family_id: FamilyId, owner_id: &str) -> Family {
        Family {
            id: family_id,
            name: "Test Family".to_string(),
            owner_id: IdentityId::new(owner_id.to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        }
    }

    #[tokio::test]
    async fn test_create_recipe_success() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let recipe_repo = MockRecipeRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "name": "Mashed Banana", "emoji": "🍌", "ingredients": ["banana"], "age_min": 6}}"#,
            family_id.0
        );

        let result = create_recipe(
            family_id,
            &request_body,
            &context,
            &family_repo,
            &recipe_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 201);

        let response: RecipeResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.name, "Mashed Banana");
        assert_eq!(response.family_id, family_id);
    }

    #[tokio::test]
    async fn test_create_recipe_invalid_json() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let recipe_repo = MockRecipeRepository::new();

        let request_body = r#"{"name": invalid}"#;
        let result = create_recipe(
            family_id,
            request_body,
            &context,
            &family_repo,
            &recipe_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "body");
                assert!(err.message.contains("Invalid JSON"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_recipe_empty_name() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let recipe_repo = MockRecipeRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(r#"{{"family_id": "{}", "name": ""}}"#, family_id.0);

        let result = create_recipe(
            family_id,
            &request_body,
            &context,
            &family_repo,
            &recipe_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "name");
                assert!(err.message.contains("empty"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_recipe_family_not_found() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let recipe_repo = MockRecipeRepository::new();

        let request_body = format!(
            r#"{{"family_id": "{}", "name": "Mashed Banana"}}"#,
            family_id.0
        );

        let result = create_recipe(
            family_id,
            &request_body,
            &context,
            &family_repo,
            &recipe_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_get_recipe_success() {
        let family_id = FamilyId::new();
        let recipe_id = RecipeId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let recipe_repo = MockRecipeRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let recipe = Recipe {
            id: recipe_id,
            family_id,
            name: "Mashed Banana".to_string(),
            emoji: Some("🍌".to_string()),
            ingredients: vec!["banana".to_string()],
            age_min: Some(6),
            texture: Some("smooth".to_string()),
            allergens: vec![],
            prep_notes: Some("Mash ripe banana".to_string()),
            safe: Some(true),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        recipe_repo
            .recipes
            .lock()
            .unwrap()
            .insert(recipe_id, recipe);

        let result = get_recipe(family_id, recipe_id, &context, &family_repo, &recipe_repo).await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: RecipeResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.name, "Mashed Banana");
    }

    #[tokio::test]
    async fn test_get_recipe_not_found() {
        let family_id = FamilyId::new();
        let recipe_id = RecipeId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let recipe_repo = MockRecipeRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let result = get_recipe(family_id, recipe_id, &context, &family_repo, &recipe_repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_list_recipes_success() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let recipe_repo = MockRecipeRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let recipe1 = Recipe {
            id: RecipeId::new(),
            family_id,
            name: "Mashed Banana".to_string(),
            emoji: Some("🍌".to_string()),
            ingredients: vec!["banana".to_string()],
            age_min: Some(6),
            texture: Some("smooth".to_string()),
            allergens: vec![],
            prep_notes: None,
            safe: Some(true),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        let recipe2 = Recipe {
            id: RecipeId::new(),
            family_id,
            name: "Sweet Potato Puree".to_string(),
            emoji: Some("🍠".to_string()),
            ingredients: vec!["sweet potato".to_string()],
            age_min: Some(8),
            texture: Some("smooth".to_string()),
            allergens: vec![],
            prep_notes: None,
            safe: Some(true),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        recipe_repo
            .recipes
            .lock()
            .unwrap()
            .insert(recipe1.id, recipe1);
        recipe_repo
            .recipes
            .lock()
            .unwrap()
            .insert(recipe2.id, recipe2);

        let pagination = PaginationParams {
            limit: None,
            next_token: None,
        };
        let result =
            list_recipes(family_id, &context, &family_repo, &recipe_repo, pagination).await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: RecipeListResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.recipes.len(), 2);
        assert!(response.next_token.is_none());
    }

    #[tokio::test]
    async fn test_delete_recipe_success() {
        let family_id = FamilyId::new();
        let recipe_id = RecipeId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let recipe_repo = MockRecipeRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let recipe = Recipe {
            id: recipe_id,
            family_id,
            name: "Mashed Banana".to_string(),
            emoji: None,
            ingredients: vec![],
            age_min: None,
            texture: None,
            allergens: vec![],
            prep_notes: None,
            safe: None,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        recipe_repo
            .recipes
            .lock()
            .unwrap()
            .insert(recipe_id, recipe);

        let result =
            delete_recipe(family_id, recipe_id, &context, &family_repo, &recipe_repo).await;

        assert!(result.is_ok());
        let (status, _) = result.unwrap();
        assert_eq!(status, 204);
    }
}

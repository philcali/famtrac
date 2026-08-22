// Recipe route handlers
//
// All recipe routes are nested under /families/{family_id}/recipes
// since recipes are subresources of families.

use crate::context::RequestContext;
use crate::domain::{FamilyId, RecipeId};
use crate::errors::HandlerError;
use crate::handlers;
use crate::repository::{DynamoDbFamilyRepository, DynamoDbRecipeRepository};
use crate::router::extractors::extract_uuid_param;
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Route handler for /families/{family_id}/recipes/* routes
///
/// This function handles routing for recipe-related endpoints nested under a family:
/// - GET /families/{family_id}/recipes - List all recipes for a family
/// - POST /families/{family_id}/recipes - Create a new recipe
/// - GET /families/{family_id}/recipes/{id} - Get a recipe by ID
/// - PUT /families/{family_id}/recipes/{id} - Update a recipe
/// - DELETE /families/{family_id}/recipes/{id} - Delete a recipe
#[allow(clippy::too_many_arguments)]
pub async fn route_recipe(
    method: &str,
    family_id: FamilyId,
    sub_path: &str,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    recipe_repo: &DynamoDbRecipeRepository,
) -> Result<serde_json::Value, HandlerError> {
    match (method, sub_path) {
        // GET /families/{family_id}/recipes - List all recipes
        ("GET", "") | ("GET", "/") => {
            let query_params = &request.query_string_parameters;
            let pagination = handlers::PaginationParams {
                limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                next_token: query_params.first("next_token").map(|s| s.to_string()),
            };
            let (_status, response_json) =
                handlers::list_recipes(family_id, context, family_repo, recipe_repo, pagination)
                    .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // POST /families/{family_id}/recipes - Create a new recipe
        ("POST", "") | ("POST", "/") => {
            let (_status, response_json) =
                handlers::create_recipe(family_id, body, context, family_repo, recipe_repo).await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // GET /families/{family_id}/recipes/{id} - Get a recipe by ID
        ("GET", p) if !p.is_empty() && p != "/" => {
            let recipe_id =
                extract_uuid_param(&format!("/recipes{}", sub_path), "/recipes/", "recipe_id")?;
            let (_status, response_json) = handlers::get_recipe(
                family_id,
                RecipeId(recipe_id),
                context,
                family_repo,
                recipe_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // PUT /families/{family_id}/recipes/{id} - Update a recipe
        ("PUT", p) if !p.is_empty() && p != "/" => {
            let recipe_id =
                extract_uuid_param(&format!("/recipes{}", sub_path), "/recipes/", "recipe_id")?;
            let (_status, response_json) = handlers::update_recipe(
                family_id,
                RecipeId(recipe_id),
                body,
                context,
                family_repo,
                recipe_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // DELETE /families/{family_id}/recipes/{id} - Delete a recipe
        ("DELETE", p) if !p.is_empty() && p != "/" => {
            let recipe_id =
                extract_uuid_param(&format!("/recipes{}", sub_path), "/recipes/", "recipe_id")?;
            handlers::delete_recipe(
                family_id,
                RecipeId(recipe_id),
                context,
                family_repo,
                recipe_repo,
            )
            .await?;
            Ok(serde_json::Value::Null)
        }

        // Unknown route
        _ => Err(HandlerError::NotFound(format!(
            "Route not found: {} /families/{}/recipes{}",
            method, family_id.0, sub_path
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_uuid_extraction_for_get_recipe() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/recipes/{}", uuid_str);

        let result = extract_uuid_param(&path, "/recipes/", "recipe_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_invalid_uuid_returns_validation_error() {
        let path = "/recipes/not-a-uuid";

        let result = extract_uuid_param(path, "/recipes/", "recipe_id");
        assert!(result.is_err());

        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "recipe_id");
                assert_eq!(err.message, "Invalid recipe_id format");
                assert_eq!(err.constraint, Some("must be a valid UUID".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}

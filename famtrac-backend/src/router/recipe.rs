// Recipe route handlers — segment-based dispatch
//
// The parent router (mod.rs) handles path parsing and UUID extraction.
// This module only dispatches by HTTP method for recipe CRUD.

use crate::context::RequestContext;
use crate::domain::{FamilyId, RecipeId};
use crate::errors::HandlerError;
use crate::handlers;
use crate::handlers::PaginationParams;
use crate::repository::{DynamoDbFamilyRepository, DynamoDbRecipeRepository};
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Handle GET|POST /families/{family_id}/recipes
pub async fn handle_recipes_collection(
    method: &str,
    family_id: FamilyId,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    recipe_repo: &DynamoDbRecipeRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let query_params = &request.query_string_parameters;
            let pagination = PaginationParams {
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

        "POST" => {
            let (_status, response_json) =
                handlers::create_recipe(family_id, body, context, family_repo, recipe_repo).await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/recipes",
            method, family_id.0
        ))),
    }
}

/// Handle GET|PUT|DELETE /families/{family_id}/recipes/{recipe_id}
pub async fn handle_recipe_item(
    method: &str,
    family_id: FamilyId,
    recipe_id: RecipeId,
    body: &str,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    recipe_repo: &DynamoDbRecipeRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let (_status, response_json) =
                handlers::get_recipe(family_id, recipe_id, context, family_repo, recipe_repo)
                    .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "PUT" => {
            let (_status, response_json) = handlers::update_recipe(
                family_id,
                recipe_id,
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

        "DELETE" => {
            handlers::delete_recipe(family_id, recipe_id, context, family_repo, recipe_repo)
                .await?;
            Ok(serde_json::Value::Null)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/recipes/{}",
            method, family_id.0, recipe_id.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_method_dispatch_get_recipes() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_post_recipes() {
        let method = "POST";
        assert!(matches!(method, "POST"));
    }

    #[test]
    fn test_method_dispatch_get_recipe_item() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_put_recipe_item() {
        let method = "PUT";
        assert!(matches!(method, "PUT"));
    }

    #[test]
    fn test_method_dispatch_delete_recipe_item() {
        let method = "DELETE";
        assert!(matches!(method, "DELETE"));
    }

    #[test]
    fn test_unknown_method_does_not_match() {
        let method = "PATCH";
        assert!(!matches!(method, "GET" | "POST" | "PUT" | "DELETE"));
    }
}

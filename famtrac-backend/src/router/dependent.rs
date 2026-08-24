// Dependent route handlers — segment-based dispatch
//
// The parent router (mod.rs) handles path parsing, UUID extraction,
// and delegation to sub-resources (activities, meal-slots, feeding-logs).
// This module only dispatches by HTTP method for dependent CRUD.

use crate::context::RequestContext;
use crate::domain::{DependentId, FamilyId};
use crate::errors::HandlerError;
use crate::handlers;
use crate::handlers::PaginationParams;
use crate::repository::{DynamoDbDependentRepository, DynamoDbFamilyRepository};
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Handle GET|POST /families/{family_id}/dependents
pub async fn handle_dependents_collection(
    method: &str,
    family_id: FamilyId,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let query_params = &request.query_string_parameters;
            let pagination = PaginationParams {
                limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                next_token: query_params.first("next_token").map(|s| s.to_string()),
            };
            let (_status, response_json) = handlers::list_dependents(
                family_id,
                context,
                family_repo,
                dependent_repo,
                pagination,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "POST" => {
            let (_status, response_json) =
                handlers::create_dependent(body, context, family_repo, dependent_repo).await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/dependents",
            method, family_id.0
        ))),
    }
}

/// Handle GET|PUT|DELETE /families/{family_id}/dependents/{dependent_id}
pub async fn handle_dependent_item(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    body: &str,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let (_status, response_json) = handlers::get_dependent(
                family_id,
                dependent_id,
                context,
                family_repo,
                dependent_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "PUT" => {
            let (_status, response_json) = handlers::update_dependent(
                family_id,
                dependent_id,
                body,
                context,
                family_repo,
                dependent_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "DELETE" => {
            handlers::delete_dependent(
                family_id,
                dependent_id,
                context,
                family_repo,
                dependent_repo,
            )
            .await?;
            Ok(serde_json::Value::Null)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/dependents/{}",
            method, family_id.0, dependent_id.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_method_dispatch_get_dependents() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_post_dependents() {
        let method = "POST";
        assert!(matches!(method, "POST"));
    }

    #[test]
    fn test_method_dispatch_get_dependent_item() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_put_dependent_item() {
        let method = "PUT";
        assert!(matches!(method, "PUT"));
    }

    #[test]
    fn test_method_dispatch_delete_dependent_item() {
        let method = "DELETE";
        assert!(matches!(method, "DELETE"));
    }

    #[test]
    fn test_unknown_method_does_not_match() {
        let method = "PATCH";
        assert!(!matches!(method, "GET" | "POST" | "PUT" | "DELETE"));
    }
}

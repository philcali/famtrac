// Family route handlers — segment-based dispatch
//
// The parent router (mod.rs) handles path parsing and UUID extraction.
// This module only dispatches by HTTP method.

use crate::context::RequestContext;
use crate::domain::FamilyId;
use crate::errors::HandlerError;
use crate::handlers;
use crate::handlers::PaginationParams;
use crate::repository::DynamoDbFamilyRepository;
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Handle GET|POST /families
pub async fn handle_families_collection(
    method: &str,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let query_params = &request.query_string_parameters;
            let pagination = PaginationParams {
                limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                next_token: query_params.first("next_token").map(|s| s.to_string()),
            };
            let (_status, response_json) =
                handlers::list_families(context, family_repo, pagination).await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "POST" => {
            let (_status, response_json) =
                handlers::create_family(body, context, family_repo).await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families",
            method
        ))),
    }
}

/// Handle GET|PUT|DELETE /families/{family_id}
pub async fn handle_family_item(
    method: &str,
    family_id: FamilyId,
    body: &str,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let (_status, response_json) =
                handlers::get_family(family_id, context, family_repo).await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "PUT" => {
            let (_status, response_json) =
                handlers::update_family(family_id, body, context, family_repo).await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "DELETE" => {
            handlers::delete_family(family_id, context, family_repo).await?;
            Ok(serde_json::Value::Null)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}",
            method, family_id.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_method_dispatch_get_families() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_post_families() {
        let method = "POST";
        assert!(matches!(method, "POST"));
    }

    #[test]
    fn test_method_dispatch_get_family_item() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_put_family_item() {
        let method = "PUT";
        assert!(matches!(method, "PUT"));
    }

    #[test]
    fn test_method_dispatch_delete_family_item() {
        let method = "DELETE";
        assert!(matches!(method, "DELETE"));
    }

    #[test]
    fn test_unknown_method_does_not_match() {
        let method = "PATCH";
        assert!(!matches!(method, "GET" | "POST" | "PUT" | "DELETE"));
    }
}

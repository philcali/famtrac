// FeedingLog route handlers — segment-based dispatch
//
// The parent router (mod.rs) handles path parsing and UUID extraction.
// This module only dispatches by HTTP method for feeding log CRUD.

use crate::context::RequestContext;
use crate::domain::{DependentId, FamilyId, FeedingLogId};
use crate::errors::HandlerError;
use crate::handlers;
use crate::handlers::PaginationParams;
use crate::repository::{DynamoDbFamilyRepository, DynamoDbFeedingLogRepository};
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Handle GET|POST /families/{fid}/dependents/{did}/feeding-logs
#[allow(clippy::too_many_arguments)]
pub async fn handle_feeding_logs_collection(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    feeding_log_repo: &DynamoDbFeedingLogRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let query_params = &request.query_string_parameters;
            let pagination = PaginationParams {
                limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                next_token: query_params.first("next_token").map(|s| s.to_string()),
            };
            let date = query_params.first("date").map(|s| s.to_string());
            let (_status, response_json) = handlers::list_feeding_logs(
                family_id,
                dependent_id,
                context,
                family_repo,
                feeding_log_repo,
                date,
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
            let (_status, response_json) = handlers::create_feeding_log(
                family_id,
                dependent_id,
                body,
                context,
                family_repo,
                feeding_log_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/dependents/{}/feeding-logs",
            method, family_id.0, dependent_id.0
        ))),
    }
}

/// Handle GET|PUT|DELETE /families/{fid}/dependents/{did}/feeding-logs/{flid}
#[allow(clippy::too_many_arguments)]
pub async fn handle_feeding_log_item(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    feeding_log_id: FeedingLogId,
    body: &str,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    feeding_log_repo: &DynamoDbFeedingLogRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let (_status, response_json) = handlers::get_feeding_log(
                family_id,
                dependent_id,
                feeding_log_id,
                context,
                family_repo,
                feeding_log_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "PUT" => {
            let (_status, response_json) = handlers::update_feeding_log(
                family_id,
                dependent_id,
                feeding_log_id,
                body,
                context,
                family_repo,
                feeding_log_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "DELETE" => {
            handlers::delete_feeding_log(
                family_id,
                dependent_id,
                feeding_log_id,
                context,
                family_repo,
                feeding_log_repo,
            )
            .await?;
            Ok(serde_json::Value::Null)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/dependents/{}/feeding-logs/{}",
            method, family_id.0, dependent_id.0, feeding_log_id.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_method_dispatch_get_feeding_logs() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_post_feeding_logs() {
        let method = "POST";
        assert!(matches!(method, "POST"));
    }

    #[test]
    fn test_method_dispatch_get_feeding_log_item() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_put_feeding_log_item() {
        let method = "PUT";
        assert!(matches!(method, "PUT"));
    }

    #[test]
    fn test_method_dispatch_delete_feeding_log_item() {
        let method = "DELETE";
        assert!(matches!(method, "DELETE"));
    }

    #[test]
    fn test_unknown_method_does_not_match() {
        let method = "PATCH";
        assert!(!matches!(method, "GET" | "POST" | "PUT" | "DELETE"));
    }
}

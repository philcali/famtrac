// FeedingLog route handlers
//
// All feeding log routes are nested under /families/{family_id}/dependents/{dependent_id}/feeding-logs

use crate::context::RequestContext;
use crate::domain::{DependentId, FamilyId, FeedingLogId};
use crate::errors::HandlerError;
use crate::handlers;
use crate::repository::{DynamoDbFamilyRepository, DynamoDbFeedingLogRepository};
use crate::router::extractors::extract_uuid_param;
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
use serde_json::json;

/// Route handler for /families/{family_id}/dependents/{dependent_id}/feeding-logs/* routes
#[allow(clippy::too_many_arguments)]
pub async fn route_feeding_log(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    sub_path: &str,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    feeding_log_repo: &DynamoDbFeedingLogRepository,
) -> Result<serde_json::Value, HandlerError> {
    match (method, sub_path) {
        // GET /families/{family_id}/dependents/{dependent_id}/feeding-logs - List feeding logs
        ("GET", "") | ("GET", "/") => {
            let query_params = &request.query_string_parameters;

            let limit = query_params
                .first("limit")
                .and_then(|s| s.parse::<u32>().ok());
            let next_token = query_params.first("next_token").map(String::from);
            let date = query_params.first("date").map(String::from);

            let pagination = handlers::PaginationParams { limit, next_token };

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

        // POST /families/{family_id}/dependents/{dependent_id}/feeding-logs - Create feeding log
        ("POST", "") | ("POST", "/") => {
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

        // GET /families/{family_id}/dependents/{dependent_id}/feeding-logs/{id} - Get feeding log
        ("GET", p) if !p.is_empty() && p != "/" => {
            let feeding_log_id = extract_uuid_param(
                &format!("/dependents/{}/feeding-logs{}", dependent_id.0, sub_path),
                &format!("/dependents/{}/feeding-logs", dependent_id.0),
                "feeding_log_id",
            )?;
            let (_status, response_json) = handlers::get_feeding_log(
                family_id,
                dependent_id,
                FeedingLogId(feeding_log_id),
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

        // PUT /families/{family_id}/dependents/{dependent_id}/feeding-logs/{id} - Update feeding log
        ("PUT", p) if !p.is_empty() && p != "/" => {
            let feeding_log_id = extract_uuid_param(
                &format!("/dependents/{}/feeding-logs{}", dependent_id.0, sub_path),
                &format!("/dependents/{}/feeding-logs", dependent_id.0),
                "feeding_log_id",
            )?;
            let (_status, response_json) = handlers::update_feeding_log(
                family_id,
                dependent_id,
                FeedingLogId(feeding_log_id),
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

        // DELETE /families/{family_id}/dependents/{dependent_id}/feeding-logs/{id} - Delete feeding log
        ("DELETE", p) if !p.is_empty() && p != "/" => {
            let feeding_log_id = extract_uuid_param(
                &format!("/dependents/{}/feeding-logs{}", dependent_id.0, sub_path),
                &format!("/dependents/{}/feeding-logs", dependent_id.0),
                "feeding_log_id",
            )?;
            handlers::delete_feeding_log(
                family_id,
                dependent_id,
                FeedingLogId(feeding_log_id),
                context,
                family_repo,
                feeding_log_repo,
            )
            .await?;
            Ok(json!(null))
        }

        // Unknown route
        _ => Err(HandlerError::NotFound(format!(
            "Route not found: {} /families/{}/dependents/{}/feeding-logs{}",
            method, family_id.0, dependent_id.0, sub_path
        ))),
    }
}

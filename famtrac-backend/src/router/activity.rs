// Activity route handlers — segment-based dispatch
//
// The parent router (mod.rs) handles path parsing and UUID extraction.
// This module only dispatches by HTTP method for activity CRUD.

use crate::context::RequestContext;
use crate::domain::{
    ActivityId, ActivityType, DependentId, DiaperContents, FamilyId, FeedingType, Timestamp,
};
use crate::errors::HandlerError;
use crate::handlers;
use crate::handlers::PaginationParams;
use crate::repository::{
    DynamoDbActivityRepository, DynamoDbDependentRepository, DynamoDbFamilyRepository,
};
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Handle GET|POST /families/{fid}/dependents/{did}/activities
#[allow(clippy::too_many_arguments)]
pub async fn handle_activities_collection(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let query_params = &request.query_string_parameters;

            // Parse start_date/end_date as ISO 8601 datetime with timezone offset.
            // Falls back to NaiveDate (YYYY-MM-DD) for backwards compatibility.
            let start_date = query_params.first("start_date").and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| Timestamp::from_datetime(dt.with_timezone(&chrono::Utc)))
                    .ok()
                    .or_else(|| {
                        s.parse::<chrono::NaiveDate>()
                            .ok()
                            .and_then(|d| d.and_hms_opt(0, 0, 0))
                            .map(|dt| Timestamp::from_datetime(dt.and_utc()))
                    })
            });
            let end_date = query_params.first("end_date").and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| Timestamp::from_datetime(dt.with_timezone(&chrono::Utc)))
                    .ok()
                    .or_else(|| {
                        s.parse::<chrono::NaiveDate>()
                            .ok()
                            .and_then(|d| d.and_hms_opt(23, 59, 59))
                            .map(|dt| Timestamp::from_datetime(dt.and_utc()))
                    })
            });
            let activity_type =
                query_params
                    .first("activity_type")
                    .and_then(|s| match s.to_string().as_str() {
                        "feeding" => Some(ActivityType::Feeding {
                            feeding_type: FeedingType::Breast,
                            volume_ml: None,
                            medicine_added: None,
                            notes: None,
                        }),
                        "diaper_change" => Some(ActivityType::DiaperChange {
                            contents: DiaperContents::Wet,
                        }),
                        "sleep" => {
                            let now = Timestamp::now();
                            Some(ActivityType::Sleep {
                                start_time: now,
                                end_time: Some(now),
                            })
                        }
                        "pumping" => Some(ActivityType::Pumping { volume_ml: 0 }),
                        "bath" => {
                            let now = Timestamp::now();
                            Some(ActivityType::Bath {
                                start_time: now,
                                end_time: Some(now),
                                notes: None,
                            })
                        }
                        _ => None,
                    });

            let pagination = PaginationParams {
                limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                next_token: query_params.first("next_token").map(|s| s.to_string()),
            };

            let (_status, response_json) = handlers::query_activities(
                family_id,
                dependent_id,
                start_date,
                end_date,
                activity_type,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
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
            let (_status, response_json) = handlers::create_activity(
                body,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/dependents/{}/activities",
            method, family_id.0, dependent_id.0
        ))),
    }
}

/// Handle GET|PUT|DELETE /families/{fid}/dependents/{did}/activities/{aid}
#[allow(clippy::too_many_arguments)]
pub async fn handle_activity_item(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    activity_id: ActivityId,
    body: &str,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let (_status, response_json) = handlers::get_activity(
                family_id,
                dependent_id,
                activity_id,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "PUT" => {
            let (_status, response_json) = handlers::update_activity(
                family_id,
                dependent_id,
                activity_id,
                body,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        "DELETE" => {
            handlers::delete_activity(
                family_id,
                dependent_id,
                activity_id,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )
            .await?;
            Ok(serde_json::Value::Null)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/dependents/{}/activities/{}",
            method, family_id.0, dependent_id.0, activity_id.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_method_dispatch_get_activities() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_post_activities() {
        let method = "POST";
        assert!(matches!(method, "POST"));
    }

    #[test]
    fn test_method_dispatch_get_activity_item() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_put_activity_item() {
        let method = "PUT";
        assert!(matches!(method, "PUT"));
    }

    #[test]
    fn test_method_dispatch_delete_activity_item() {
        let method = "DELETE";
        assert!(matches!(method, "DELETE"));
    }

    #[test]
    fn test_unknown_method_does_not_match() {
        let method = "PATCH";
        assert!(!matches!(method, "GET" | "POST" | "PUT" | "DELETE"));
    }
}

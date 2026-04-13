// Activity route handlers
//
// All activity routes are nested under /families/{family_id}/dependents/{dependent_id}/activities
// since activities are subresources of dependents within families.

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
use crate::router::extractors::extract_uuid_param;
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Route handler for /families/{fid}/dependents/{did}/activities/* routes
///
/// This function handles routing for activity-related endpoints nested under a dependent:
/// - POST   .../activities - Create a new activity
/// - GET    .../activities - Query activities (with query params)
/// - GET    .../activities/{id} - Get an activity by ID
/// - PUT    .../activities/{id} - Update an activity
/// - DELETE .../activities/{id} - Delete an activity
#[allow(clippy::too_many_arguments)]
pub async fn route_activity(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    sub_path: &str,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
) -> Result<serde_json::Value, HandlerError> {
    match (method, sub_path) {
        // POST .../activities - Create a new activity
        ("POST", "") | ("POST", "/") => {
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

        // GET .../activities - Query activities for a dependent (with query params)
        ("GET", "") | ("GET", "/") => {
            let query_params = &request.query_string_parameters;

            // Parse start_date/end_date as ISO 8601 datetime with timezone offset.
            // The client sends the local day boundaries with its UTC offset so the
            // server can compute the correct UTC range for the user's timezone.
            // Falls back to NaiveDate (YYYY-MM-DD) for backwards compatibility,
            // treating it as UTC midnight.
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

        // GET .../activities/{id} - Get an activity by ID
        ("GET", p) if !p.is_empty() && p != "/" => {
            let activity_id = extract_uuid_param(
                &format!("/activities{}", sub_path),
                "/activities/",
                "activity_id",
            )?;
            let (_status, response_json) = handlers::get_activity(
                family_id,
                dependent_id,
                ActivityId(activity_id),
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

        // PUT .../activities/{id} - Update an activity
        ("PUT", p) if !p.is_empty() && p != "/" => {
            let activity_id = extract_uuid_param(
                &format!("/activities{}", sub_path),
                "/activities/",
                "activity_id",
            )?;
            let (_status, response_json) = handlers::update_activity(
                family_id,
                dependent_id,
                ActivityId(activity_id),
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

        // DELETE .../activities/{id} - Delete an activity
        ("DELETE", p) if !p.is_empty() && p != "/" => {
            let activity_id = extract_uuid_param(
                &format!("/activities{}", sub_path),
                "/activities/",
                "activity_id",
            )?;
            let (_status, _response_json) = handlers::delete_activity(
                family_id,
                dependent_id,
                ActivityId(activity_id),
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )
            .await?;
            Ok(serde_json::Value::Null)
        }

        // Unknown route
        _ => Err(HandlerError::NotFound(format!(
            "Route not found: {} /families/{}/dependents/{}/activities{}",
            method, family_id.0, dependent_id.0, sub_path
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_uuid_extraction_for_get_activity() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/activities/{}", uuid_str);

        let result = extract_uuid_param(&path, "/activities/", "activity_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_invalid_activity_uuid_returns_validation_error() {
        let path = "/activities/not-a-uuid";

        let result = extract_uuid_param(path, "/activities/", "activity_id");
        assert!(result.is_err());

        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "activity_id");
                assert_eq!(err.message, "Invalid activity_id format");
                assert_eq!(err.constraint, Some("must be a valid UUID".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}

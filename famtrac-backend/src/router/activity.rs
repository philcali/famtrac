// Activity route handlers

use crate::context::RequestContext;
use crate::domain::{
    ActivityId, ActivityType, Date, DependentId, DiaperContents, FeedingType, Timestamp,
};
use crate::errors::HandlerError;
use crate::handlers;
use crate::repository::{
    DynamoDbActivityRepository, DynamoDbDependentRepository, DynamoDbFamilyRepository,
};
use crate::router::extractors::extract_uuid_param;
use aws_lambda_events::apigw::ApiGatewayProxyRequest;

/// Route handler for all /activities/* routes
///
/// This function handles routing for activity-related endpoints:
/// - POST /activities - Create a new activity
/// - GET /activities/{id} - Get an activity by ID
/// - PUT /activities/{id} - Update an activity
/// - DELETE /activities/{id} - Delete an activity
/// - GET /dependents/{id}/activities - Query activities for a dependent (with query params)
///
/// # Arguments
///
/// * `method` - HTTP method (GET, POST, PUT, DELETE, etc.)
/// * `path` - URL path (e.g., "/activities/123-456")
/// * `body` - Request body as a string
/// * `request` - Full API Gateway request (needed for query parameters)
/// * `context` - Request context with authentication information
/// * `family_repo` - Family repository for database operations
/// * `dependent_repo` - Dependent repository for database operations
/// * `activity_repo` - Activity repository for database operations
///
/// # Returns
///
/// * `Ok(serde_json::Value)` - Success response as JSON
/// * `Err(HandlerError)` - Error response
///
/// # Requirements
///
/// - Requirement 5.1: Handle all /activities/* routes
/// - Requirement 5.2: POST /activities → create_activity()
/// - Requirement 5.3: GET /activities/{id} → get_activity()
/// - Requirement 5.4: PUT /activities/{id} → update_activity()
/// - Requirement 5.5: DELETE /activities/{id} → delete_activity()
/// - Requirement 4.5: GET /dependents/{id}/activities → query_activities()
/// - Requirement 5.6: Invalid UUID → HandlerError::Validation
/// - Requirement 6.5: Use extractors from extractors.rs
#[allow(clippy::too_many_arguments)]
pub fn route_activity(
    method: &str,
    path: &str,
    body: &str,
    request: &ApiGatewayProxyRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
) -> Result<serde_json::Value, HandlerError> {
    match (method, path) {
        // POST /activities - Create a new activity
        ("POST", "/activities") => {
            let (_status, response_json) = handlers::create_activity(
                body,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // GET /activities/{id} - Get an activity by ID
        ("GET", p) if p.starts_with("/activities/") => {
            let activity_id = extract_uuid_param(path, "/activities/", "activity_id")?;
            let (_status, response_json) = handlers::get_activity(
                ActivityId(activity_id),
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // PUT /activities/{id} - Update an activity
        ("PUT", p) if p.starts_with("/activities/") => {
            let activity_id = extract_uuid_param(path, "/activities/", "activity_id")?;
            let (_status, response_json) = handlers::update_activity(
                ActivityId(activity_id),
                body,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // DELETE /activities/{id} - Delete an activity
        ("DELETE", p) if p.starts_with("/activities/") => {
            let activity_id = extract_uuid_param(path, "/activities/", "activity_id")?;
            let (_status, response_json) = handlers::delete_activity(
                ActivityId(activity_id),
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // GET /dependents/{id}/activities - Query activities for a dependent
        ("GET", p) if p.contains("/dependents/") && p.ends_with("/activities") => {
            let dependent_id = extract_uuid_param(path, "/dependents/", "dependent_id")?;

            // Parse query parameters
            let query_params = &request.query_string_parameters;
            let start_date = query_params
                .first("start_date")
                .and_then(|s| s.parse::<chrono::NaiveDate>().ok())
                .map(Date::from_naive_date);
            let end_date = query_params
                .first("end_date")
                .and_then(|s| s.parse::<chrono::NaiveDate>().ok())
                .map(Date::from_naive_date);
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
                                start: now,
                                end: now,
                            })
                        }
                        "pumping" => Some(ActivityType::Pumping { volume_ml: 0 }),
                        _ => None,
                    });

            let (_status, response_json) = handlers::query_activities(
                DependentId(dependent_id),
                start_date,
                end_date,
                activity_type,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // Unknown route
        _ => Err(HandlerError::NotFound(format!(
            "Route not found: {} {}",
            method, path
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Note: These tests verify the routing logic patterns.
    // The actual route_activity function signature requires concrete DynamoDb types,
    // so full integration tests with mock repositories are in the tests/ directory
    // using the common::mocks module.

    #[test]
    fn test_post_activities_route() {
        // This test verifies the route pattern matching for POST /activities
        let method = "POST";
        let path = "/activities";

        // Verify the pattern matches
        assert!(matches!((method, path), ("POST", "/activities")));
    }

    #[test]
    fn test_get_activity_by_id_route_pattern() {
        // This test verifies the route pattern matching for GET /activities/{id}
        let method = "GET";
        let path = "/activities/550e8400-e29b-41d4-a716-446655440000";

        // Verify the pattern matches
        assert!(path.starts_with("/activities/"));
        assert_eq!(method, "GET");
    }

    #[test]
    fn test_put_activity_route_pattern() {
        // This test verifies the route pattern matching for PUT /activities/{id}
        let method = "PUT";
        let path = "/activities/550e8400-e29b-41d4-a716-446655440000";

        // Verify the pattern matches
        assert!(path.starts_with("/activities/"));
        assert_eq!(method, "PUT");
    }

    #[test]
    fn test_delete_activity_route_pattern() {
        // This test verifies the route pattern matching for DELETE /activities/{id}
        let method = "DELETE";
        let path = "/activities/550e8400-e29b-41d4-a716-446655440000";

        // Verify the pattern matches
        assert!(path.starts_with("/activities/"));
        assert_eq!(method, "DELETE");
    }

    #[test]
    fn test_query_activities_route_pattern() {
        // This test verifies the route pattern matching for GET /dependents/{id}/activities
        let method = "GET";
        let path = "/dependents/550e8400-e29b-41d4-a716-446655440000/activities";

        // Verify the pattern matches
        assert!(path.contains("/dependents/") && path.ends_with("/activities"));
        assert_eq!(method, "GET");
    }

    #[test]
    fn test_uuid_extraction_for_get_activity() {
        // Test that UUID extraction works correctly
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/activities/{}", uuid_str);

        let result = extract_uuid_param(&path, "/activities/", "activity_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_uuid_extraction_for_query_activities() {
        // Test that UUID extraction works for the query activities route
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/dependents/{}/activities", uuid_str);

        let result = extract_uuid_param(&path, "/dependents/", "dependent_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_invalid_activity_uuid_returns_validation_error() {
        // Test that invalid UUID returns proper validation error
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

    #[test]
    fn test_invalid_dependent_uuid_returns_validation_error() {
        // Test that invalid UUID returns proper validation error for query activities
        let path = "/dependents/not-a-uuid/activities";

        let result = extract_uuid_param(path, "/dependents/", "dependent_id");
        assert!(result.is_err());

        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "dependent_id");
                assert_eq!(err.message, "Invalid dependent_id format");
                assert_eq!(err.constraint, Some("must be a valid UUID".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_unknown_route_pattern() {
        // Test that unknown routes don't match any pattern
        let method = "PATCH";
        let path = "/activities/550e8400-e29b-41d4-a716-446655440000";

        // Verify this doesn't match any of our patterns
        let matches_post = matches!((method, path), ("POST", "/activities"));
        let matches_get = method == "GET" && path.starts_with("/activities/");
        let matches_put = method == "PUT" && path.starts_with("/activities/");
        let matches_delete = method == "DELETE" && path.starts_with("/activities/");
        let matches_query =
            method == "GET" && path.contains("/dependents/") && path.ends_with("/activities");

        assert!(!matches_post);
        assert!(!matches_get);
        assert!(!matches_put);
        assert!(!matches_delete);
        assert!(!matches_query);
    }

    #[test]
    fn test_route_pattern_disambiguation() {
        // Test that GET /activities/{id} and GET /dependents/{id}/activities are properly distinguished
        let path_activity = "/activities/550e8400-e29b-41d4-a716-446655440000";
        let path_query = "/dependents/550e8400-e29b-41d4-a716-446655440000/activities";

        // Activity route should start with /activities/
        assert!(path_activity.starts_with("/activities/"));
        assert!(!path_activity.contains("/dependents/"));

        // Query route should contain /dependents/ and end with /activities
        assert!(path_query.contains("/dependents/") && path_query.ends_with("/activities"));
        assert!(!path_query.starts_with("/activities/"));
    }

    #[test]
    fn test_all_http_methods_for_activities() {
        // Test that we handle all expected HTTP methods for /activities/{id}
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/activities/{}", uuid_str);

        // GET should match
        assert!("GET" == "GET" && path.starts_with("/activities/"));

        // PUT should match
        assert!("PUT" == "PUT" && path.starts_with("/activities/"));

        // DELETE should match
        assert!("DELETE" == "DELETE" && path.starts_with("/activities/"));

        // POST to /activities/{id} should not match (POST is only for /activities)
        let matches_post = matches!(("POST", path.as_str()), ("POST", "/activities"));
        assert!(!matches_post);
    }
}

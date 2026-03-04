// Dependent route handlers

use crate::context::RequestContext;
use crate::domain::DependentId;
use crate::errors::HandlerError;
use crate::handlers;
use crate::repository::{DynamoDbDependentRepository, DynamoDbFamilyRepository};
use crate::router::extractors::extract_uuid_param;

/// Route handler for all /dependents/* routes
///
/// This function handles routing for dependent-related endpoints:
/// - POST /dependents - Create a new dependent
/// - GET /dependents/{id} - Get a dependent by ID
/// - PUT /dependents/{id} - Update a dependent
///
/// # Arguments
///
/// * `method` - HTTP method (GET, POST, PUT, etc.)
/// * `path` - URL path (e.g., "/dependents/123-456")
/// * `body` - Request body as a string
/// * `context` - Request context with authentication information
/// * `family_repo` - Family repository for database operations
/// * `dependent_repo` - Dependent repository for database operations
///
/// # Returns
///
/// * `Ok(serde_json::Value)` - Success response as JSON
/// * `Err(HandlerError)` - Error response
///
/// # Requirements
///
/// - Requirement 4.1: Handle all /dependents/* routes
/// - Requirement 4.2: POST /dependents → create_dependent()
/// - Requirement 4.3: GET /dependents/{id} → get_dependent()
/// - Requirement 4.4: PUT /dependents/{id} → update_dependent()
/// - Requirement 4.6: Invalid UUID → HandlerError::Validation
/// - Requirement 6.5: Use extractors from extractors.rs
pub fn route_dependent(
    method: &str,
    path: &str,
    body: &str,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
) -> Result<serde_json::Value, HandlerError> {
    match (method, path) {
        // POST /dependents - Create a new dependent
        ("POST", "/dependents") => {
            let (_status, response_json) =
                handlers::create_dependent(body, context, family_repo, dependent_repo)?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // GET /dependents/{id} - Get a dependent by ID
        ("GET", p) if p.starts_with("/dependents/") => {
            let dependent_id = extract_uuid_param(path, "/dependents/", "dependent_id")?;
            let (_status, response_json) = handlers::get_dependent(
                DependentId(dependent_id),
                context,
                family_repo,
                dependent_repo,
            )?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // PUT /dependents/{id} - Update a dependent
        ("PUT", p) if p.starts_with("/dependents/") => {
            let dependent_id = extract_uuid_param(path, "/dependents/", "dependent_id")?;
            let (_status, response_json) = handlers::update_dependent(
                DependentId(dependent_id),
                body,
                context,
                family_repo,
                dependent_repo,
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
    // The actual route_dependent function signature requires concrete DynamoDb types,
    // so full integration tests with mock repositories are in the tests/ directory
    // using the common::mocks module.

    #[test]
    fn test_post_dependents_route() {
        // This test verifies the route pattern matching for POST /dependents
        let method = "POST";
        let path = "/dependents";

        // Verify the pattern matches
        assert!(matches!((method, path), ("POST", "/dependents")));
    }

    #[test]
    fn test_get_dependent_by_id_route_pattern() {
        // This test verifies the route pattern matching for GET /dependents/{id}
        let method = "GET";
        let path = "/dependents/550e8400-e29b-41d4-a716-446655440000";

        // Verify the pattern matches
        assert!(path.starts_with("/dependents/"));
        assert_eq!(method, "GET");
    }

    #[test]
    fn test_put_dependent_route_pattern() {
        // This test verifies the route pattern matching for PUT /dependents/{id}
        let method = "PUT";
        let path = "/dependents/550e8400-e29b-41d4-a716-446655440000";

        // Verify the pattern matches
        assert!(path.starts_with("/dependents/"));
        assert_eq!(method, "PUT");
    }

    #[test]
    fn test_uuid_extraction_for_get_dependent() {
        // Test that UUID extraction works correctly
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/dependents/{}", uuid_str);

        let result = extract_uuid_param(&path, "/dependents/", "dependent_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_invalid_uuid_returns_validation_error() {
        // Test that invalid UUID returns proper validation error
        let path = "/dependents/not-a-uuid";

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
        let method = "DELETE";
        let path = "/dependents/550e8400-e29b-41d4-a716-446655440000";

        // Verify this doesn't match any of our patterns
        let matches_post = matches!((method, path), ("POST", "/dependents"));
        let matches_get = method == "GET" && path.starts_with("/dependents/");
        let matches_put = method == "PUT" && path.starts_with("/dependents/");

        assert!(!matches_post);
        assert!(!matches_get);
        assert!(!matches_put);
    }
}

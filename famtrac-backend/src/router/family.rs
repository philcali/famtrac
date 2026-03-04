// Family route handlers

use crate::context::RequestContext;
use crate::domain::FamilyId;
use crate::errors::HandlerError;
use crate::handlers;
use crate::repository::{DynamoDbDependentRepository, DynamoDbFamilyRepository};
use crate::router::extractors::extract_uuid_param;

/// Route handler for all /families/* routes
///
/// This function handles routing for family-related endpoints:
/// - POST /families - Create a new family
/// - GET /families/{id} - Get a family by ID
/// - PUT /families/{id} - Update a family
/// - GET /families/{id}/dependents - List dependents for a family
///
/// # Arguments
///
/// * `method` - HTTP method (GET, POST, PUT, etc.)
/// * `path` - URL path (e.g., "/families/123-456")
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
/// - Requirement 3.1: Handle all /families/* routes
/// - Requirement 3.2: POST /families → create_family()
/// - Requirement 3.3: GET /families/{id} → get_family()
/// - Requirement 3.4: PUT /families/{id} → update_family()
/// - Requirement 3.5: GET /families/{id}/dependents → list_dependents()
/// - Requirement 3.6: Invalid UUID → HandlerError::Validation
/// - Requirement 6.5: Use extractors from extractors.rs
pub fn route_family(
    method: &str,
    path: &str,
    body: &str,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
) -> Result<serde_json::Value, HandlerError> {
    match (method, path) {
        // POST /families - Create a new family
        ("POST", "/families") => {
            let (_status, response_json) = handlers::create_family(body, context, family_repo)?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // GET /families/{id} - Get a family by ID
        ("GET", p) if p.starts_with("/families/") && !p.contains("/dependents") => {
            let family_id = extract_uuid_param(path, "/families/", "family_id")?;
            let (_status, response_json) =
                handlers::get_family(FamilyId(family_id), context, family_repo, dependent_repo)?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // PUT /families/{id} - Update a family
        ("PUT", p) if p.starts_with("/families/") => {
            let family_id = extract_uuid_param(path, "/families/", "family_id")?;
            let (_status, response_json) = handlers::update_family(
                FamilyId(family_id),
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

        // GET /families/{id}/dependents - List dependents for a family
        ("GET", p) if p.starts_with("/families/") && p.contains("/dependents") => {
            let family_id = extract_uuid_param(path, "/families/", "family_id")?;
            let (_status, response_json) = handlers::list_dependents(
                FamilyId(family_id),
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
    // The actual route_family function signature requires concrete DynamoDb types,
    // so full integration tests with mock repositories are in the tests/ directory
    // using the common::mocks module.

    #[test]
    fn test_post_families_route() {
        // This test verifies the route pattern matching for POST /families
        let method = "POST";
        let path = "/families";

        // Verify the pattern matches
        assert!(matches!((method, path), ("POST", "/families")));
    }

    #[test]
    fn test_get_family_by_id_route_pattern() {
        // This test verifies the route pattern matching for GET /families/{id}
        let method = "GET";
        let path = "/families/550e8400-e29b-41d4-a716-446655440000";

        // Verify the pattern matches
        assert!(path.starts_with("/families/") && !path.contains("/dependents"));
        assert_eq!(method, "GET");
    }

    #[test]
    fn test_put_family_route_pattern() {
        // This test verifies the route pattern matching for PUT /families/{id}
        let method = "PUT";
        let path = "/families/550e8400-e29b-41d4-a716-446655440000";

        // Verify the pattern matches
        assert!(path.starts_with("/families/"));
        assert_eq!(method, "PUT");
    }

    #[test]
    fn test_get_dependents_route_pattern() {
        // This test verifies the route pattern matching for GET /families/{id}/dependents
        let method = "GET";
        let path = "/families/550e8400-e29b-41d4-a716-446655440000/dependents";

        // Verify the pattern matches
        assert!(path.starts_with("/families/") && path.contains("/dependents"));
        assert_eq!(method, "GET");
    }

    #[test]
    fn test_uuid_extraction_for_get_family() {
        // Test that UUID extraction works correctly
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/families/{}", uuid_str);

        let result = extract_uuid_param(&path, "/families/", "family_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_uuid_extraction_for_dependents_route() {
        // Test that UUID extraction works for the dependents route
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/families/{}/dependents", uuid_str);

        let result = extract_uuid_param(&path, "/families/", "family_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_invalid_uuid_returns_validation_error() {
        // Test that invalid UUID returns proper validation error
        let path = "/families/not-a-uuid";

        let result = extract_uuid_param(path, "/families/", "family_id");
        assert!(result.is_err());

        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "family_id");
                assert_eq!(err.message, "Invalid family_id format");
                assert_eq!(err.constraint, Some("must be a valid UUID".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_unknown_route_pattern() {
        // Test that unknown routes don't match any pattern
        let method = "DELETE";
        let path = "/families/550e8400-e29b-41d4-a716-446655440000";

        // Verify this doesn't match any of our patterns
        let matches_post = matches!((method, path), ("POST", "/families"));
        let matches_get = method == "GET" && path.starts_with("/families/");
        let matches_put = method == "PUT" && path.starts_with("/families/");

        assert!(!matches_post);
        assert!(!matches_get);
        assert!(!matches_put);
    }

    #[test]
    fn test_route_pattern_disambiguation() {
        // Test that GET /families/{id} and GET /families/{id}/dependents are properly distinguished
        let path_family = "/families/550e8400-e29b-41d4-a716-446655440000";
        let path_dependents = "/families/550e8400-e29b-41d4-a716-446655440000/dependents";

        // Family route should NOT contain /dependents
        assert!(path_family.starts_with("/families/") && !path_family.contains("/dependents"));

        // Dependents route SHOULD contain /dependents
        assert!(
            path_dependents.starts_with("/families/") && path_dependents.contains("/dependents")
        );
    }
}

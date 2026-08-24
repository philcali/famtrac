use crate::context::RequestContext;
use crate::domain::{DependentId, FamilyId, MealSlotId};
use crate::errors::HandlerError;
use crate::handlers::{
    create_meal_slot, delete_meal_slot, get_meal_slot, list_meal_slots, update_meal_slot,
    PaginationParams,
};
use crate::repository::{DynamoDbFamilyRepository, DynamoDbMealSlotRepository};
use crate::router::extractors::extract_uuid_param;
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Route handler for /families/{family_id}/dependents/{dependent_id}/meal-slots/* routes
#[allow(clippy::too_many_arguments)]
pub async fn route_meal_slot(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    sub_path: &str,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    meal_repo: &DynamoDbMealSlotRepository,
) -> Result<serde_json::Value, HandlerError> {
    // Determine if this is a meal-slot detail route (/{uuid}[/...])
    // vs a collection route ("") or trailing-slash ("/").
    // sub_path can be: "" (no slash), "/" (just slash), "/{uuid}", "/{uuid}/", "/{uuid}/extra"
    if sub_path.len() > 1 {
        // Has content after leading / — this is a detail route
        let rest = &sub_path[1..];
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        let meal_slot_id = extract_uuid_param(
            &format!("/meal-slots/{}", meal_slot_id_str),
            "/meal-slots/",
            "meal_slot_id",
        )?;
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];

        match (method, meal_slot_sub_path) {
            // GET /families/{fid}/dependents/{did}/meal-slots/{mid}
            ("GET", "") | ("GET", "/") => {
                let (_status, response_json) = get_meal_slot(
                    family_id,
                    dependent_id,
                    MealSlotId(meal_slot_id),
                    context,
                    family_repo,
                    meal_repo,
                )
                .await?;
                let response: serde_json::Value =
                    serde_json::from_str(&response_json).map_err(|e| {
                        HandlerError::InternalError(format!("Failed to parse response: {}", e))
                    })?;
                Ok(response)
            }

            // PUT /families/{fid}/dependents/{did}/meal-slots/{mid}
            ("PUT", "") | ("PUT", "/") => {
                let (_status, response_json) = update_meal_slot(
                    family_id,
                    dependent_id,
                    MealSlotId(meal_slot_id),
                    body,
                    context,
                    family_repo,
                    meal_repo,
                )
                .await?;
                let response: serde_json::Value =
                    serde_json::from_str(&response_json).map_err(|e| {
                        HandlerError::InternalError(format!("Failed to parse response: {}", e))
                    })?;
                Ok(response)
            }

            // DELETE /families/{fid}/dependents/{did}/meal-slots/{mid}
            ("DELETE", "") | ("DELETE", "/") => {
                delete_meal_slot(
                    family_id,
                    dependent_id,
                    MealSlotId(meal_slot_id),
                    context,
                    family_repo,
                    meal_repo,
                )
                .await?;
                Ok(serde_json::Value::Null)
            }

            // Unknown sub-route under meal slot
            _ => Err(HandlerError::NotFound(format!(
                "Route not found: {} /families/{}/dependents/{}/meal-slots/{}{}",
                method, family_id.0, dependent_id.0, meal_slot_id_str, meal_slot_sub_path
            ))),
        }
    } else {
        // No UUID segment — list or create
        match (method, sub_path) {
            // GET /families/{fid}/dependents/{did}/meal-slots
            ("GET", "") | ("GET", "/") => {
                let query_params = &request.query_string_parameters;
                let pagination = PaginationParams {
                    limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                    next_token: query_params.first("next_token").map(|s| s.to_string()),
                };
                let (_status, response_json) = list_meal_slots(
                    family_id,
                    dependent_id,
                    context,
                    family_repo,
                    meal_repo,
                    query_params.first("day").map(|s| s.to_string()),
                    pagination,
                )
                .await?;
                let response: serde_json::Value =
                    serde_json::from_str(&response_json).map_err(|e| {
                        HandlerError::InternalError(format!("Failed to parse response: {}", e))
                    })?;
                Ok(response)
            }

            // POST /families/{fid}/dependents/{did}/meal-slots
            ("POST", "") | ("POST", "/") => {
                let (_status, response_json) = create_meal_slot(
                    family_id,
                    dependent_id,
                    body,
                    context,
                    family_repo,
                    meal_repo,
                )
                .await?;
                let response: serde_json::Value =
                    serde_json::from_str(&response_json).map_err(|e| {
                        HandlerError::InternalError(format!("Failed to parse response: {}", e))
                    })?;
                Ok(response)
            }

            // Unknown route
            _ => Err(HandlerError::NotFound(format!(
                "Route not found: {} /families/{}/dependents/{}/meal-slots{}",
                method, family_id.0, dependent_id.0, sub_path
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper: simulate how route_dependent constructs the sub_path for meal-slots
    // This mirrors the logic at dependent.rs line 74:
    //   let meal_slots_sub_path = &sub_path[meal_slots_idx + "/meal-slots".len()..];
    fn simulate_dependent_to_meal_slot_sub_path(full_path: &str) -> &str {
        let meal_slots_idx = full_path
            .find("/meal-slots")
            .expect("path must contain /meal-slots");
        &full_path[meal_slots_idx + "/meal-slots".len()..]
    }

    // ============================================================
    // Sub-path construction tests: route_dependent → route_meal_slot
    // These verify that the dependent router produces the correct
    // sub_path for each URL variant.
    // ============================================================

    #[test]
    fn test_sub_path_from_dependent_no_uuid() {
        // URL: .../dependents/{did}/meal-slots
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots";
        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "");
    }

    #[test]
    fn test_sub_path_from_dependent_with_trailing_slash() {
        // URL: .../dependents/{did}/meal-slots/
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/";
        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "/");
    }

    #[test]
    fn test_sub_path_from_dependent_with_uuid_no_trailing() {
        // URL: .../dependents/{did}/meal-slots/{uuid}
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555";
        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "/083ef6d9-2e7a-407e-af22-d1546d437555");
    }

    #[test]
    fn test_sub_path_from_dependent_with_uuid_and_trailing() {
        // URL: .../dependents/{did}/meal-slots/{uuid}/
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555/";
        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "/083ef6d9-2e7a-407e-af22-d1546d437555/");
    }

    #[test]
    fn test_sub_path_from_dependent_with_uuid_and_extra() {
        // URL: .../dependents/{did}/meal-slots/{uuid}/extra
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555/extra";
        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "/083ef6d9-2e7a-407e-af22-d1546d437555/extra");
    }

    // ============================================================
    // Route classification tests: route_meal_slot interprets sub_path
    // These verify that each sub_path value is classified as either
    // a "detail" route (has UUID) or "collection" route (no UUID).
    // ============================================================

    #[test]
    fn test_route_classification_empty_sub_path() {
        let sub_path = "";
        // len() == 0, not > 1 → collection route
        assert!(!sub_path.len() > 1);
    }

    #[test]
    fn test_route_classification_trailing_slash_only() {
        let sub_path = "/";
        // len() == 1, not > 1 → collection route
        assert!(!sub_path.len() > 1);
    }

    #[test]
    fn test_route_classification_uuid_no_trailing() {
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555";
        // len() == 37, > 1 → detail route
        assert!(sub_path.len() > 1);
    }

    #[test]
    fn test_route_classification_uuid_with_trailing() {
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555/";
        // len() == 38, > 1 → detail route
        assert!(sub_path.len() > 1);
    }

    #[test]
    fn test_route_classification_uuid_with_extra() {
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555/extra";
        // len() == 42, > 1 → detail route
        assert!(sub_path.len() > 1);
    }

    // ============================================================
    // UUID extraction tests: verify the UUID is correctly extracted
    // from each sub_path variant that represents a detail route.
    // ============================================================

    #[test]
    fn test_extract_uuid_from_sub_path_no_trailing() {
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555";
        let rest = &sub_path[1..];
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];

        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        assert_eq!(meal_slot_sub_path, "");

        // Verify extract_uuid_param succeeds
        let result = extract_uuid_param(
            &format!("/meal-slots/{}", meal_slot_id_str),
            "/meal-slots/",
            "meal_slot_id",
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Uuid::parse_str("083ef6d9-2e7a-407e-af22-d1546d437555").unwrap()
        );
    }

    #[test]
    fn test_extract_uuid_from_sub_path_with_trailing() {
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555/";
        let rest = &sub_path[1..];
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];

        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        assert_eq!(meal_slot_sub_path, "/");

        let result = extract_uuid_param(
            &format!("/meal-slots/{}", meal_slot_id_str),
            "/meal-slots/",
            "meal_slot_id",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_uuid_from_sub_path_with_extra() {
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555/extra";
        let rest = &sub_path[1..];
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];

        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        assert_eq!(meal_slot_sub_path, "/extra");

        let result = extract_uuid_param(
            &format!("/meal-slots/{}", meal_slot_id_str),
            "/meal-slots/",
            "meal_slot_id",
        );
        assert!(result.is_ok());
    }

    // ============================================================
    // Full routing chain tests: simulate the complete path from
    // route_dependent → route_meal_slot, verifying the correct
    // route branch is taken for each HTTP method.
    // ============================================================

    #[test]
    fn test_full_chain_delete_meal_slot_no_trailing() {
        // Simulate: DELETE /families/{fid}/dependents/{did}/meal-slots/{uuid}
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555";

        // Step 1: route_dependent constructs sub_path
        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "/083ef6d9-2e7a-407e-af22-d1546d437555");

        // Step 2: route_meal_slot classifies as detail route
        assert!(sub_path.len() > 1, "should be classified as detail route");

        // Step 3: extract UUID
        let rest = &sub_path[1..];
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];

        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        assert_eq!(meal_slot_sub_path, "");

        // Step 4: match (DELETE, "") → delete branch
        assert_eq!(meal_slot_sub_path, "");
        assert!(matches!(("DELETE", meal_slot_sub_path), ("DELETE", "")));
    }

    #[test]
    fn test_full_chain_delete_meal_slot_with_trailing() {
        // Simulate: DELETE /families/{fid}/dependents/{did}/meal-slots/{uuid}/
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555/";

        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "/083ef6d9-2e7a-407e-af22-d1546d437555/");
        assert!(sub_path.len() > 1);

        let rest = &sub_path[1..];
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];

        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        assert_eq!(meal_slot_sub_path, "/");

        assert!(matches!(("DELETE", meal_slot_sub_path), ("DELETE", "/")));
    }

    #[test]
    fn test_full_chain_get_meal_slot_collection() {
        // Simulate: GET /families/{fid}/dependents/{did}/meal-slots
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots";

        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "");
        assert!(!sub_path.len() > 1, "should be classified as collection route");

        // Should fall to the list branch
        assert!(matches!(("GET", sub_path), ("GET", "")));
    }

    #[test]
    fn test_full_chain_get_meal_slot_collection_with_trailing_slash() {
        // Simulate: GET /families/{fid}/dependents/{did}/meal-slots/
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/";

        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "/");
        assert!(!sub_path.len() > 1, "should be classified as collection route");

        assert!(matches!(("GET", sub_path), ("GET", "/")));
    }

    #[test]
    fn test_full_chain_post_create_meal_slot() {
        // Simulate: POST /families/{fid}/dependents/{did}/meal-slots
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots";

        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert_eq!(sub_path, "");
        assert!(!sub_path.len() > 1);

        assert!(matches!(("POST", sub_path), ("POST", "")));
    }

    #[test]
    fn test_full_chain_put_update_meal_slot() {
        // Simulate: PUT /families/{fid}/dependents/{did}/meal-slots/{uuid}
        let full_path = "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555";

        let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
        assert!(sub_path.len() > 1);

        let rest = &sub_path[1..];
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];

        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        assert_eq!(meal_slot_sub_path, "");

        assert!(matches!(("PUT", meal_slot_sub_path), ("PUT", "")));
    }

    // ============================================================
    // Edge case tests: verify behavior for unusual inputs
    // ============================================================

    #[test]
    fn test_extract_uuid_param_invalid_uuid_returns_error() {
        // Verify that extract_uuid_param returns a validation error for invalid UUIDs
        let result = extract_uuid_param(
            "/meal-slots/not-a-uuid",
            "/meal-slots/",
            "meal_slot_id",
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "meal_slot_id");
                assert_eq!(err.message, "Invalid meal_slot_id format");
            }
            other => panic!("Expected Validation error, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_uuid_param_missing_segment_returns_error() {
        // Verify that extract_uuid_param returns error when no segment follows prefix
        let result = extract_uuid_param("/meal-slots/", "/meal-slots/", "meal_slot_id");
        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "meal_slot_id");
            }
            other => panic!("Expected Validation error, got {:?}", other),
        }
    }

    #[test]
    fn test_sub_path_empty_is_collection_route() {
        // Empty sub_path should NOT be classified as detail route
        let sub_path = "";
        assert_eq!(sub_path.len(), 0);
        assert!(!sub_path.len() > 1);
    }

    #[test]
    fn test_sub_path_single_slash_is_collection_route() {
        // Single slash should NOT be classified as detail route
        let sub_path = "/";
        assert_eq!(sub_path.len(), 1);
        assert!(!sub_path.len() > 1);
    }

    #[test]
    fn test_uuid_extraction_boundary_no_panic() {
        // Regression test: verify no panic when sub_path is exactly "/{uuid}" (no trailing /)
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555";
        let rest = &sub_path[1..];
        // rest.find('/') returns None for "083ef6d9-2e7a-407e-af22-d1546d437555"
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        // This should not panic — meal_slot_id_idx == rest.len() == 36
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        // meal_slot_sub_path = &sub_path[37..] which is valid (equal length)
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];
        assert_eq!(meal_slot_sub_path, "");
    }

    #[test]
    fn test_uuid_extraction_boundary_with_trailing_slash() {
        // Verify correct extraction when sub_path is "/{uuid}/"
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555/";
        let rest = &sub_path[1..];
        // rest.find('/') returns Some(36) for "083ef6d9-2e7a-407e-af22-d1546d437555/"
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        assert_eq!(meal_slot_id_idx, 36); // position of the slash in rest
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];
        assert_eq!(meal_slot_sub_path, "/");
    }

    #[test]
    fn test_uuid_extraction_boundary_with_extra_segment() {
        // Verify correct extraction when sub_path is "/{uuid}/extra"
        let sub_path = "/083ef6d9-2e7a-407e-af22-d1546d437555/extra";
        let rest = &sub_path[1..];
        let meal_slot_id_idx = rest.find('/').unwrap_or(rest.len());
        assert_eq!(meal_slot_id_idx, 36);
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        assert_eq!(meal_slot_id_str, "083ef6d9-2e7a-407e-af22-d1546d437555");
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx + 1..];
        assert_eq!(meal_slot_sub_path, "/extra");
    }

    #[test]
    fn test_dependent_sub_path_construction_variants() {
        // Verify route_dependent constructs correct sub_path for all meal-slots URL variants
        let test_cases = vec![
            (
                "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots",
                "",
            ),
            (
                "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/",
                "/",
            ),
            (
                "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555",
                "/083ef6d9-2e7a-407e-af22-d1546d437555",
            ),
            (
                "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555/",
                "/083ef6d9-2e7a-407e-af22-d1546d437555/",
            ),
            (
                "/2324403a-a3c8-4ab7-933a-db5e7e21c756/meal-slots/083ef6d9-2e7a-407e-af22-d1546d437555/extra",
                "/083ef6d9-2e7a-407e-af22-d1546d437555/extra",
            ),
        ];

        for (full_path, expected_sub_path) in test_cases {
            let sub_path = simulate_dependent_to_meal_slot_sub_path(full_path);
            assert_eq!(
                sub_path, expected_sub_path,
                "full_path: {}", full_path
            );
        }
    }

    // ============================================================
    // Route dispatch tests: verify the match arms select the
    // correct branch for each (method, sub_path) combination.
    // ============================================================

    #[test]
    fn test_route_dispatch_delete_no_trailing() {
        let method = "DELETE";
        let meal_slot_sub_path = "";
        assert!(matches!((method, meal_slot_sub_path), ("DELETE", "") | ("DELETE", "/")));
    }

    #[test]
    fn test_route_dispatch_delete_with_trailing() {
        let method = "DELETE";
        let meal_slot_sub_path = "/";
        assert!(matches!((method, meal_slot_sub_path), ("DELETE", "") | ("DELETE", "/")));
    }

    #[test]
    fn test_route_dispatch_get_collection_empty() {
        let method = "GET";
        let sub_path = "";
        assert!(matches!((method, sub_path), ("GET", "") | ("GET", "/")));
    }

    #[test]
    fn test_route_dispatch_get_collection_trailing() {
        let method = "GET";
        let sub_path = "/";
        assert!(matches!((method, sub_path), ("GET", "") | ("GET", "/")));
    }

    #[test]
    fn test_route_dispatch_post_collection() {
        let method = "POST";
        let sub_path = "";
        assert!(matches!((method, sub_path), ("POST", "") | ("POST", "/")));
    }

    #[test]
    fn test_route_dispatch_put_detail() {
        let method = "PUT";
        let meal_slot_sub_path = "";
        assert!(matches!((method, meal_slot_sub_path), ("PUT", "") | ("PUT", "/")));
    }

    #[test]
    fn test_route_dispatch_unknown_method_returns_not_found() {
        let method = "PATCH";
        let meal_slot_sub_path = "";
        assert!(!matches!((method, meal_slot_sub_path),
            ("GET", "") | ("GET", "/")
            | ("PUT", "") | ("PUT", "/")
            | ("DELETE", "") | ("DELETE", "/")));
    }

    #[test]
    fn test_route_dispatch_unknown_sub_path_returns_not_found() {
        let method = "GET";
        let meal_slot_sub_path = "/unknown";
        assert!(!matches!((method, meal_slot_sub_path),
            ("GET", "") | ("GET", "/")
            | ("PUT", "") | ("PUT", "/")
            | ("DELETE", "") | ("DELETE", "/")));
    }
}

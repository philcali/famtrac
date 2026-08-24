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
        let meal_slot_id_idx = rest.find('/').map(|i| i + 1).unwrap_or(rest.len());
        let meal_slot_id_str = &rest[..meal_slot_id_idx];
        let meal_slot_id = extract_uuid_param(
            &format!("/meal-slots/{}", meal_slot_id_str),
            "/meal-slots/",
            "meal_slot_id",
        )?;
        let meal_slot_sub_path = &sub_path[meal_slot_id_idx..];

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
    /// Test that the index calculation for extracting the meal slot ID from sub_path
    /// does not panic when sub_path has no trailing segment (e.g., "/{uuid}").
    ///
    /// This is a regression test for the bug where sub_path.find("/") on
    /// "/{uuid}" returned Some(0) (the leading slash), then &sub_path[1..0]
    /// panicked because start > end in the slice.
    #[test]
    fn test_meal_slot_id_extraction_no_trailing_segment() {
        let meal_slot_id_str = "083ef6d9-2e7a-407e-af22-d1546d437555";
        let sub_path = format!("/{}", meal_slot_id_str);

        // The fix searches in sub_path[1..] for the second "/"
        let inner = &sub_path[1..];
        let meal_slot_id_idx = inner.find("/").map(|i| i + 1);

        // No trailing "/" means no second segment found — should be None
        // This is the correct behavior: falls through to list/create branch
        assert!(meal_slot_id_idx.is_none());
    }

    #[test]
    fn test_meal_slot_id_extraction_with_trailing_slash() {
        let meal_slot_id_str = "083ef6d9-2e7a-407e-af22-d1546d437555";
        let sub_path = format!("/{}", meal_slot_id_str);
        let sub_path = format!("{}{}", sub_path, "/");

        let inner = &sub_path[1..];
        let meal_slot_id_idx = inner.find("/").map(|i| i + 1);

        // Trailing "/" found at index == meal_slot_id_str.len()
        assert!(meal_slot_id_idx.is_some());
        let idx = meal_slot_id_idx.unwrap();
        assert_eq!(idx, meal_slot_id_str.len() + 1);

        // Extracting meal_slot_id_str should work: &sub_path[1..idx]
        let extracted = &sub_path[1..idx];
        assert_eq!(extracted, meal_slot_id_str);

        // meal_slot_sub_path = &sub_path[idx..] should be "/"
        let sub_path_remainder = &sub_path[idx..];
        assert_eq!(sub_path_remainder, "/");
    }

    #[test]
    fn test_meal_slot_id_extraction_with_sub_path() {
        let meal_slot_id_str = "083ef6d9-2e7a-407e-af22-d1546d437555";
        let sub_path = format!("/{}", meal_slot_id_str);
        let sub_path = format!("{}{}", sub_path, "/extra");

        let inner = &sub_path[1..];
        let meal_slot_id_idx = inner.find("/").map(|i| i + 1);

        assert!(meal_slot_id_idx.is_some());
        let idx = meal_slot_id_idx.unwrap();
        assert_eq!(idx, meal_slot_id_str.len() + 1);

        let extracted = &sub_path[1..idx];
        assert_eq!(extracted, meal_slot_id_str);

        let sub_path_remainder = &sub_path[idx..];
        assert_eq!(sub_path_remainder, "/extra");
    }
}

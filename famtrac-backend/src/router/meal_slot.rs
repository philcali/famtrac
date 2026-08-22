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
    // Check if this is a meal-slot detail route: /{meal_slot_id}[/...]
    if let Some(meal_slot_id_idx) = sub_path.find("/") {
        // Extract meal_slot_id from the sub_path
        let meal_slot_id_str = &sub_path[1..meal_slot_id_idx]; // skip leading /
        let meal_slot_id =
            extract_uuid_param(&format!("/meal-slots/{}", meal_slot_id_str), "/meal-slots/", "meal_slot_id")?;
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
        // No sub-path after /meal-slots - list or create
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

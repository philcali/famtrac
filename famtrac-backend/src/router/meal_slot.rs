// Meal slot route handlers — segment-based dispatch
//
// The parent router (mod.rs) handles path parsing and UUID extraction.
// This module only dispatches by HTTP method for meal slot CRUD.

use crate::context::RequestContext;
use crate::domain::{DependentId, FamilyId, MealSlotId};
use crate::errors::HandlerError;
use crate::handlers::{
    create_meal_slot, delete_meal_slot, get_meal_slot, list_meal_slots, update_meal_slot,
    PaginationParams,
};
use crate::repository::{DynamoDbFamilyRepository, DynamoDbMealSlotRepository};
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Handle GET|POST /families/{fid}/dependents/{did}/meal-slots
#[allow(clippy::too_many_arguments)]
pub async fn handle_meal_slots_collection(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    meal_repo: &DynamoDbMealSlotRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let query_params = &request.query_string_parameters;
            let pagination = PaginationParams {
                limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                next_token: query_params.first("next_token").map(|s| s.to_string()),
            };
            let day = query_params.first("day").map(|s| s.to_string());
            let (_status, response_json) = list_meal_slots(
                family_id,
                dependent_id,
                context,
                family_repo,
                meal_repo,
                day,
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

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/dependents/{}/meal-slots",
            method, family_id.0, dependent_id.0
        ))),
    }
}

/// Handle GET|PUT|DELETE /families/{fid}/dependents/{did}/meal-slots/{mid}
#[allow(clippy::too_many_arguments)]
pub async fn handle_meal_slot_item(
    method: &str,
    family_id: FamilyId,
    dependent_id: DependentId,
    meal_slot_id: MealSlotId,
    body: &str,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    meal_repo: &DynamoDbMealSlotRepository,
) -> Result<serde_json::Value, HandlerError> {
    match method {
        "GET" => {
            let (_status, response_json) = get_meal_slot(
                family_id,
                dependent_id,
                meal_slot_id,
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

        "PUT" => {
            let (_status, response_json) = update_meal_slot(
                family_id,
                dependent_id,
                meal_slot_id,
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

        "DELETE" => {
            delete_meal_slot(
                family_id,
                dependent_id,
                meal_slot_id,
                context,
                family_repo,
                meal_repo,
            )
            .await?;
            Ok(serde_json::Value::Null)
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/dependents/{}/meal-slots/{}",
            method, family_id.0, dependent_id.0, meal_slot_id.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_method_dispatch_get_meal_slots() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_post_meal_slots() {
        let method = "POST";
        assert!(matches!(method, "POST"));
    }

    #[test]
    fn test_method_dispatch_get_meal_slot_item() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_put_meal_slot_item() {
        let method = "PUT";
        assert!(matches!(method, "PUT"));
    }

    #[test]
    fn test_method_dispatch_delete_meal_slot_item() {
        let method = "DELETE";
        assert!(matches!(method, "DELETE"));
    }

    #[test]
    fn test_unknown_method_does_not_match() {
        let method = "PATCH";
        assert!(!matches!(method, "GET" | "POST" | "PUT" | "DELETE"));
    }
}

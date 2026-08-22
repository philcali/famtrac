// Dependent route handlers
//
// All dependent routes are nested under /families/{family_id}/dependents
// since dependents are subresources of families.

use crate::context::RequestContext;
use crate::domain::{DependentId, FamilyId};
use crate::errors::HandlerError;
use crate::handlers;
use crate::repository::{
    DynamoDbActivityRepository, DynamoDbDependentRepository, DynamoDbFamilyRepository,
    DynamoDbFeedingLogRepository, DynamoDbMealSlotRepository,
};
use crate::router::extractors::extract_uuid_param;
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Route handler for /families/{family_id}/dependents/* routes
///
/// This function handles routing for dependent-related endpoints nested under a family:
/// - POST /families/{family_id}/dependents - Create a new dependent
/// - GET /families/{family_id}/dependents/{id} - Get a dependent by ID
/// - PUT /families/{family_id}/dependents/{id} - Update a dependent
/// - * /families/{family_id}/dependents/{id}/activities/* - Delegate to activity router
///
/// The list endpoint (GET /families/{family_id}/dependents) is handled by the family router.
#[allow(clippy::too_many_arguments)]
pub async fn route_dependent(
    method: &str,
    family_id: FamilyId,
    sub_path: &str,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
    meal_repo: &DynamoDbMealSlotRepository,
    feeding_log_repo: &DynamoDbFeedingLogRepository,
) -> Result<serde_json::Value, HandlerError> {
    // Check if this is an activity sub-route: /{dependent_id}/activities[/...]
    if let Some(activities_idx) = sub_path.find("/activities") {
        // Extract dependent_id from the sub_path before /activities
        let before_activities = &sub_path[..activities_idx];
        let dependent_id = extract_uuid_param(
            &format!("/dependents{}", before_activities),
            "/dependents/",
            "dependent_id",
        )?;
        let activity_sub_path = &sub_path[activities_idx + "/activities".len()..];
        return super::activity::route_activity(
            method,
            family_id,
            DependentId(dependent_id),
            activity_sub_path,
            body,
            request,
            context,
            family_repo,
            dependent_repo,
            activity_repo,
        )
        .await;
    }

    // Check if this is a meal-slots sub-route: /{dependent_id}/meal-slots[/...]
    if let Some(meal_slots_idx) = sub_path.find("/meal-slots") {
        // Extract dependent_id from the sub_path before /meal-slots
        let before_meal_slots = &sub_path[..meal_slots_idx];
        let dependent_id = extract_uuid_param(
            &format!("/dependents{}", before_meal_slots),
            "/dependents/",
            "dependent_id",
        )?;
        let meal_slots_sub_path = &sub_path[meal_slots_idx + "/meal-slots".len()..];
        return super::meal_slot::route_meal_slot(
            method,
            family_id,
            DependentId(dependent_id),
            meal_slots_sub_path,
            body,
            request,
            context,
            family_repo,
            meal_repo,
        )
        .await;
    }

    // Check if this is a feeding-logs sub-route: /{dependent_id}/feeding-logs[/...]
    if let Some(feeding_logs_idx) = sub_path.find("/feeding-logs") {
        // Extract dependent_id from the sub_path before /feeding-logs
        let before_feeding_logs = &sub_path[..feeding_logs_idx];
        let dependent_id = extract_uuid_param(
            &format!("/dependents{}", before_feeding_logs),
            "/dependents/",
            "dependent_id",
        )?;
        let feeding_logs_sub_path = &sub_path[feeding_logs_idx + "/feeding-logs".len()..];
        return super::feeding_log::route_feeding_log(
            method,
            family_id,
            DependentId(dependent_id),
            feeding_logs_sub_path,
            body,
            request,
            context,
            family_repo,
            feeding_log_repo,
        )
        .await;
    }

    match (method, sub_path) {
        // POST /families/{family_id}/dependents - Create a new dependent
        ("POST", "") | ("POST", "/") => {
            let (_status, response_json) =
                handlers::create_dependent(body, context, family_repo, dependent_repo).await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // GET /families/{family_id}/dependents/{id} - Get a dependent by ID
        ("GET", p) if !p.is_empty() && p != "/" => {
            let dependent_id = extract_uuid_param(
                &format!("/dependents{}", sub_path),
                "/dependents/",
                "dependent_id",
            )?;
            let (_status, response_json) = handlers::get_dependent(
                family_id,
                DependentId(dependent_id),
                context,
                family_repo,
                dependent_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // PUT /families/{family_id}/dependents/{id} - Update a dependent
        ("PUT", p) if !p.is_empty() && p != "/" => {
            let dependent_id = extract_uuid_param(
                &format!("/dependents{}", sub_path),
                "/dependents/",
                "dependent_id",
            )?;
            let (_status, response_json) = handlers::update_dependent(
                family_id,
                DependentId(dependent_id),
                body,
                context,
                family_repo,
                dependent_repo,
            )
            .await?;
            let response: serde_json::Value =
                serde_json::from_str(&response_json).map_err(|e| {
                    HandlerError::InternalError(format!("Failed to parse response: {}", e))
                })?;
            Ok(response)
        }

        // DELETE /families/{family_id}/dependents/{id} - Delete a dependent
        ("DELETE", p) if !p.is_empty() && p != "/" && !p.contains("/activities") => {
            let dependent_id = extract_uuid_param(
                &format!("/dependents{}", sub_path),
                "/dependents/",
                "dependent_id",
            )?;
            handlers::delete_dependent(
                family_id,
                DependentId(dependent_id),
                context,
                family_repo,
                dependent_repo,
            )
            .await?;
            Ok(serde_json::Value::Null)
        }

        // Unknown route
        _ => Err(HandlerError::NotFound(format!(
            "Route not found: {} /families/{}/dependents{}",
            method, family_id.0, sub_path
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_uuid_extraction_for_get_dependent() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/dependents/{}", uuid_str);

        let result = extract_uuid_param(&path, "/dependents/", "dependent_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_invalid_uuid_returns_validation_error() {
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
}

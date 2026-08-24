// Router module — segment-based path matching
//
// All request routing flows through `route_request`. The path is split into
// segments once, then matched declaratively. Sub-routers no longer perform
// their own string parsing or substring searches.

use crate::context::RequestContext;
use crate::domain::{
    ActivityId, DependentId, FamilyId, FeedingLogId, MealSlotId, RecipeId, ShareId,
};
use crate::errors::HandlerError;
use crate::repository::{
    DynamoDbActivityRepository, DynamoDbDependentRepository, DynamoDbFamilyRepository,
    DynamoDbFeedingLogRepository, DynamoDbMealSlotRepository, DynamoDbRecipeRepository,
    DynamoDbShareRepository,
};
use crate::utils::cors::CorsConfig;
use crate::utils::response::HttpResponse;
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

pub mod activity;
pub mod dependent;
pub mod extractors;
pub mod family;
pub mod feeding_log;
pub mod meal_slot;
pub mod recipe;
pub mod share;

use extractors::parse_uuid;

/// Split a path into non-empty segments.
/// e.g. "/families/abc/dependents" → ["families", "abc", "dependents"]
fn split_path(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

/// Main routing function — segment-based dispatch.
#[allow(clippy::too_many_arguments)]
pub async fn route_request(
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
    recipe_repo: &DynamoDbRecipeRepository,
    meal_repo: &DynamoDbMealSlotRepository,
    feeding_log_repo: &DynamoDbFeedingLogRepository,
    share_repo: &DynamoDbShareRepository,
    cors_config: &CorsConfig,
) -> HttpResponse {
    let method = request.request_context.http.method.as_str();
    let path = request.raw_path.as_deref().unwrap_or("/");
    let body = request.body.as_deref().unwrap_or("");

    eprintln!("Routing: {} {}", method, path);

    let segments = split_path(path);

    let result = dispatch(
        method,
        &segments,
        body,
        request,
        context,
        family_repo,
        dependent_repo,
        activity_repo,
        recipe_repo,
        meal_repo,
        feeding_log_repo,
        share_repo,
    )
    .await;

    match result {
        Ok(RouteResponse::Json(value)) => {
            let body_str = serde_json::to_string(&value)
                .unwrap_or_else(|_| r#"{"error":"Failed to serialize response"}"#.to_string());
            HttpResponse::from_handler_result(Ok((200u16, body_str)), cors_config)
        }
        Ok(RouteResponse::StatusAndBody(status, body_str)) => {
            HttpResponse::from_handler_result(Ok((status, body_str)), cors_config)
        }
        Ok(RouteResponse::NoContent) => {
            HttpResponse::from_handler_result(Ok((204u16, String::new())), cors_config)
        }
        Err(e) => HttpResponse::from_handler_result(Err(e), cors_config),
    }
}

/// Internal response type for the dispatch layer.
#[allow(dead_code)]
enum RouteResponse {
    /// A JSON value to serialize with 200 status.
    Json(serde_json::Value),
    /// A pre-serialized response with explicit status.
    StatusAndBody(u16, String),
    /// 204 No Content.
    NoContent,
}

/// Central dispatch — matches path segments to handler functions.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn dispatch(
    method: &str,
    segments: &[&str],
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
    recipe_repo: &DynamoDbRecipeRepository,
    meal_repo: &DynamoDbMealSlotRepository,
    feeding_log_repo: &DynamoDbFeedingLogRepository,
    share_repo: &DynamoDbShareRepository,
) -> Result<RouteResponse, HandlerError> {
    match segments {
        // ===== Top-level shares =====
        // GET /shares
        ["shares"] => {
            let result = share::handle_shares_collection(
                method,
                body,
                request,
                context,
                share_repo,
                family_repo,
            )
            .await?;
            Ok(RouteResponse::StatusAndBody(result.0, result.1))
        }
        // PUT|DELETE /shares/{sid}
        ["shares", sid] => {
            let share_id = ShareId(parse_uuid(sid, "share_id")?);
            let result =
                share::handle_share_item(method, share_id, body, context, share_repo, family_repo)
                    .await?;
            Ok(RouteResponse::StatusAndBody(result.0, result.1))
        }
        // POST /shares/{sid}/accept
        ["shares", sid, "accept"] => {
            let share_id = ShareId(parse_uuid(sid, "share_id")?);
            let result = share::handle_share_accept(method, share_id, context, share_repo).await?;
            Ok(RouteResponse::StatusAndBody(result.0, result.1))
        }

        // ===== Families collection =====
        // GET|POST /families
        ["families"] => {
            let result =
                family::handle_families_collection(method, body, request, context, family_repo)
                    .await?;
            Ok(RouteResponse::Json(result))
        }

        // ===== Family item =====
        // GET|PUT|DELETE /families/{fid}
        ["families", fid] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let result =
                family::handle_family_item(method, family_id, body, context, family_repo).await?;
            Ok(RouteResponse::Json(result))
        }

        // ===== Family shares =====
        // GET|POST /families/{fid}/shares
        ["families", fid, "shares"] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let result = share::handle_family_shares(
                method,
                family_id,
                body,
                request,
                context,
                share_repo,
                family_repo,
            )
            .await?;
            Ok(RouteResponse::StatusAndBody(result.0, result.1))
        }

        // ===== Recipes =====
        // GET|POST /families/{fid}/recipes
        ["families", fid, "recipes"] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let result = recipe::handle_recipes_collection(
                method,
                family_id,
                body,
                request,
                context,
                family_repo,
                recipe_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }
        // GET|PUT|DELETE /families/{fid}/recipes/{rid}
        ["families", fid, "recipes", rid] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let recipe_id = RecipeId(parse_uuid(rid, "recipe_id")?);
            let result = recipe::handle_recipe_item(
                method,
                family_id,
                recipe_id,
                body,
                context,
                family_repo,
                recipe_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }

        // ===== Dependents =====
        // GET|POST /families/{fid}/dependents
        ["families", fid, "dependents"] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let result = dependent::handle_dependents_collection(
                method,
                family_id,
                body,
                request,
                context,
                family_repo,
                dependent_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }
        // GET|PUT|DELETE /families/{fid}/dependents/{did}
        ["families", fid, "dependents", did] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let dependent_id = DependentId(parse_uuid(did, "dependent_id")?);
            let result = dependent::handle_dependent_item(
                method,
                family_id,
                dependent_id,
                body,
                context,
                family_repo,
                dependent_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }

        // ===== Activities =====
        // GET|POST /families/{fid}/dependents/{did}/activities
        ["families", fid, "dependents", did, "activities"] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let dependent_id = DependentId(parse_uuid(did, "dependent_id")?);
            let result = activity::handle_activities_collection(
                method,
                family_id,
                dependent_id,
                body,
                request,
                context,
                family_repo,
                dependent_repo,
                activity_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }
        // GET|PUT|DELETE /families/{fid}/dependents/{did}/activities/{aid}
        ["families", fid, "dependents", did, "activities", aid] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let dependent_id = DependentId(parse_uuid(did, "dependent_id")?);
            let activity_id = ActivityId(parse_uuid(aid, "activity_id")?);
            let result = activity::handle_activity_item(
                method,
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
            Ok(RouteResponse::Json(result))
        }

        // ===== Meal Slots =====
        // GET|POST /families/{fid}/dependents/{did}/meal-slots
        ["families", fid, "dependents", did, "meal-slots"] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let dependent_id = DependentId(parse_uuid(did, "dependent_id")?);
            let result = meal_slot::handle_meal_slots_collection(
                method,
                family_id,
                dependent_id,
                body,
                request,
                context,
                family_repo,
                meal_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }
        // GET|PUT|DELETE /families/{fid}/dependents/{did}/meal-slots/{mid}
        ["families", fid, "dependents", did, "meal-slots", mid] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let dependent_id = DependentId(parse_uuid(did, "dependent_id")?);
            let meal_slot_id = MealSlotId(parse_uuid(mid, "meal_slot_id")?);
            let result = meal_slot::handle_meal_slot_item(
                method,
                family_id,
                dependent_id,
                meal_slot_id,
                body,
                context,
                family_repo,
                meal_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }

        // ===== Feeding Logs =====
        // GET|POST /families/{fid}/dependents/{did}/feeding-logs
        ["families", fid, "dependents", did, "feeding-logs"] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let dependent_id = DependentId(parse_uuid(did, "dependent_id")?);
            let result = feeding_log::handle_feeding_logs_collection(
                method,
                family_id,
                dependent_id,
                body,
                request,
                context,
                family_repo,
                feeding_log_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }
        // GET|PUT|DELETE /families/{fid}/dependents/{did}/feeding-logs/{flid}
        ["families", fid, "dependents", did, "feeding-logs", flid] => {
            let family_id = FamilyId(parse_uuid(fid, "family_id")?);
            let dependent_id = DependentId(parse_uuid(did, "dependent_id")?);
            let feeding_log_id = FeedingLogId(parse_uuid(flid, "feeding_log_id")?);
            let result = feeding_log::handle_feeding_log_item(
                method,
                family_id,
                dependent_id,
                feeding_log_id,
                body,
                context,
                family_repo,
                feeding_log_repo,
            )
            .await?;
            Ok(RouteResponse::Json(result))
        }

        // ===== Not found =====
        _ => Err(HandlerError::NotFound(format!(
            "Route not found: {} {}",
            method,
            segments.join("/")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_path_basic() {
        assert_eq!(split_path("/families"), vec!["families"]);
        assert_eq!(
            split_path("/families/abc-123/dependents"),
            vec!["families", "abc-123", "dependents"]
        );
    }

    #[test]
    fn test_split_path_trailing_slash() {
        assert_eq!(split_path("/families/"), vec!["families"]);
        assert_eq!(
            split_path("/families/abc/recipes/"),
            vec!["families", "abc", "recipes"]
        );
    }

    #[test]
    fn test_split_path_root() {
        let result: Vec<&str> = split_path("/");
        assert!(result.is_empty());
    }

    #[test]
    fn test_segment_matching_families() {
        let segments = split_path("/families");
        assert!(matches!(segments.as_slice(), ["families"]));
    }

    #[test]
    fn test_segment_matching_family_item() {
        let segments = split_path("/families/550e8400-e29b-41d4-a716-446655440000");
        assert!(matches!(segments.as_slice(), ["families", _fid]));
    }

    #[test]
    fn test_segment_matching_recipes() {
        let segments = split_path("/families/abc/recipes");
        assert!(matches!(segments.as_slice(), ["families", _fid, "recipes"]));
    }

    #[test]
    fn test_segment_matching_recipe_item() {
        let segments = split_path("/families/abc/recipes/def");
        assert!(matches!(
            segments.as_slice(),
            ["families", _fid, "recipes", _rid]
        ));
    }

    #[test]
    fn test_segment_matching_meal_slots() {
        let segments = split_path("/families/abc/dependents/def/meal-slots");
        assert!(matches!(
            segments.as_slice(),
            ["families", _, "dependents", _, "meal-slots"]
        ));
    }

    #[test]
    fn test_segment_matching_meal_slot_item() {
        let segments = split_path("/families/abc/dependents/def/meal-slots/ghi");
        assert!(matches!(
            segments.as_slice(),
            ["families", _, "dependents", _, "meal-slots", _]
        ));
    }

    #[test]
    fn test_segment_matching_activities() {
        let segments = split_path("/families/abc/dependents/def/activities");
        assert!(matches!(
            segments.as_slice(),
            ["families", _, "dependents", _, "activities"]
        ));
    }

    #[test]
    fn test_segment_matching_shares_top_level() {
        let segments = split_path("/shares");
        assert!(matches!(segments.as_slice(), ["shares"]));
    }

    #[test]
    fn test_segment_matching_share_accept() {
        let segments = split_path("/shares/abc/accept");
        assert!(matches!(segments.as_slice(), ["shares", _, "accept"]));
    }

    #[test]
    fn test_segment_matching_no_collision() {
        // Verify that /families/{fid} and /families/{fid}/recipes are distinct
        let a = split_path("/families/abc");
        let b = split_path("/families/abc/recipes");
        assert!(matches!(a.as_slice(), ["families", _]));
        assert!(matches!(b.as_slice(), ["families", _, "recipes"]));
    }
}

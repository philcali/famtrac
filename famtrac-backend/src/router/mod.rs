// Router module for handling HTTP request routing
//
// This module contains the main routing logic that was refactored from main.rs.
// It delegates to submodules for different resource types (family, dependent, activity).

use crate::context::RequestContext;
use crate::errors::HandlerError;
use crate::repository::{
    DynamoDbActivityRepository, DynamoDbDependentRepository, DynamoDbFamilyRepository,
    DynamoDbMealSlotRepository, DynamoDbRecipeRepository, DynamoDbShareRepository,
};
use crate::utils::cors::CorsConfig;
use crate::utils::response::HttpResponse;
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

pub mod activity;
pub mod dependent;
pub mod extractors;
pub mod family;
pub mod meal_slot;
pub mod recipe;
pub mod share;

/// Main routing function that dispatches requests to appropriate route handlers
///
/// This function extracts the HTTP method, path, and body from the API Gateway request,
/// logs the routing information, and delegates to the appropriate submodule based on
/// the path prefix.
///
/// # Arguments
///
/// * `request` - API Gateway proxy request containing method, path, body, and query params
/// * `context` - Request context with authentication information
/// * `family_repo` - Family repository for database operations
/// * `dependent_repo` - Dependent repository for database operations
/// * `activity_repo` - Activity repository for database operations
/// * `cors_config` - CORS configuration for response headers
///
/// # Returns
///
/// * `HttpResponse` - HTTP response with status, body, and CORS headers
///
/// # Requirements
///
/// - Requirement 2.1: Export route_request() with same signature as main.rs
/// - Requirement 2.2: Accept ApiGatewayProxyRequest, RequestContext, repositories, and CorsConfig
/// - Requirement 2.3: Return HttpResponse
/// - Requirement 2.4: Match HTTP method and path to delegate to route handlers
/// - Requirement 2.5: Return HandlerError::NotFound for unknown routes
/// - Requirement 7.4: Preserve logging statement "Routing: {method} {path}"
/// - Requirement 7.5: Preserve CORS header handling through HttpResponse::from_handler_result()
pub async fn route_request(
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
    recipe_repo: &DynamoDbRecipeRepository,
    meal_repo: &DynamoDbMealSlotRepository,
    share_repo: &DynamoDbShareRepository,
    cors_config: &CorsConfig,
) -> HttpResponse {
    // Extract HTTP method, path, and body from request
    let method = request.request_context.http.method.as_str();
    let path = request.raw_path.as_deref().unwrap_or("/");
    let body = request.body.as_deref().unwrap_or("");

    // Log routing information (Requirement 7.4)
    eprintln!("Routing: {} {}", method, path);

    // Check for share routes first (top-level /shares paths)
    if path == "/shares" || (path.starts_with("/shares/") && !path.starts_with("/shared-families"))
    {
        let handler_result = share::route_shares(
            method,
            path,
            body,
            request,
            context,
            share_repo,
            family_repo,
        )
        .await;
        return HttpResponse::from_handler_result(handler_result, cors_config);
    }

    // Match on path prefix to delegate to appropriate route handler
    let result = if path.starts_with("/families") {
        // Check if this is a /families/{fid}/shares route
        if let Some(shares_idx) = path.find("/shares") {
            let family_id_str = &path["/families/".len()..shares_idx];
            let family_id = match crate::router::extractors::extract_uuid_param(
                &format!("/families/{}", family_id_str),
                "/families/",
                "family_id",
            ) {
                Ok(id) => id,
                Err(e) => return HttpResponse::from_handler_result(Err(e), cors_config),
            };
            let sub_path = &path[shares_idx + "/shares".len()..];
            let handler_result = share::route_family_shares(
                method,
                crate::domain::FamilyId(family_id),
                sub_path,
                body,
                request,
                context,
                share_repo,
                family_repo,
            )
            .await;
            return HttpResponse::from_handler_result(handler_result, cors_config);
        }

        // Delegate to family route handler
        // Handles /families/*, /families/{id}/dependents/*,
        // /families/{id}/dependents/{id}/activities/*, and /families/{id}/recipes/*
        family::route_family(
            method,
            path,
            body,
            request,
            context,
            family_repo,
            dependent_repo,
            activity_repo,
            recipe_repo,
            meal_repo,
        )
        .await
    } else {
        // Unknown route - return NotFound error (Requirement 2.5)
        Err(HandlerError::NotFound(format!(
            "Route not found: {} {}",
            method, path
        )))
    };

    // Convert JSON result to (status, body) tuple for HttpResponse::from_handler_result
    let handler_result = result.map(|json_value| {
        let body = serde_json::to_string(&json_value)
            .unwrap_or_else(|_| r#"{"error":"Failed to serialize response"}"#.to_string());
        (200u16, body)
    });

    // Convert handler result to HttpResponse with CORS headers (Requirement 7.5)
    HttpResponse::from_handler_result(handler_result, cors_config)
}

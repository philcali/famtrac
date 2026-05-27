use crate::context::RequestContext;
use crate::domain::{ApiTokenResponse, CreateApiTokenRequest, IdentityId, Timestamp};
use crate::errors::HandlerError;
use crate::handlers::{create_api_token, list_api_tokens, revoke_api_token};
use crate::repository::{ApiTokenRepository, DynamoDbApiTokenRepository};
use crate::router::extractors::extract_path_param;
use crate::utils::cors::CorsConfig;
use crate::utils::response::HttpResponse;
use serde_json::json;

/// Route API token management endpoints
pub async fn route_api_tokens(
    method: &str,
    path: &str,
    body: &str,
    context: &RequestContext,
    token_repo: &DynamoDbApiTokenRepository,
    cors_config: &CorsConfig,
) -> HttpResponse {
    // POST /tokens - Create a new API token
    if method == "POST" && path == "/tokens" {
        return handle_create_token(body, context, token_repo, cors_config).await;
    }

    // GET /tokens - List API tokens
    if method == "GET" && path == "/tokens" {
        return handle_list_tokens(context, token_repo, cors_config).await;
    }

    // DELETE /tokens/{token_id} - Revoke an API token
    if method == "DELETE" && path.starts_with("/tokens/") {
        return handle_revoke_token(path, context, token_repo, cors_config).await;
    }

    HttpResponse::from_handler_result(
        Err(HandlerError::NotFound(format!("Route not found: {} {}", method, path))),
        cors_config,
    )
}

async fn handle_create_token(
    body: &str,
    context: &RequestContext,
    token_repo: &DynamoDbApiTokenRepository,
    cors_config: &CorsConfig,
) -> HttpResponse {
    let request: CreateApiTokenRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::from_handler_result(
                Err(HandlerError::Validation(crate::errors::ValidationError {
                    field: "body".to_string(),
                    message: format!("Invalid JSON: {}", e),
                    constraint: None,
                })),
                cors_config,
            );
        }
    };

    let result = create_api_token(context, request, token_repo).await;
    HttpResponse::from_handler_result(
        result.map(|r| {
            let body = serde_json::to_string(&json!({
                "token": r.token,
                "id": r.id,
                "name": r.name,
                "status": r.status,
                "created_at": r.created_at,
                "expires_at": r.expires_at,
            })).unwrap_or_else(|_| "{}".to_string());
            (201u16, body)
        }),
        cors_config,
    )
}

async fn handle_list_tokens(
    context: &RequestContext,
    token_repo: &DynamoDbApiTokenRepository,
    cors_config: &CorsConfig,
) -> HttpResponse {
    let pagination = crate::handlers::PaginationParams {
        next_token: None,
        limit: Some(20),
    };

    let result = list_api_tokens(context, pagination, token_repo).await;
    HttpResponse::from_handler_result(
        result.map(|r| {
            let body = serde_json::to_string(&json!({
                "items": r.items,
                "next_token": r.next_token,
            })).unwrap_or_else(|_| "{}".to_string());
            (200u16, body)
        }),
        cors_config,
    )
}

async fn handle_revoke_token(
    path: &str,
    context: &RequestContext,
    token_repo: &DynamoDbApiTokenRepository,
    cors_config: &CorsConfig,
) -> HttpResponse {
    let token_id = match extract_path_param(path, "/tokens/") {
        Some(id) => id,
        None => return HttpResponse::from_handler_result(Err(HandlerError::Validation(crate::errors::ValidationError {
            field: "token_id".to_string(),
            message: "Invalid token_id format".to_string(),
            constraint: Some("must be a valid token identifier".to_string()),
        })), cors_config),
    };

    let result = revoke_api_token(context, &token_id, token_repo).await;
    HttpResponse::from_handler_result(
        result.map(|_| {
            let body = serde_json::to_string(&json!({ "status": "revoked" })).unwrap_or_else(|_| "{}".to_string());
            (200u16, body)
        }),
        cors_config,
    )
}

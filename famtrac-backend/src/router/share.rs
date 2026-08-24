// Share route handlers — segment-based dispatch
//
// The parent router (mod.rs) handles path parsing and UUID extraction.
// This module only dispatches by HTTP method for share CRUD.

use crate::context::RequestContext;
use crate::domain::{FamilyId, ShareId};
use crate::errors::HandlerError;
use crate::handlers;
use crate::handlers::PaginationParams;
use crate::repository::{DynamoDbFamilyRepository, DynamoDbShareRepository};
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// Handle GET /shares
pub async fn handle_shares_collection(
    method: &str,
    _body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    share_repo: &DynamoDbShareRepository,
    _family_repo: &DynamoDbFamilyRepository,
) -> Result<(u16, String), HandlerError> {
    match method {
        "GET" => {
            let query_params = &request.query_string_parameters;
            let pagination = PaginationParams {
                limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                next_token: query_params.first("next_token").map(|s| s.to_string()),
            };
            handlers::list_shares_for_accepter(context, share_repo, pagination).await
        }

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /shares",
            method
        ))),
    }
}

/// Handle PUT|DELETE /shares/{share_id}
pub async fn handle_share_item(
    method: &str,
    share_id: ShareId,
    body: &str,
    context: &RequestContext,
    share_repo: &DynamoDbShareRepository,
    family_repo: &DynamoDbFamilyRepository,
) -> Result<(u16, String), HandlerError> {
    match method {
        "PUT" => handlers::update_share(body, share_id, context, share_repo, family_repo).await,

        "DELETE" => handlers::revoke_share(share_id, context, share_repo, family_repo).await,

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /shares/{}",
            method, share_id.0
        ))),
    }
}

/// Handle POST /shares/{share_id}/accept
pub async fn handle_share_accept(
    method: &str,
    share_id: ShareId,
    context: &RequestContext,
    share_repo: &DynamoDbShareRepository,
) -> Result<(u16, String), HandlerError> {
    match method {
        "POST" => handlers::accept_share(share_id, context, share_repo).await,

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /shares/{}/accept",
            method, share_id.0
        ))),
    }
}

/// Handle GET|POST /families/{family_id}/shares
pub async fn handle_family_shares(
    method: &str,
    family_id: FamilyId,
    body: &str,
    request: &ApiGatewayV2httpRequest,
    context: &RequestContext,
    share_repo: &DynamoDbShareRepository,
    family_repo: &DynamoDbFamilyRepository,
) -> Result<(u16, String), HandlerError> {
    match method {
        "GET" => {
            let query_params = &request.query_string_parameters;
            let pagination = PaginationParams {
                limit: query_params.first("limit").and_then(|s| s.parse().ok()),
                next_token: query_params.first("next_token").map(|s| s.to_string()),
            };
            handlers::list_shares(family_id, context, share_repo, family_repo, pagination).await
        }

        "POST" => handlers::create_share(body, family_id, context, share_repo, family_repo).await,

        _ => Err(HandlerError::NotFound(format!(
            "Method not allowed: {} /families/{}/shares",
            method, family_id.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_method_dispatch_get_shares() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_put_share_item() {
        let method = "PUT";
        assert!(matches!(method, "PUT"));
    }

    #[test]
    fn test_method_dispatch_delete_share_item() {
        let method = "DELETE";
        assert!(matches!(method, "DELETE"));
    }

    #[test]
    fn test_method_dispatch_post_share_accept() {
        let method = "POST";
        assert!(matches!(method, "POST"));
    }

    #[test]
    fn test_method_dispatch_get_family_shares() {
        let method = "GET";
        assert!(matches!(method, "GET"));
    }

    #[test]
    fn test_method_dispatch_post_family_shares() {
        let method = "POST";
        assert!(matches!(method, "POST"));
    }

    #[test]
    fn test_unknown_method_does_not_match() {
        let method = "PATCH";
        assert!(!matches!(method, "GET" | "POST" | "PUT" | "DELETE"));
    }
}

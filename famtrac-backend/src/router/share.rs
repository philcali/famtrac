// Share route handlers
//
// Handles routing for share-related endpoints:
// - POST /families/{fid}/shares → create_share
// - GET /families/{fid}/shares → list_shares
// - PUT /shares/{sid} → update_share
// - DELETE /shares/{sid} → revoke_share
// - POST /shares/{sid}/accept → accept_share
// - GET /shares → list_shares_for_accepter

use crate::context::RequestContext;
use crate::domain::{FamilyId, ShareId};
use crate::errors::HandlerError;
use crate::handlers;
use crate::repository::{DynamoDbFamilyRepository, DynamoDbShareRepository};
use crate::router::extractors::extract_uuid_param;

/// Route handler for /families/{family_id}/shares routes
///
/// Handles share endpoints nested under a family:
/// - POST /families/{fid}/shares → create_share
/// - GET /families/{fid}/shares → list_shares
///
/// Requirements: 1.1, 5.1
pub async fn route_family_shares(
    method: &str,
    family_id: FamilyId,
    sub_path: &str,
    body: &str,
    context: &RequestContext,
    share_repo: &DynamoDbShareRepository,
    family_repo: &DynamoDbFamilyRepository,
) -> Result<(u16, String), HandlerError> {
    match (method, sub_path) {
        // POST /families/{fid}/shares - Create a new share
        ("POST", "" | "/") => {
            handlers::create_share(body, family_id, context, share_repo, family_repo).await
        }

        // GET /families/{fid}/shares - List shares for a family
        ("GET", "" | "/") => {
            handlers::list_shares(family_id, context, share_repo, family_repo).await
        }

        _ => Err(HandlerError::NotFound(format!(
            "Route not found: {} /families/{}/shares{}",
            method, family_id.0, sub_path
        ))),
    }
}

/// Route handler for /shares/* routes (not nested under a family)
///
/// Handles share endpoints at the top level:
/// - GET /shares → list_shares_for_accepter
/// - PUT /shares/{sid} → update_share
/// - DELETE /shares/{sid} → revoke_share
/// - POST /shares/{sid}/accept → accept_share
///
/// Requirements: 6.1, 7.1, 8.1, 9.1
pub async fn route_shares(
    method: &str,
    path: &str,
    body: &str,
    context: &RequestContext,
    share_repo: &DynamoDbShareRepository,
    family_repo: &DynamoDbFamilyRepository,
) -> Result<(u16, String), HandlerError> {
    match (method, path) {
        // GET /shares - List shares for the accepter
        ("GET", "/shares") => handlers::list_shares_for_accepter(context, share_repo).await,

        // PUT /shares/{sid} - Update a share's permission scope
        ("PUT", p) if p.starts_with("/shares/") && !p.contains("/accept") => {
            let share_id = extract_uuid_param(p, "/shares/", "share_id")?;
            handlers::update_share(body, ShareId(share_id), context, share_repo, family_repo).await
        }

        // DELETE /shares/{sid} - Revoke a share
        ("DELETE", p) if p.starts_with("/shares/") => {
            let share_id = extract_uuid_param(p, "/shares/", "share_id")?;
            handlers::revoke_share(ShareId(share_id), context, share_repo, family_repo).await
        }

        // POST /shares/{sid}/accept - Accept a pending share
        ("POST", p) if p.starts_with("/shares/") && p.ends_with("/accept") => {
            let share_id = extract_uuid_param(p, "/shares/", "share_id")?;
            handlers::accept_share(ShareId(share_id), context, share_repo).await
        }

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

    #[test]
    fn test_share_uuid_extraction() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/shares/{}", uuid_str);
        let result = extract_uuid_param(&path, "/shares/", "share_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_share_uuid_extraction_with_accept_suffix() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/shares/{}/accept", uuid_str);
        let result = extract_uuid_param(&path, "/shares/", "share_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_invalid_share_uuid() {
        let path = "/shares/not-a-uuid";
        let result = extract_uuid_param(path, "/shares/", "share_id");
        assert!(result.is_err());
        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "share_id");
                assert_eq!(err.message, "Invalid share_id format");
                assert_eq!(err.constraint, Some("must be a valid UUID".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_accept_path_pattern() {
        let path = "/shares/550e8400-e29b-41d4-a716-446655440000/accept";
        assert!(path.starts_with("/shares/") && path.ends_with("/accept"));
    }

    #[test]
    fn test_update_path_does_not_match_accept() {
        let path = "/shares/550e8400-e29b-41d4-a716-446655440000";
        assert!(path.starts_with("/shares/") && !path.contains("/accept"));
    }
}

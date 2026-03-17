use crate::context::RequestContext;
use crate::domain::{FamilyId, PermissionScope, Share, ShareId, ShareStatus, Timestamp};
use crate::errors::{HandlerError, ValidationError};
use crate::repository::{FamilyRepository, ShareRepository};
use serde::{Deserialize, Serialize};

/// Default share expiration period: 7 days in seconds
const SHARE_EXPIRATION_SECONDS: i64 = 7 * 24 * 60 * 60;

/// Request body for creating a new share
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateShareRequest {
    pub accepter_email: String,
    pub permission_scope: PermissionScope,
}

/// Request body for updating a share's permission scope
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateShareRequest {
    pub permission_scope: PermissionScope,
}

/// Response body for share operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareResponse {
    pub id: ShareId,
    pub family_id: FamilyId,
    pub requester_id: String,
    pub accepter_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepter_id: Option<String>,
    pub permission_scope: PermissionScope,
    pub status: ShareStatus,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Response body for listing shares
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareListResponse {
    pub shares: Vec<ShareResponse>,
}

impl From<Share> for ShareResponse {
    fn from(share: Share) -> Self {
        ShareResponse {
            id: share.id,
            family_id: share.family_id,
            requester_id: share.requester_id.0,
            accepter_email: share.accepter_email,
            accepter_id: share.accepter_id.map(|id| id.0),
            permission_scope: share.permission_scope,
            status: share.status,
            created_at: share.created_at.to_iso8601(),
            updated_at: share.updated_at.to_iso8601(),
            expires_at: share.expires_at.map(|t| t.to_iso8601()),
        }
    }
}

/// Handler for POST /families/{family_id}/shares
/// Creates a new share for a family with the specified accepter and permission scope
///
/// Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 11.1, 11.2, 11.3
pub async fn create_share<SR: ShareRepository, FR: FamilyRepository>(
    request_body: &str,
    family_id: FamilyId,
    context: &RequestContext,
    share_repo: &SR,
    family_repo: &FR,
) -> Result<(u16, String), HandlerError> {
    // Parse request body (Requirement 11.1, 11.2)
    let request: CreateShareRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate permission scope (Requirement 2.2-2.6)
    request.permission_scope.validate()?;

    // Verify requester owns the family (Requirement 1.2)
    let family = family_repo
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound("Family not found".to_string()))?;

    // Reject self-sharing (Requirement 1.4)
    if let Some(ref email) = context.email {
        if email == &request.accepter_email {
            return Err(HandlerError::Validation(ValidationError {
                field: "accepter_email".to_string(),
                message: "cannot share with yourself".to_string(),
                constraint: Some("accepter_email must differ from requester email".to_string()),
            }));
        }
    }

    // Check for duplicate share (Requirement 1.3)
    let existing = share_repo
        .get_by_family_and_email(family_id, &request.accepter_email)
        .await?;
    if existing.is_some() {
        return Err(HandlerError::Conflict(
            "Share already exists for this family and email".to_string(),
        ));
    }

    // Create Share with Pending status (Requirement 1.1, 1.5)
    let now = Timestamp::now();
    let expires_at =
        Timestamp::from_datetime(now.0 + chrono::Duration::seconds(SHARE_EXPIRATION_SECONDS));
    let share = Share {
        id: ShareId::new(),
        family_id: family.id,
        requester_id: context.identity_id.clone(),
        accepter_email: request.accepter_email,
        accepter_id: None,
        permission_scope: request.permission_scope,
        status: ShareStatus::Pending,
        created_at: now,
        updated_at: now,
        expires_at: Some(expires_at),
    };

    // Persist to repository (Requirement 1.5)
    let created_share = share_repo.create(share).await?;

    // Serialize response (Requirement 11.3)
    let response = ShareResponse::from(created_share);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 201 Created
    Ok((201, response_json))
}

/// Handler for GET /families/{family_id}/shares
/// Lists all shares for a family owned by the requester
///
/// Requirements: 5.1, 5.2, 5.3, 10.3
pub async fn list_shares<SR: ShareRepository, FR: FamilyRepository>(
    family_id: FamilyId,
    context: &RequestContext,
    share_repo: &SR,
    family_repo: &FR,
) -> Result<(u16, String), HandlerError> {
    // Verify requester owns the family (Requirement 5.2)
    let family = family_repo
        .get(context.identity_id.clone(), family_id)
        .await?;
    family.ok_or(HandlerError::NotFound("Family not found".to_string()))?;

    // Query shares by family (Requirement 5.1, 5.3)
    let shares = share_repo
        .list_by_family(context.identity_id.clone(), family_id)
        .await?;

    // Convert to response (Requirement 10.3)
    let shares_response: Vec<ShareResponse> = shares.into_iter().map(ShareResponse::from).collect();
    let response = ShareListResponse {
        shares: shares_response,
    };

    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for PUT /shares/{share_id}
/// Updates the permission scope of an existing share
///
/// Requirements: 6.1, 6.3, 6.4, 6.5
pub async fn update_share<SR: ShareRepository, FR: FamilyRepository>(
    request_body: &str,
    share_id: ShareId,
    context: &RequestContext,
    share_repo: &SR,
    family_repo: &FR,
) -> Result<(u16, String), HandlerError> {
    // Parse request body
    let request: UpdateShareRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate new permission scope (Requirement 6.5)
    request.permission_scope.validate()?;

    // Look up the share by requester (Requirement 6.3, 6.4)
    let share = share_repo
        .get(context.identity_id.clone(), share_id)
        .await?;
    let mut share = share.ok_or(HandlerError::NotFound("Share not found".to_string()))?;

    // Verify requester owns the family (Requirement 6.4)
    let family = family_repo
        .get(context.identity_id.clone(), share.family_id)
        .await?;
    family.ok_or(HandlerError::NotFound("Family not found".to_string()))?;

    // Replace permission scope (Requirement 6.1)
    share.permission_scope = request.permission_scope;
    share.updated_at = Timestamp::now();

    // Persist update
    let updated_share = share_repo.update(share).await?;

    let response = ShareResponse::from(updated_share);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for DELETE /shares/{share_id}
/// Revokes (deletes) a share record
///
/// Requirements: 7.1, 7.3, 7.4
pub async fn revoke_share<SR: ShareRepository, FR: FamilyRepository>(
    share_id: ShareId,
    context: &RequestContext,
    share_repo: &SR,
    family_repo: &FR,
) -> Result<(u16, String), HandlerError> {
    // Look up the share by requester (Requirement 7.3, 7.4)
    let share = share_repo
        .get(context.identity_id.clone(), share_id)
        .await?;
    let share = share.ok_or(HandlerError::NotFound("Share not found".to_string()))?;

    // Verify requester owns the family (Requirement 7.4)
    let family = family_repo
        .get(context.identity_id.clone(), share.family_id)
        .await?;
    family.ok_or(HandlerError::NotFound("Family not found".to_string()))?;

    // Delete the share record (Requirement 7.1)
    share_repo
        .delete(context.identity_id.clone(), share_id)
        .await?;

    // Return 204 No Content
    Ok((204, String::new()))
}

/// Handler for POST /shares/{share_id}/accept
/// Accepts a pending share, transitioning it to active status
///
/// Requirements: 9.1, 9.2, 9.3, 9.4, 10.1, 10.2
pub async fn accept_share<SR: ShareRepository>(
    share_id: ShareId,
    context: &RequestContext,
    share_repo: &SR,
) -> Result<(u16, String), HandlerError> {
    // Get accepter email from context (Requirement 9.1, 9.2)
    let accepter_email = context.email.as_ref().ok_or_else(|| {
        HandlerError::Forbidden("Email not available in authentication context".to_string())
    })?;

    // Look up share by accepter email and share ID (Requirement 9.4)
    let share = share_repo
        .get_by_email_and_share_id(accepter_email, share_id)
        .await?;
    let mut share = share.ok_or(HandlerError::NotFound("Share not found".to_string()))?;

    // Verify accepter email matches (Requirement 9.2)
    if &share.accepter_email != accepter_email {
        return Err(HandlerError::Forbidden("Email does not match".to_string()));
    }

    // Verify share is in Pending status (Requirement 9.3)
    if share.status != ShareStatus::Pending {
        return Err(HandlerError::Validation(ValidationError {
            field: "status".to_string(),
            message: "Share is not in pending status".to_string(),
            constraint: Some("share must be in pending status to accept".to_string()),
        }));
    }

    // Check expiration (Requirement 10.1, 10.2)
    if let Some(expires_at) = share.expires_at {
        let now = Timestamp::now();
        if now > expires_at {
            return Err(HandlerError::Validation(ValidationError {
                field: "expires_at".to_string(),
                message: "Share has expired".to_string(),
                constraint: Some("share must not be expired to accept".to_string()),
            }));
        }
    }

    // Set status to Active and store accepter's IdentityId (Requirement 9.1)
    share.status = ShareStatus::Active;
    share.accepter_id = Some(context.identity_id.clone());
    share.updated_at = Timestamp::now();

    // Persist update
    let updated_share = share_repo.update(share).await?;

    let response = ShareResponse::from(updated_share);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for GET /shared-families
/// Lists all shares where the authenticated user is the accepter
///
/// Requirements: 8.1, 8.2
pub async fn list_shared_families<SR: ShareRepository>(
    context: &RequestContext,
    share_repo: &SR,
) -> Result<(u16, String), HandlerError> {
    // Query shares by accepter identity ID via GSI (Requirement 8.1)
    let shares = share_repo
        .list_by_accepter_id(context.identity_id.clone())
        .await?;

    // Convert to response (Requirement 8.2)
    let shares_response: Vec<ShareResponse> = shares.into_iter().map(ShareResponse::from).collect();
    let response = ShareListResponse {
        shares: shares_response,
    };

    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

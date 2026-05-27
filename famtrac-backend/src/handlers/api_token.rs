use crate::context::RequestContext;
use crate::domain::{ApiTokenResponse, CreateApiTokenRequest, IdentityId, Timestamp};
use crate::errors::HandlerError;
use crate::repository::{ApiTokenRepository, DynamoDbApiTokenRepository};
use uuid::Uuid;

/// Create a new API token for the given user
pub async fn create_api_token(
    context: &RequestContext,
    request: CreateApiTokenRequest,
    token_repo: &DynamoDbApiTokenRepository,
) -> Result<ApiTokenResponse, HandlerError> {
    request.validate().map_err(HandlerError::Validation)?;

    let token_str = generate_token();

    let now = Timestamp::from_datetime(chrono::Utc::now());

    let expires_at = request.expires_in_days.map(|days| {
        Timestamp::from_datetime(now.0 + chrono::Duration::days(i64::from(days)))
    });

    let api_token = crate::domain::ApiToken {
        token: token_str.clone(),
        user_id: context.identity_id.clone(),
        username: context.username.clone(),
        name: request.name.clone(),
        status: crate::domain::ApiTokenStatus::Active,
        created_at: now.clone(),
        updated_at: now.clone(),
        expires_at,
    };

    let saved = token_repo.create(api_token).await.map_err(|e| match e {
        crate::errors::StoreError::ConflictError(_) => {
            HandlerError::Conflict("API token with this value already exists".to_string())
        }
        e => HandlerError::Store(e),
    })?;

    Ok(ApiTokenResponse {
        token: Some(saved.token.clone()),
        id: saved.token,
        name: saved.name,
        status: saved.status,
        created_at: saved.created_at,
        expires_at: saved.expires_at,
    })
}

/// List all API tokens for the given user
pub async fn list_api_tokens(
    context: &RequestContext,
    pagination: crate::handlers::PaginationParams,
    token_repo: &DynamoDbApiTokenRepository,
) -> Result<crate::handlers::PaginatedResponse<ApiTokenResponse>, HandlerError> {
    let tokens = token_repo
        .list_by_user(context.identity_id.clone(), pagination)
        .await
        .map_err(HandlerError::Store)?;

    let responses = tokens
        .items
        .into_iter()
        .map(|t| ApiTokenResponse {
            token: None,
            id: t.token,
            name: t.name,
            status: t.status,
            created_at: t.created_at,
            expires_at: t.expires_at,
        })
        .collect();

    Ok(crate::handlers::PaginatedResponse {
        items: responses,
        next_token: tokens.next_token,
        total_count: None,
    })
}

/// Revoke an API token
pub async fn revoke_api_token(
    context: &RequestContext,
    token_str: &str,
    token_repo: &DynamoDbApiTokenRepository,
) -> Result<(), HandlerError> {
    token_repo
        .revoke(context.identity_id.clone(), token_str)
        .await
        .map_err(HandlerError::Store)?;

    Ok(())
}

fn generate_token() -> String {
    let uuid = Uuid::new_v4();
    format!("fam_{}", uuid)
}

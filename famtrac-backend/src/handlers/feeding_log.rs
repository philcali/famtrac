use super::permission::check_permission;
use crate::context::RequestContext;
use crate::domain::{
    DependentId, FamilyId, FeedingLog, FeedingLogId, PermissionAction, RecipeId, Timestamp,
};
use crate::errors::HandlerError;
use crate::handlers::pagination::PaginationParams;
use crate::repository::{FamilyRepository, FeedingLogRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request body for creating a new feeding log
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFeedingLogRequest {
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub date: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reaction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Request body for updating a feeding log
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFeedingLogRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reaction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Response body for feeding log operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedingLogResponse {
    pub id: FeedingLogId,
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub date: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reaction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Response body for listing feeding logs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedingLogListResponse {
    pub feeding_logs: Vec<FeedingLogResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

impl From<FeedingLog> for FeedingLogResponse {
    fn from(feeding_log: FeedingLog) -> Self {
        FeedingLogResponse {
            id: feeding_log.id,
            family_id: feeding_log.family_id,
            dependent_id: feeding_log.dependent_id,
            date: feeding_log.date,
            time: feeding_log.time,
            recipe_id: feeding_log.recipe_id.map(|r| r.0.to_string()),
            amount: feeding_log.amount,
            reaction: feeding_log.reaction,
            notes: feeding_log.notes,
            created_at: feeding_log.created_at.to_iso8601(),
            updated_at: feeding_log.updated_at.to_iso8601(),
        }
    }
}

/// Handler for POST /families/{family_id}/dependents/{dependent_id}/feeding-logs
pub async fn create_feeding_log<F: FamilyRepository, FL: FeedingLogRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    feeding_log_repository: &FL,
) -> Result<(u16, String), HandlerError> {
    let request: CreateFeedingLogRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate date format (YYYY-MM-DD)
    if request.date.len() != 10
        || request.date.chars().nth(4) != Some('-')
        || request.date.chars().nth(7) != Some('-')
        || !request.date[..4].chars().all(|c| c.is_ascii_digit())
        || !request.date[5..7].chars().all(|c| c.is_ascii_digit())
        || !request.date[8..].chars().all(|c| c.is_ascii_digit())
    {
        return Err(HandlerError::Validation(crate::errors::ValidationError {
            field: "date".to_string(),
            message: "Date must be in YYYY-MM-DD format".to_string(),
            constraint: Some("must be a valid date string".to_string()),
        }));
    }

    // Validate time format (HH:MM)
    if request.time.len() != 5 || request.time.chars().filter(|c| *c == ':').count() != 1 {
        return Err(HandlerError::Validation(crate::errors::ValidationError {
            field: "time".to_string(),
            message: "Time must be in HH:MM format".to_string(),
            constraint: Some("must be a valid time string".to_string()),
        }));
    }

    // Verify Family ownership
    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Enforce permission
    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    // Create feeding log
    let now = Timestamp::now();
    let recipe_id = request
        .recipe_id
        .and_then(|s| Uuid::parse_str(&s).ok().map(RecipeId));
    let feeding_log = FeedingLog {
        id: FeedingLogId::new(),
        family_id,
        dependent_id,
        date: request.date,
        time: request.time,
        recipe_id,
        amount: request.amount,
        reaction: request.reaction,
        notes: request.notes,
        created_at: now,
        updated_at: now,
        share_id: None,
        permission_scope: None,
    };

    let created_feeding_log = feeding_log_repository.create(feeding_log).await?;
    let response = FeedingLogResponse::from(created_feeding_log);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((201, response_json))
}

/// Handler for GET /families/{family_id}/dependents/{dependent_id}/feeding-logs/{feeding_log_id}
pub async fn get_feeding_log<F: FamilyRepository, FL: FeedingLogRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    feeding_log_id: FeedingLogId,
    context: &RequestContext,
    family_repository: &F,
    feeding_log_repository: &FL,
) -> Result<(u16, String), HandlerError> {
    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentRead,
    )?;

    let feeding_log = feeding_log_repository
        .get(family_id, dependent_id, feeding_log_id)
        .await?;
    let feeding_log = feeding_log.ok_or(HandlerError::NotFound(format!(
        "Feeding log with id {:?} not found",
        feeding_log_id
    )))?;

    let response = FeedingLogResponse::from(feeding_log);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for PUT /families/{family_id}/dependents/{dependent_id}/feeding-logs/{feeding_log_id}
pub async fn update_feeding_log<F: FamilyRepository, FL: FeedingLogRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    feeding_log_id: FeedingLogId,
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    feeding_log_repository: &FL,
) -> Result<(u16, String), HandlerError> {
    let request: UpdateFeedingLogRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    let feeding_log = feeding_log_repository
        .get(family_id, dependent_id, feeding_log_id)
        .await?;
    let mut feeding_log = feeding_log.ok_or(HandlerError::NotFound(format!(
        "Feeding log with id {:?} not found",
        feeding_log_id
    )))?;

    if let Some(date) = &request.date {
        if date.len() != 10
            || date.chars().nth(4) != Some('-')
            || date.chars().nth(7) != Some('-')
            || !date[..4].chars().all(|c| c.is_ascii_digit())
            || !date[5..7].chars().all(|c| c.is_ascii_digit())
            || !date[8..].chars().all(|c| c.is_ascii_digit())
        {
            return Err(HandlerError::Validation(crate::errors::ValidationError {
                field: "date".to_string(),
                message: "Date must be in YYYY-MM-DD format".to_string(),
                constraint: Some("must be a valid date string".to_string()),
            }));
        }
        feeding_log.date = date.clone();
    }
    if let Some(time) = &request.time {
        if time.len() != 5 || time.chars().filter(|c| *c == ':').count() != 1 {
            return Err(HandlerError::Validation(crate::errors::ValidationError {
                field: "time".to_string(),
                message: "Time must be in HH:MM format".to_string(),
                constraint: Some("must be a valid time string".to_string()),
            }));
        }
        feeding_log.time = time.clone();
    }
    if let Some(recipe_id_str) = &request.recipe_id {
        feeding_log.recipe_id = Uuid::parse_str(recipe_id_str).ok().map(RecipeId);
    }
    if let Some(amount) = &request.amount {
        feeding_log.amount = *amount;
    }
    if let Some(reaction) = &request.reaction {
        feeding_log.reaction = Some(reaction.clone());
    }
    if let Some(notes) = &request.notes {
        feeding_log.notes = Some(notes.clone());
    }
    feeding_log.updated_at = Timestamp::now();

    let updated_feeding_log = feeding_log_repository.update(feeding_log).await?;
    let response = FeedingLogResponse::from(updated_feeding_log);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for DELETE /families/{family_id}/dependents/{dependent_id}/feeding-logs/{feeding_log_id}
pub async fn delete_feeding_log<F: FamilyRepository, FL: FeedingLogRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    feeding_log_id: FeedingLogId,
    context: &RequestContext,
    family_repository: &F,
    feeding_log_repository: &FL,
) -> Result<(u16, String), HandlerError> {
    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    feeding_log_repository
        .delete(family_id, dependent_id, feeding_log_id)
        .await?;

    Ok((204, String::new()))
}

/// Handler for GET /families/{family_id}/dependents/{dependent_id}/feeding-logs
pub async fn list_feeding_logs<F: FamilyRepository, FL: FeedingLogRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    context: &RequestContext,
    family_repository: &F,
    feeding_log_repository: &FL,
    date: Option<String>,
    pagination: PaginationParams,
) -> Result<(u16, String), HandlerError> {
    let family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentRead,
    )?;

    let paginated_result = feeding_log_repository
        .list_by_dependent(family_id, dependent_id, date, pagination)
        .await?;

    let feeding_logs_response: Vec<FeedingLogResponse> = paginated_result
        .items
        .into_iter()
        .map(FeedingLogResponse::from)
        .collect();

    let response = FeedingLogListResponse {
        feeding_logs: feeding_logs_response,
        next_token: paginated_result.next_token,
    };

    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Family, IdentityId};
    use crate::test_utils::mocks::{MockFamilyRepository, MockFeedingLogRepository};

    fn create_test_context(identity_id: &str) -> RequestContext {
        RequestContext {
            identity_id: IdentityId::new(identity_id.to_string()),
            username: None,
        }
    }

    fn create_test_family(family_id: FamilyId, owner_id: &str) -> Family {
        Family {
            id: family_id,
            name: "Test Family".to_string(),
            owner_id: IdentityId::new(owner_id.to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        }
    }

    #[tokio::test]
    async fn test_create_feeding_log_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "dependent_id": "{}", "date": "2025-01-15", "time": "12:00", "amount": 150.5}}"#,
            family_id.0, dependent_id.0
        );

        let result = create_feeding_log(
            family_id,
            dependent_id,
            &request_body,
            &context,
            &family_repo,
            &feeding_log_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 201);

        let response: FeedingLogResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.date, "2025-01-15");
        assert_eq!(response.time, "12:00");
        assert!((response.amount - 150.5).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_create_feeding_log_invalid_json() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let request_body = r#"{"date": invalid}"#;
        let result = create_feeding_log(
            family_id,
            dependent_id,
            request_body,
            &context,
            &family_repo,
            &feeding_log_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "body");
                assert!(err.message.contains("Invalid JSON"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_feeding_log_invalid_date_format() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "dependent_id": "{}", "date": "01-15-2025", "time": "12:00", "amount": 100}}"#,
            family_id.0, dependent_id.0
        );

        let result = create_feeding_log(
            family_id,
            dependent_id,
            &request_body,
            &context,
            &family_repo,
            &feeding_log_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "date");
                assert!(err.message.contains("YYYY-MM-DD"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_feeding_log_invalid_time_format() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "dependent_id": "{}", "date": "2025-01-15", "time": "1200", "amount": 100}}"#,
            family_id.0, dependent_id.0
        );

        let result = create_feeding_log(
            family_id,
            dependent_id,
            &request_body,
            &context,
            &family_repo,
            &feeding_log_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "time");
                assert!(err.message.contains("HH:MM"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_feeding_log_family_not_found() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let request_body = format!(
            r#"{{"family_id": "{}", "dependent_id": "{}", "date": "2025-01-15", "time": "12:00", "amount": 100}}"#,
            family_id.0, dependent_id.0
        );

        let result = create_feeding_log(
            family_id,
            dependent_id,
            &request_body,
            &context,
            &family_repo,
            &feeding_log_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_get_feeding_log_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let feeding_log_id = FeedingLogId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let feeding_log = FeedingLog {
            id: feeding_log_id,
            family_id,
            dependent_id,
            date: "2025-01-15".to_string(),
            time: "12:00".to_string(),
            recipe_id: None,
            amount: 150.5,
            reaction: Some("Good".to_string()),
            notes: Some("Lunch".to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        feeding_log_repo
            .feeding_logs
            .lock()
            .unwrap()
            .insert((family_id, dependent_id, feeding_log_id), feeding_log);

        let result = get_feeding_log(
            family_id,
            dependent_id,
            feeding_log_id,
            &context,
            &family_repo,
            &feeding_log_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: FeedingLogResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.date, "2025-01-15");
        assert_eq!(response.reaction, Some("Good".to_string()));
    }

    #[tokio::test]
    async fn test_get_feeding_log_not_found() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let feeding_log_id = FeedingLogId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let result = get_feeding_log(
            family_id,
            dependent_id,
            feeding_log_id,
            &context,
            &family_repo,
            &feeding_log_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_list_feeding_logs_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let feeding_log1 = FeedingLog {
            id: FeedingLogId::new(),
            family_id,
            dependent_id,
            date: "2025-01-15".to_string(),
            time: "08:00".to_string(),
            recipe_id: None,
            amount: 120.0,
            reaction: Some("Good".to_string()),
            notes: Some("Breakfast".to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        let feeding_log2 = FeedingLog {
            id: FeedingLogId::new(),
            family_id,
            dependent_id,
            date: "2025-01-15".to_string(),
            time: "12:00".to_string(),
            recipe_id: None,
            amount: 150.5,
            reaction: Some("Fine".to_string()),
            notes: Some("Lunch".to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        feeding_log_repo
            .feeding_logs
            .lock()
            .unwrap()
            .insert((family_id, dependent_id, feeding_log1.id), feeding_log1);
        feeding_log_repo
            .feeding_logs
            .lock()
            .unwrap()
            .insert((family_id, dependent_id, feeding_log2.id), feeding_log2);

        let pagination = PaginationParams {
            limit: None,
            next_token: None,
        };
        let result = list_feeding_logs(
            family_id,
            dependent_id,
            &context,
            &family_repo,
            &feeding_log_repo,
            None,
            pagination,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: FeedingLogListResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.feeding_logs.len(), 2);
        assert!(response.next_token.is_none());
    }

    #[tokio::test]
    async fn test_delete_feeding_log_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let feeding_log_id = FeedingLogId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let feeding_log_repo = MockFeedingLogRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let feeding_log = FeedingLog {
            id: feeding_log_id,
            family_id,
            dependent_id,
            date: "2025-01-15".to_string(),
            time: "12:00".to_string(),
            recipe_id: None,
            amount: 150.5,
            reaction: Some("Good".to_string()),
            notes: Some("Lunch".to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        feeding_log_repo
            .feeding_logs
            .lock()
            .unwrap()
            .insert((family_id, dependent_id, feeding_log_id), feeding_log);

        let result = delete_feeding_log(
            family_id,
            dependent_id,
            feeding_log_id,
            &context,
            &family_repo,
            &feeding_log_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, _) = result.unwrap();
        assert_eq!(status, 204);
    }
}

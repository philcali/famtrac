use super::permission::check_permission;
use crate::context::RequestContext;
use crate::domain::{Date, Dependent, DependentId, FamilyId, PermissionAction, Timestamp};
use crate::errors::{validate_date_of_birth, validate_dependent_name, HandlerError};
use crate::handlers::pagination::PaginationParams;
use crate::repository::{DependentRepository, FamilyRepository};
use serde::{Deserialize, Serialize};

/// Request body for creating a new dependent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateDependentRequest {
    pub family_id: FamilyId,
    pub name: String,
    pub date_of_birth: Date,
}

/// Request body for updating a dependent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateDependentRequest {
    pub name: String,
    pub date_of_birth: Date,
}

/// Response body for dependent operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependentResponse {
    pub id: DependentId,
    pub family_id: FamilyId,
    pub name: String,
    pub date_of_birth: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Response body for listing dependents
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependentListResponse {
    pub dependents: Vec<DependentResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

impl From<Dependent> for DependentResponse {
    fn from(dependent: Dependent) -> Self {
        DependentResponse {
            id: dependent.id,
            family_id: dependent.family_id,
            name: dependent.name,
            date_of_birth: dependent.date_of_birth.0.format("%Y-%m-%d").to_string(),
            created_at: dependent.created_at.to_iso8601(),
            updated_at: dependent.updated_at.to_iso8601(),
        }
    }
}

/// Handler for POST /dependents
/// Creates a new dependent with authorization check on parent family
///
/// Requirements:
/// - 2.1: Create a new Dependent with a unique identifier, name, date of birth, and associated Family identifier
/// - 2.2: Return descriptive error messages for invalid data
/// - 2.6: Verify the Identity has access to the associated Family
/// - 8.1: Return 400 Bad Request for malformed JSON
/// - 10.1: Parse incoming JSON request bodies into strongly-typed Rust structures
pub async fn create_dependent<F: FamilyRepository, D: DependentRepository>(
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    dependent_repository: &D,
) -> Result<(u16, String), HandlerError> {
    // Parse request body (Requirement 8.1, 10.1)
    let request: CreateDependentRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate dependent name (Requirement 2.2)
    validate_dependent_name(&request.name)?;

    // Validate date of birth (Requirement 2.2)
    validate_date_of_birth(&request.date_of_birth)?;

    // Retrieve parent Family with owner-scoped key (implicit authorization) (Requirement 2.6, 4.3)
    let _family = family_repository
        .get(context.identity_id.clone(), request.family_id)
        .await?;
    let _family = _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        request.family_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.2, 4.5)
    check_permission(
        _family.share_id.as_ref(),
        _family.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    // Create dependent (Requirement 2.1)
    let now = Timestamp::now();
    let dependent = Dependent {
        id: DependentId::new(),
        family_id: request.family_id,
        name: request.name,
        date_of_birth: request.date_of_birth,
        created_at: now,
        updated_at: now,
        share_id: None,
        permission_scope: None,
    };

    // Persist to repository
    let created_dependent = dependent_repository.create(dependent).await?;

    // Convert to response and serialize
    let response = DependentResponse::from(created_dependent);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 201 Created
    Ok((201, response_json))
}

/// Handler for GET /dependents/{dependent_id}
/// Retrieves a dependent by ID with authorization check
///
/// Requirements:
/// - 2.3: Retrieve a Dependent by its unique identifier
/// - 2.6: Verify the Identity has access to the associated Family
/// - 10.3: Serialize response data into valid JSON format
pub async fn get_dependent<F: FamilyRepository, D: DependentRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    context: &RequestContext,
    family_repository: &F,
    dependent_repository: &D,
) -> Result<(u16, String), HandlerError> {
    // Verify Family ownership with owner-scoped key (implicit authorization) (Requirement 2.6, 4.4)
    let _family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Retrieve dependent from repository (Requirement 2.3)
    let dependent = dependent_repository.get(family_id, dependent_id).await?;

    // Return 404 if dependent doesn't exist
    let dependent = dependent.ok_or(HandlerError::NotFound(format!(
        "Dependent with id {:?} not found",
        dependent_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.1, 4.5)
    check_permission(
        dependent.share_id.as_ref(),
        dependent.permission_scope.as_ref(),
        PermissionAction::DependentRead,
    )?;

    // Convert to response and serialize (Requirement 10.3)
    let response = DependentResponse::from(dependent);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

/// Handler for PUT /dependents/{dependent_id}
/// Updates a dependent with authorization check
///
/// Requirements:
/// - 2.4: Update Dependent information
/// - 2.6: Verify the Identity has access to the associated Family
/// - 8.1: Return 400 Bad Request for malformed JSON
/// - 10.1: Parse incoming JSON request bodies into strongly-typed Rust structures
pub async fn update_dependent<F: FamilyRepository, D: DependentRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    dependent_repository: &D,
) -> Result<(u16, String), HandlerError> {
    // Parse request body (Requirement 8.1, 10.1)
    let request: UpdateDependentRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate dependent name
    validate_dependent_name(&request.name)?;

    // Validate date of birth
    validate_date_of_birth(&request.date_of_birth)?;

    // Verify Family ownership with owner-scoped key (implicit authorization) (Requirement 2.6, 4.4)
    let _family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Retrieve existing dependent (Requirement 2.4)
    let dependent = dependent_repository.get(family_id, dependent_id).await?;

    // Return 404 if dependent doesn't exist
    let mut dependent = dependent.ok_or(HandlerError::NotFound(format!(
        "Dependent with id {:?} not found",
        dependent_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.2, 4.5)
    check_permission(
        dependent.share_id.as_ref(),
        dependent.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    // Update dependent data
    dependent.name = request.name;
    dependent.date_of_birth = request.date_of_birth;
    dependent.updated_at = Timestamp::now();

    // Persist to repository
    let updated_dependent = dependent_repository.update(dependent).await?;

    // Convert to response and serialize
    let response = DependentResponse::from(updated_dependent);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

/// Handler for DELETE /families/{family_id}/dependents/{dependent_id}
/// Deletes a dependent with authorization check
pub async fn delete_dependent<F: FamilyRepository, D: DependentRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    context: &RequestContext,
    family_repository: &F,
    dependent_repository: &D,
) -> Result<(u16, String), HandlerError> {
    // Verify Family ownership with owner-scoped key (implicit authorization)
    let _family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Retrieve dependent
    let dependent = dependent_repository.get(family_id, dependent_id).await?;

    // Return 404 if dependent doesn't exist
    let _dependent = dependent.ok_or(HandlerError::NotFound(format!(
        "Dependent with id {:?} not found",
        dependent_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.2, 4.5)
    check_permission(
        _dependent.share_id.as_ref(),
        _dependent.permission_scope.as_ref(),
        PermissionAction::DependentWrite,
    )?;

    // Delete from repository
    dependent_repository.delete(family_id, dependent_id).await?;

    // Return 204 No Content
    Ok((204, String::new()))
}

/// Handler for GET /families/{family_id}/dependents
/// Lists all dependents for a family with authorization check
///
/// Requirements:
/// - 2.5: List all Dependents associated with a specific Family
/// - 2.6: Verify the Identity has access to the associated Family
/// - 10.3: Serialize response data into valid JSON format
pub async fn list_dependents<F: FamilyRepository, D: DependentRepository>(
    family_id: FamilyId,
    context: &RequestContext,
    family_repository: &F,
    dependent_repository: &D,
    pagination: PaginationParams,
) -> Result<(u16, String), HandlerError> {
    // Retrieve Family with owner-scoped key (implicit authorization) (Requirement 2.6, 4.5)
    let _family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let family = _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.1, 4.5)
    check_permission(
        family.share_id.as_ref(),
        family.permission_scope.as_ref(),
        PermissionAction::DependentRead,
    )?;

    // List Dependents for Family with pagination (Requirement 2.5)
    let paginated_result = dependent_repository
        .list_by_family(family_id, pagination)
        .await?;

    // Convert to response and wrap in list response structure (Requirement 10.3)
    let dependents_response: Vec<DependentResponse> = paginated_result
        .items
        .into_iter()
        .map(DependentResponse::from)
        .collect();

    let response = DependentListResponse {
        dependents: dependents_response,
        next_token: paginated_result.next_token,
    };

    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Family, IdentityId};
    use crate::test_utils::mocks::{MockDependentRepository, MockFamilyRepository};

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
    async fn test_create_dependent_success() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "name": "Baby Alice", "date_of_birth": "2024-01-15"}}"#,
            family_id.0
        );

        let result = create_dependent(&request_body, &context, &family_repo, &dependent_repo).await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 201);

        let response: DependentResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.name, "Baby Alice");
        assert_eq!(response.family_id, family_id);
    }

    #[tokio::test]
    async fn test_create_dependent_invalid_json() {
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let request_body = r#"{"name": invalid}"#;
        let result = create_dependent(request_body, &context, &family_repo, &dependent_repo).await;

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
    async fn test_create_dependent_empty_name() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "name": "", "date_of_birth": "2024-01-15"}}"#,
            family_id.0
        );

        let result = create_dependent(&request_body, &context, &family_repo, &dependent_repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "name");
                assert!(err.message.contains("empty"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_dependent_future_date_of_birth() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let future_date = (chrono::Utc::now() + chrono::Duration::days(30))
            .date_naive()
            .format("%Y-%m-%d");
        let request_body = format!(
            r#"{{"family_id": "{}", "name": "Baby Alice", "date_of_birth": "{}"}}"#,
            family_id.0, future_date
        );

        let result = create_dependent(&request_body, &context, &family_repo, &dependent_repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "date_of_birth");
                assert!(err.message.contains("future"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_dependent_family_not_found() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let request_body = format!(
            r#"{{"family_id": "{}", "name": "Baby Alice", "date_of_birth": "2024-01-15"}}"#,
            family_id.0
        );

        let result = create_dependent(&request_body, &context, &family_repo, &dependent_repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_create_dependent_unauthorized() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-456"); // Different user
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "name": "Baby Alice", "date_of_birth": "2024-01-15"}}"#,
            family_id.0
        );

        let result = create_dependent(&request_body, &context, &family_repo, &dependent_repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(_) => {}
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_get_dependent_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        // Setup dependent
        let dependent = Dependent {
            id: dependent_id,
            family_id,
            name: "Baby Alice".to_string(),
            date_of_birth: Date::from_naive_date(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        dependent_repo
            .dependents
            .lock()
            .unwrap()
            .insert(dependent_id, dependent);

        let result = get_dependent(
            family_id,
            dependent_id,
            &context,
            &family_repo,
            &dependent_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: DependentResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.name, "Baby Alice");
        assert_eq!(response.family_id, family_id);
    }

    #[tokio::test]
    async fn test_get_dependent_not_found() {
        let dependent_id = DependentId::new();
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let result = get_dependent(
            family_id,
            dependent_id,
            &context,
            &family_repo,
            &dependent_repo,
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
    async fn test_get_dependent_unauthorized() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-456"); // Different user
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        // Setup dependent
        let dependent = Dependent {
            id: dependent_id,
            family_id,
            name: "Baby Alice".to_string(),
            date_of_birth: Date::from_naive_date(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        dependent_repo
            .dependents
            .lock()
            .unwrap()
            .insert(dependent_id, dependent);

        let result = get_dependent(
            family_id,
            dependent_id,
            &context,
            &family_repo,
            &dependent_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(_) => {}
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_update_dependent_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        // Setup dependent
        let dependent = Dependent {
            id: dependent_id,
            family_id,
            name: "Baby Alice".to_string(),
            date_of_birth: Date::from_naive_date(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        dependent_repo
            .dependents
            .lock()
            .unwrap()
            .insert(dependent_id, dependent);

        let request_body = r#"{"name": "Alice Updated", "date_of_birth": "2024-01-20"}"#;
        let result = update_dependent(
            family_id,
            dependent_id,
            request_body,
            &context,
            &family_repo,
            &dependent_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: DependentResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.name, "Alice Updated");
    }

    #[tokio::test]
    async fn test_update_dependent_not_found() {
        let dependent_id = DependentId::new();
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let request_body = r#"{"name": "Alice Updated", "date_of_birth": "2024-01-20"}"#;
        let result = update_dependent(
            family_id,
            dependent_id,
            request_body,
            &context,
            &family_repo,
            &dependent_repo,
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
    async fn test_update_dependent_unauthorized() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-456"); // Different user
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        // Setup dependent
        let dependent = Dependent {
            id: dependent_id,
            family_id,
            name: "Baby Alice".to_string(),
            date_of_birth: Date::from_naive_date(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        dependent_repo
            .dependents
            .lock()
            .unwrap()
            .insert(dependent_id, dependent);

        let request_body = r#"{"name": "Alice Updated", "date_of_birth": "2024-01-20"}"#;
        let result = update_dependent(
            family_id,
            dependent_id,
            request_body,
            &context,
            &family_repo,
            &dependent_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(_) => {}
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_update_dependent_invalid_json() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        // Setup dependent
        let dependent = Dependent {
            id: dependent_id,
            family_id,
            name: "Baby Alice".to_string(),
            date_of_birth: Date::from_naive_date(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        dependent_repo
            .dependents
            .lock()
            .unwrap()
            .insert(dependent_id, dependent);

        let request_body = r#"{"name": invalid}"#;
        let result = update_dependent(
            family_id,
            dependent_id,
            request_body,
            &context,
            &family_repo,
            &dependent_repo,
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
    async fn test_list_dependents_success() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        // Setup dependents
        let dependent1 = Dependent {
            id: DependentId::new(),
            family_id,
            name: "Baby Alice".to_string(),
            date_of_birth: Date::from_naive_date(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        let dependent2 = Dependent {
            id: DependentId::new(),
            family_id,
            name: "Baby Bob".to_string(),
            date_of_birth: Date::from_naive_date(
                chrono::NaiveDate::from_ymd_opt(2023, 6, 10).unwrap(),
            ),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        dependent_repo
            .dependents
            .lock()
            .unwrap()
            .insert(dependent1.id, dependent1);
        dependent_repo
            .dependents
            .lock()
            .unwrap()
            .insert(dependent2.id, dependent2);

        let pagination = PaginationParams {
            limit: None,
            next_token: None,
        };
        let result = list_dependents(
            family_id,
            &context,
            &family_repo,
            &dependent_repo,
            pagination,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: DependentListResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.dependents.len(), 2);
        assert!(response.next_token.is_none());
    }

    #[tokio::test]
    async fn test_list_dependents_empty() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let pagination = PaginationParams {
            limit: None,
            next_token: None,
        };
        let result = list_dependents(
            family_id,
            &context,
            &family_repo,
            &dependent_repo,
            pagination,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: DependentListResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.dependents.len(), 0);
        assert!(response.next_token.is_none());
    }

    #[tokio::test]
    async fn test_list_dependents_family_not_found() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let pagination = PaginationParams {
            limit: None,
            next_token: None,
        };
        let result = list_dependents(
            family_id,
            &context,
            &family_repo,
            &dependent_repo,
            pagination,
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
    async fn test_list_dependents_unauthorized() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-456"); // Different user
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        // Setup family
        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let pagination = PaginationParams {
            limit: None,
            next_token: None,
        };
        let result = list_dependents(
            family_id,
            &context,
            &family_repo,
            &dependent_repo,
            pagination,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(_) => {}
            _ => panic!("Expected not found error"),
        }
    }
}

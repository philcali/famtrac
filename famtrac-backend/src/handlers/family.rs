use crate::context::RequestContext;
use crate::domain::{Family, FamilyId, Timestamp};
use crate::errors::{validate_family_name, HandlerError};
use crate::repository::FamilyRepository;
use serde::{Deserialize, Serialize};

/// Request body for creating a new family
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateFamilyRequest {
    pub name: String,
}

/// Request body for updating a family
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateFamilyRequest {
    pub name: String,
}

/// Response body for family operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyResponse {
    pub id: FamilyId,
    pub name: String,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Response body for listing families
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyListResponse {
    pub families: Vec<FamilyResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

impl From<Family> for FamilyResponse {
    fn from(family: Family) -> Self {
        FamilyResponse {
            id: family.id,
            name: family.name,
            owner_id: family.owner_id.0,
            created_at: family.created_at.to_iso8601(),
            updated_at: family.updated_at.to_iso8601(),
        }
    }
}

/// Handler for POST /families
/// Creates a new family with the authenticated identity as owner
///
/// Requirements:
/// - 1.1: Create a new Family with a unique identifier and name
/// - 1.2: Return descriptive error messages for invalid data
/// - 8.1: Return 400 Bad Request for malformed JSON
/// - 10.1: Parse incoming JSON request bodies into strongly-typed Rust structures
pub async fn create_family<R: FamilyRepository>(
    request_body: &str,
    context: &RequestContext,
    repository: &R,
) -> Result<(u16, String), HandlerError> {
    // Parse request body (Requirement 8.1, 10.1)
    let request: CreateFamilyRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate family name (Requirement 1.2)
    validate_family_name(&request.name)?;

    // Create family with identity as owner (Requirement 1.1)
    let now = Timestamp::now();
    let family = Family {
        id: FamilyId::new(),
        name: request.name,
        owner_id: context.identity_id.clone(),
        created_at: now,
        updated_at: now,
    };

    // Persist to repository
    let created_family = repository.create(family).await?;

    // Convert to response and serialize
    let response = FamilyResponse::from(created_family);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 201 Created
    Ok((201, response_json))
}
/// Handler for GET /families/{family_id}
/// Retrieves a family by ID with authorization check
///
/// Requirements:
/// - 1.3: Retrieve a Family by its unique identifier
/// - 1.5: Verify the Identity has access to that Family
/// - 10.3: Serialize response data into valid JSON format
pub async fn get_family<R: FamilyRepository>(
    family_id: FamilyId,
    context: &RequestContext,
    repository: &R,
) -> Result<(u16, String), HandlerError> {
    // Retrieve family from repository with owner-scoped key (Requirement 1.3, 4.1)
    let family = repository
        .get(context.identity_id.clone(), family_id)
        .await?;

    // Return 404 if family doesn't exist or owner doesn't match (implicit authorization)
    let family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Convert to response and serialize (Requirement 10.3)
    let response = FamilyResponse::from(family);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

/// Handler for PUT /families/{family_id}
/// Updates a family with authorization check
///
/// Requirements:
/// - 1.4: Update Family information
/// - 1.5: Verify the Identity has access to that Family
/// - 8.1: Return 400 Bad Request for malformed JSON
/// - 10.1: Parse incoming JSON request bodies into strongly-typed Rust structures
pub async fn update_family<R: FamilyRepository>(
    family_id: FamilyId,
    request_body: &str,
    context: &RequestContext,
    repository: &R,
) -> Result<(u16, String), HandlerError> {
    // Parse request body (Requirement 8.1, 10.1)
    let request: UpdateFamilyRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate family name
    validate_family_name(&request.name)?;

    // Retrieve existing family with owner-scoped key (Requirement 1.4, 4.2)
    let family = repository
        .get(context.identity_id.clone(), family_id)
        .await?;

    // Return 404 if family doesn't exist or owner doesn't match (implicit authorization)
    let mut family = family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Update family data
    family.name = request.name;
    family.updated_at = Timestamp::now();

    // Persist to repository
    let updated_family = repository.update(family).await?;

    // Convert to response and serialize
    let response = FamilyResponse::from(updated_family);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

/// Handler for GET /families
/// Lists all families owned by the authenticated identity
///
/// Requirements:
/// - 1.3: Retrieve families by owner
/// - 10.3: Serialize response data into valid JSON format
pub async fn list_families<R: FamilyRepository>(
    context: &RequestContext,
    repository: &R,
) -> Result<(u16, String), HandlerError> {
    // Retrieve all families owned by the authenticated identity
    let families = repository.get_by_owner(context.identity_id.clone()).await?;

    // Convert to response format
    let families_response: Vec<FamilyResponse> =
        families.into_iter().map(FamilyResponse::from).collect();

    // Wrap in list response structure
    let response = FamilyListResponse {
        families: families_response,
        next_token: None, // Pagination not yet implemented
    };

    // Serialize response (Requirement 10.3)
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::IdentityId;

    use crate::test_utils::mocks::MockFamilyRepository;

    fn create_test_context(identity_id: &str) -> RequestContext {
        RequestContext {
            identity_id: IdentityId::new(identity_id.to_string()),
        }
    }

    #[tokio::test]
    async fn test_create_family_success() {
        let request_body = r#"{"name": "Smith Family"}"#;
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let result = create_family(request_body, &context, &repository).await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 201);

        let response: FamilyResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.name, "Smith Family");
        assert_eq!(response.owner_id, "user-123");
    }

    #[tokio::test]
    async fn test_create_family_invalid_json() {
        let request_body = r#"{"name": invalid}"#;
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let result = create_family(request_body, &context, &repository).await;

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
    async fn test_create_family_empty_name() {
        let request_body = r#"{"name": ""}"#;
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let result = create_family(request_body, &context, &repository).await;

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
    async fn test_create_family_whitespace_only_name() {
        let request_body = r#"{"name": "   "}"#;
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let result = create_family(request_body, &context, &repository).await;

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
    async fn test_create_family_name_too_long() {
        let long_name = "a".repeat(101);
        let request_body = format!(r#"{{"name": "{}"}}"#, long_name);
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let result = create_family(&request_body, &context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "name");
                assert!(err.message.contains("too long"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_family_repository_error() {
        let request_body = r#"{"name": "Smith Family"}"#;
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::with_failure();

        let result = create_family(request_body, &context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Store(_) => {}
            _ => panic!("Expected store error"),
        }
    }

    #[tokio::test]
    async fn test_create_family_valid_name_at_boundary() {
        let name_100_chars = "a".repeat(100);
        let request_body = format!(r#"{{"name": "{}"}}"#, name_100_chars);
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let result = create_family(&request_body, &context, &repository).await;

        assert!(result.is_ok());
        let (status, _) = result.unwrap();
        assert_eq!(status, 201);
    }
    #[tokio::test]
    async fn test_get_family_success() {
        let family_id = FamilyId::new();
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-123");

        let repository = MockFamilyRepository::new();
        let family = Family {
            id: family_id,
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        repository
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let result = get_family(family_id, &context, &repository).await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: FamilyResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.name, "Smith Family");
        assert_eq!(response.owner_id, "user-123");
    }

    #[tokio::test]
    async fn test_get_family_not_found() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let result = get_family(family_id, &context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_get_family_unauthorized() {
        let family_id = FamilyId::new();
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-456"); // Different user

        let repository = MockFamilyRepository::new();
        let family = Family {
            id: family_id,
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        repository
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let result = get_family(family_id, &context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(_) => {}
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_update_family_success() {
        let family_id = FamilyId::new();
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-123");

        let repository = MockFamilyRepository::new();
        let family = Family {
            id: family_id,
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        repository
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = r#"{"name": "Updated Family Name"}"#;
        let result = update_family(family_id, request_body, &context, &repository).await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: FamilyResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.name, "Updated Family Name");
        assert_eq!(response.owner_id, "user-123");
    }

    #[tokio::test]
    async fn test_update_family_not_found() {
        let family_id = FamilyId::new();
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let request_body = r#"{"name": "Updated Family Name"}"#;
        let result = update_family(family_id, request_body, &context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(msg) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_update_family_unauthorized() {
        let family_id = FamilyId::new();
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-456"); // Different user

        let repository = MockFamilyRepository::new();
        let family = Family {
            id: family_id,
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        repository
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = r#"{"name": "Updated Family Name"}"#;
        let result = update_family(family_id, request_body, &context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::NotFound(_) => {}
            _ => panic!("Expected not found error"),
        }
    }

    #[tokio::test]
    async fn test_update_family_invalid_json() {
        let family_id = FamilyId::new();
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-123");

        let repository = MockFamilyRepository::new();
        let family = Family {
            id: family_id,
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        repository
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = r#"{"name": invalid}"#;
        let result = update_family(family_id, request_body, &context, &repository).await;

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
    async fn test_update_family_empty_name() {
        let family_id = FamilyId::new();
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-123");

        let repository = MockFamilyRepository::new();
        let family = Family {
            id: family_id,
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        repository
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = r#"{"name": ""}"#;
        let result = update_family(family_id, request_body, &context, &repository).await;

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
    async fn test_update_family_name_too_long() {
        let family_id = FamilyId::new();
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-123");

        let repository = MockFamilyRepository::new();
        let family = Family {
            id: family_id,
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        repository
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let long_name = "a".repeat(101);
        let request_body = format!(r#"{{"name": "{}"}}"#, long_name);
        let result = update_family(family_id, &request_body, &context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "name");
                assert!(err.message.contains("too long"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_update_family_repository_error() {
        let family_id = FamilyId::new();
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-123");

        let mut repository = MockFamilyRepository::new();
        let family = Family {
            id: family_id,
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        repository
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);
        repository = MockFamilyRepository::with_failure();

        let request_body = r#"{"name": "Updated Family Name"}"#;
        let result = update_family(family_id, request_body, &context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Store(_) => {}
            _ => panic!("Expected store error"),
        }
    }
    #[tokio::test]
    async fn test_list_families_success() {
        let owner_id = IdentityId::new("user-123".to_string());
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        // Create multiple families
        let family1 = Family {
            id: FamilyId::new(),
            name: "Smith Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        let family2 = Family {
            id: FamilyId::new(),
            name: "Jones Family".to_string(),
            owner_id: owner_id.clone(),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };

        repository
            .families
            .lock()
            .unwrap()
            .insert(family1.id, family1);
        repository
            .families
            .lock()
            .unwrap()
            .insert(family2.id, family2);

        let result = list_families(&context, &repository).await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: FamilyListResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.families.len(), 2);
        assert!(response.families.iter().any(|f| f.name == "Smith Family"));
        assert!(response.families.iter().any(|f| f.name == "Jones Family"));
    }

    #[tokio::test]
    async fn test_list_families_empty() {
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::new();

        let result = list_families(&context, &repository).await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: FamilyListResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.families.len(), 0);
    }

    #[tokio::test]
    async fn test_list_families_repository_error() {
        let context = create_test_context("user-123");
        let repository = MockFamilyRepository::with_failure();

        let result = list_families(&context, &repository).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Store(_) => {}
            _ => panic!("Expected store error"),
        }
    }
}

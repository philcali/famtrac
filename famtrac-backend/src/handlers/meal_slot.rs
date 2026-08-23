use super::permission::check_permission;
use crate::context::RequestContext;
use crate::domain::{
    DependentId, FamilyId, MealSlot, MealSlotId, PermissionAction, RecipeId, Timestamp,
};
use crate::errors::HandlerError;
use crate::handlers::pagination::PaginationParams;
use crate::repository::{FamilyRepository, MealSlotRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request body for creating a new meal slot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateMealSlotRequest {
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub day: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Request body for updating a meal slot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateMealSlotRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Response body for meal slot operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealSlotResponse {
    pub id: MealSlotId,
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub day: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Response body for listing meal slots
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealSlotListResponse {
    pub meal_slots: Vec<MealSlotResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

impl From<MealSlot> for MealSlotResponse {
    fn from(meal_slot: MealSlot) -> Self {
        MealSlotResponse {
            id: meal_slot.id,
            family_id: meal_slot.family_id,
            dependent_id: meal_slot.dependent_id,
            day: meal_slot.day,
            time: meal_slot.time,
            recipe_id: meal_slot.recipe_id.map(|r| r.0.to_string()),
            notes: meal_slot.notes,
            created_at: meal_slot.created_at.to_iso8601(),
            updated_at: meal_slot.updated_at.to_iso8601(),
        }
    }
}

/// Handler for POST /families/{family_id}/dependents/{dependent_id}/meal-slots
pub async fn create_meal_slot<F: FamilyRepository, M: MealSlotRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    meal_slot_repository: &M,
) -> Result<(u16, String), HandlerError> {
    let request: CreateMealSlotRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate day format (YYYY-MM-DD)
    if request.day.len() != 10
        || request.day.chars().nth(4) != Some('-')
        || request.day.chars().nth(7) != Some('-')
        || !request.day[..4].chars().all(|c| c.is_ascii_digit())
        || !request.day[5..7].chars().all(|c| c.is_ascii_digit())
        || !request.day[8..].chars().all(|c| c.is_ascii_digit())
    {
        return Err(HandlerError::Validation(crate::errors::ValidationError {
            field: "day".to_string(),
            message: "Day must be in YYYY-MM-DD format".to_string(),
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

    // Create meal slot
    let now = Timestamp::now();
    let recipe_id = request
        .recipe_id
        .and_then(|s| Uuid::parse_str(&s).ok().map(RecipeId));
    let meal_slot = MealSlot {
        id: MealSlotId::new(),
        family_id,
        dependent_id,
        day: request.day,
        time: request.time,
        recipe_id,
        notes: request.notes,
        created_at: now,
        updated_at: now,
        share_id: None,
        permission_scope: None,
    };

    let created_meal_slot = meal_slot_repository.create(meal_slot).await?;
    let response = MealSlotResponse::from(created_meal_slot);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((201, response_json))
}

/// Handler for GET /families/{family_id}/dependents/{dependent_id}/meal-slots/{meal_slot_id}
pub async fn get_meal_slot<F: FamilyRepository, M: MealSlotRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    meal_slot_id: MealSlotId,
    context: &RequestContext,
    family_repository: &F,
    meal_slot_repository: &M,
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

    let meal_slot = meal_slot_repository
        .get(family_id, dependent_id, meal_slot_id)
        .await?;
    let meal_slot = meal_slot.ok_or(HandlerError::NotFound(format!(
        "Meal slot with id {:?} not found",
        meal_slot_id
    )))?;

    let response = MealSlotResponse::from(meal_slot);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for PUT /families/{family_id}/dependents/{dependent_id}/meal-slots/{meal_slot_id}
pub async fn update_meal_slot<F: FamilyRepository, M: MealSlotRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    meal_slot_id: MealSlotId,
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    meal_slot_repository: &M,
) -> Result<(u16, String), HandlerError> {
    let request: UpdateMealSlotRequest = serde_json::from_str(request_body).map_err(|e| {
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

    let meal_slot = meal_slot_repository
        .get(family_id, dependent_id, meal_slot_id)
        .await?;
    let mut meal_slot = meal_slot.ok_or(HandlerError::NotFound(format!(
        "Meal slot with id {:?} not found",
        meal_slot_id
    )))?;

    if let Some(day) = &request.day {
        if day.len() != 10
            || day.chars().nth(4) != Some('-')
            || day.chars().nth(7) != Some('-')
            || !day[..4].chars().all(|c| c.is_ascii_digit())
            || !day[5..7].chars().all(|c| c.is_ascii_digit())
            || !day[8..].chars().all(|c| c.is_ascii_digit())
        {
            return Err(HandlerError::Validation(crate::errors::ValidationError {
                field: "day".to_string(),
                message: "Day must be in YYYY-MM-DD format".to_string(),
                constraint: Some("must be a valid date string".to_string()),
            }));
        }
        meal_slot.day = day.clone();
    }
    if let Some(time) = &request.time {
        if time.len() != 5 || time.chars().filter(|c| *c == ':').count() != 1 {
            return Err(HandlerError::Validation(crate::errors::ValidationError {
                field: "time".to_string(),
                message: "Time must be in HH:MM format".to_string(),
                constraint: Some("must be a valid time string".to_string()),
            }));
        }
        meal_slot.time = time.clone();
    }
    if let Some(recipe_id_str) = &request.recipe_id {
        meal_slot.recipe_id = Uuid::parse_str(recipe_id_str).ok().map(RecipeId);
    }
    if let Some(notes) = &request.notes {
        meal_slot.notes = Some(notes.clone());
    }
    meal_slot.updated_at = Timestamp::now();

    let updated_meal_slot = meal_slot_repository.update(meal_slot).await?;
    let response = MealSlotResponse::from(updated_meal_slot);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    Ok((200, response_json))
}

/// Handler for DELETE /families/{family_id}/dependents/{dependent_id}/meal-slots/{meal_slot_id}
pub async fn delete_meal_slot<F: FamilyRepository, M: MealSlotRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    meal_slot_id: MealSlotId,
    context: &RequestContext,
    family_repository: &F,
    meal_slot_repository: &M,
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

    meal_slot_repository
        .delete(family_id, dependent_id, meal_slot_id)
        .await?;

    Ok((204, String::new()))
}

/// Handler for GET /families/{family_id}/dependents/{dependent_id}/meal-slots
pub async fn list_meal_slots<F: FamilyRepository, M: MealSlotRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    context: &RequestContext,
    family_repository: &F,
    meal_slot_repository: &M,
    day: Option<String>,
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

    let paginated_result = meal_slot_repository
        .list_by_dependent(family_id, dependent_id, day, pagination)
        .await?;

    let meal_slots_response: Vec<MealSlotResponse> = paginated_result
        .items
        .into_iter()
        .map(MealSlotResponse::from)
        .collect();

    let response = MealSlotListResponse {
        meal_slots: meal_slots_response,
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
    use crate::test_utils::mocks::{MockFamilyRepository, MockMealSlotRepository};

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
    async fn test_create_meal_slot_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "dependent_id": "{}", "day": "2025-01-15", "time": "12:00", "notes": "Lunch slot"}}"#,
            family_id.0, dependent_id.0
        );

        let result = create_meal_slot(
            family_id,
            dependent_id,
            &request_body,
            &context,
            &family_repo,
            &meal_slot_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 201);

        let response: MealSlotResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.day, "2025-01-15");
        assert_eq!(response.time, "12:00");
        assert_eq!(response.notes, Some("Lunch slot".to_string()));
    }

    #[tokio::test]
    async fn test_create_meal_slot_invalid_json() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let request_body = r#"{"day": invalid}"#;
        let result = create_meal_slot(
            family_id,
            dependent_id,
            request_body,
            &context,
            &family_repo,
            &meal_slot_repo,
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
    async fn test_create_meal_slot_invalid_day_format() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "dependent_id": "{}", "day": "01-15-2025", "time": "12:00"}}"#,
            family_id.0, dependent_id.0
        );

        let result = create_meal_slot(
            family_id,
            dependent_id,
            &request_body,
            &context,
            &family_repo,
            &meal_slot_repo,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HandlerError::Validation(err) => {
                assert_eq!(err.field, "day");
                assert!(err.message.contains("YYYY-MM-DD"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_create_meal_slot_invalid_time_format() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let request_body = format!(
            r#"{{"family_id": "{}", "dependent_id": "{}", "day": "2025-01-15", "time": "1200"}}"#,
            family_id.0, dependent_id.0
        );

        let result = create_meal_slot(
            family_id,
            dependent_id,
            &request_body,
            &context,
            &family_repo,
            &meal_slot_repo,
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
    async fn test_create_meal_slot_family_not_found() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let request_body = format!(
            r#"{{"family_id": "{}", "dependent_id": "{}", "day": "2025-01-15", "time": "12:00"}}"#,
            family_id.0, dependent_id.0
        );

        let result = create_meal_slot(
            family_id,
            dependent_id,
            &request_body,
            &context,
            &family_repo,
            &meal_slot_repo,
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
    async fn test_get_meal_slot_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let meal_slot_id = MealSlotId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let meal_slot = MealSlot {
            id: meal_slot_id,
            family_id,
            dependent_id,
            day: "2025-01-15".to_string(),
            time: "12:00".to_string(),
            recipe_id: None,
            notes: Some("Lunch".to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        meal_slot_repo
            .meal_slots
            .lock()
            .unwrap()
            .insert(meal_slot_id, meal_slot);

        let result = get_meal_slot(
            family_id,
            dependent_id,
            meal_slot_id,
            &context,
            &family_repo,
            &meal_slot_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: MealSlotResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.day, "2025-01-15");
    }

    #[tokio::test]
    async fn test_get_meal_slot_not_found() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let meal_slot_id = MealSlotId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let result = get_meal_slot(
            family_id,
            dependent_id,
            meal_slot_id,
            &context,
            &family_repo,
            &meal_slot_repo,
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
    async fn test_list_meal_slots_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let meal_slot1 = MealSlot {
            id: MealSlotId::new(),
            family_id,
            dependent_id,
            day: "2025-01-15".to_string(),
            time: "08:00".to_string(),
            recipe_id: None,
            notes: Some("Breakfast".to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        let meal_slot2 = MealSlot {
            id: MealSlotId::new(),
            family_id,
            dependent_id,
            day: "2025-01-15".to_string(),
            time: "12:00".to_string(),
            recipe_id: None,
            notes: Some("Lunch".to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        meal_slot_repo
            .meal_slots
            .lock()
            .unwrap()
            .insert(meal_slot1.id, meal_slot1);
        meal_slot_repo
            .meal_slots
            .lock()
            .unwrap()
            .insert(meal_slot2.id, meal_slot2);

        let pagination = PaginationParams {
            limit: None,
            next_token: None,
        };
        let result = list_meal_slots(
            family_id,
            dependent_id,
            &context,
            &family_repo,
            &meal_slot_repo,
            None,
            pagination,
        )
        .await;

        assert!(result.is_ok());
        let (status, response_json) = result.unwrap();
        assert_eq!(status, 200);

        let response: MealSlotListResponse = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response.meal_slots.len(), 2);
        assert!(response.next_token.is_none());
    }

    #[tokio::test]
    async fn test_delete_meal_slot_success() {
        let family_id = FamilyId::new();
        let dependent_id = DependentId::new();
        let meal_slot_id = MealSlotId::new();
        let context = create_test_context("user-123");
        let family_repo = MockFamilyRepository::new();
        let meal_slot_repo = MockMealSlotRepository::new();

        let family = create_test_family(family_id, "user-123");
        family_repo
            .families
            .lock()
            .unwrap()
            .insert(family_id, family);

        let meal_slot = MealSlot {
            id: meal_slot_id,
            family_id,
            dependent_id,
            day: "2025-01-15".to_string(),
            time: "12:00".to_string(),
            recipe_id: None,
            notes: Some("Lunch".to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            share_id: None,
            permission_scope: None,
        };
        meal_slot_repo
            .meal_slots
            .lock()
            .unwrap()
            .insert(meal_slot_id, meal_slot);

        let result = delete_meal_slot(
            family_id,
            dependent_id,
            meal_slot_id,
            &context,
            &family_repo,
            &meal_slot_repo,
        )
        .await;

        assert!(result.is_ok());
        let (status, _) = result.unwrap();
        assert_eq!(status, 204);
    }
}

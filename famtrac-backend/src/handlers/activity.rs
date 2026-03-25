use super::permission::check_permission;
use crate::context::RequestContext;
use crate::domain::{
    Activity, ActivityId, ActivityType, DependentId, FamilyId, PermissionAction, Timestamp,
};
use crate::errors::{validate_activity_timestamp, validate_activity_type, HandlerError};
use crate::handlers::pagination::PaginationParams;
use crate::repository::{
    ActivityQueryParams, ActivityRepository, DependentRepository, FamilyRepository,
};
use serde::{Deserialize, Serialize};

/// Request body for creating a new activity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateActivityRequest {
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub timestamp: Timestamp,
    #[serde(flatten)]
    pub activity_type: ActivityType,
}

/// Request body for updating an activity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateActivityRequest {
    pub timestamp: Timestamp,
    #[serde(flatten)]
    pub activity_type: ActivityType,
}

/// Response body for activity operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityResponse {
    pub id: ActivityId,
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub timestamp: String,
    #[serde(flatten)]
    pub activity_type: ActivityType,
    pub created_at: String,
    pub updated_at: String,
}

/// Response body for listing activities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityListResponse {
    pub activities: Vec<ActivityResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

impl From<Activity> for ActivityResponse {
    fn from(activity: Activity) -> Self {
        ActivityResponse {
            id: activity.id,
            family_id: activity.family_id,
            dependent_id: activity.dependent_id,
            timestamp: activity.timestamp.to_iso8601(),
            activity_type: activity.activity_type,
            created_at: activity.created_at.to_iso8601(),
            updated_at: activity.updated_at.to_iso8601(),
        }
    }
}

/// Handler for POST /activities
/// Creates a new activity with authorization check on parent dependent's family
///
/// Requirements:
/// - 3.1: Create an Activity with a unique identifier, timestamp, Dependent identifier, activity type, and type-specific attributes
/// - 3.2: Support activity types including feeding, diaper change, sleep, and pumping
/// - 3.3: Validate that the timestamp is not in the future
/// - 3.4: Return descriptive error messages for invalid data
/// - 3.5: Verify the Identity has access to the associated Dependent's Family
/// - 7.1, 7.2, 7.3, 7.4: Validate type-specific attributes
/// - 8.1: Return 400 Bad Request for malformed JSON
/// - 10.1: Parse incoming JSON request bodies into strongly-typed Rust structures
pub async fn create_activity<F: FamilyRepository, D: DependentRepository, A: ActivityRepository>(
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    dependent_repository: &D,
    activity_repository: &A,
) -> Result<(u16, String), HandlerError> {
    // Parse request body (Requirement 8.1, 10.1)
    let request: CreateActivityRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate activity timestamp (Requirement 3.3, 3.4)
    validate_activity_timestamp(&request.timestamp)?;

    // Validate type-specific attributes (Requirements 7.1, 7.2, 7.3, 7.4)
    validate_activity_type(&request.activity_type)?;

    // Retrieve parent Family with owner-scoped key (implicit authorization) (Requirement 3.5, 4.6)
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
        PermissionAction::ActivityWrite,
    )?;

    // Verify dependent exists
    let _dependent = dependent_repository
        .get(request.family_id, request.dependent_id)
        .await?;
    let _dependent = _dependent.ok_or(HandlerError::NotFound(format!(
        "Dependent with id {:?} not found",
        request.dependent_id
    )))?;

    // Create activity (Requirement 3.1, 3.2)
    let now = Timestamp::now();
    let activity = Activity {
        id: ActivityId::new(),
        family_id: request.family_id,
        dependent_id: request.dependent_id,
        timestamp: request.timestamp,
        activity_type: request.activity_type,
        created_at: now,
        updated_at: now,
        share_id: None,
        permission_scope: None,
    };

    // Persist to repository
    let created_activity = activity_repository.create(activity).await?;

    // Convert to response and serialize
    let response = ActivityResponse::from(created_activity);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 201 Created
    Ok((201, response_json))
}

/// Handler for GET /activities/{activity_id}
/// Retrieves an activity by ID with authorization check
///
/// Requirements:
/// - 3.1: Retrieve an Activity by its unique identifier
/// - 10.3: Serialize response data into valid JSON format
pub async fn get_activity<F: FamilyRepository, D: DependentRepository, A: ActivityRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    activity_id: ActivityId,
    context: &RequestContext,
    family_repository: &F,
    _dependent_repository: &D,
    activity_repository: &A,
) -> Result<(u16, String), HandlerError> {
    // Authorize access via owner-scoped family get (implicit authorization) (Requirement 4.6)
    let _family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let _family = _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Retrieve activity from repository (Requirement 3.1)
    let activity = activity_repository
        .get(family_id, dependent_id, activity_id)
        .await?;

    // Return 404 if activity doesn't exist
    let activity = activity.ok_or(HandlerError::NotFound(format!(
        "Activity with id {:?} not found",
        activity_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.1, 4.5)
    check_permission(
        activity.share_id.as_ref(),
        activity.permission_scope.as_ref(),
        PermissionAction::ActivityRead,
    )?;

    // Convert to response and serialize (Requirement 10.3)
    let response = ActivityResponse::from(activity);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

/// Handler for PUT /activities/{activity_id}
/// Updates an activity with authorization check
///
/// Requirements:
/// - 5.1: Update an existing Activity by its unique identifier
/// - 5.2: Return descriptive error messages for invalid data
/// - 5.3: Return not found error when Activity identifier does not exist
/// - 5.4: Preserve the original creation timestamp when updating an Activity
/// - 5.5: Verify the Identity has access to the associated Dependent's Family
/// - 8.1: Return 400 Bad Request for malformed JSON
/// - 10.1: Parse incoming JSON request bodies into strongly-typed Rust structures
#[allow(clippy::too_many_arguments)]
pub async fn update_activity<F: FamilyRepository, D: DependentRepository, A: ActivityRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    activity_id: ActivityId,
    request_body: &str,
    context: &RequestContext,
    family_repository: &F,
    _dependent_repository: &D,
    activity_repository: &A,
) -> Result<(u16, String), HandlerError> {
    // Parse request body (Requirement 8.1, 10.1)
    let request: UpdateActivityRequest = serde_json::from_str(request_body).map_err(|e| {
        HandlerError::Validation(crate::errors::ValidationError {
            field: "body".to_string(),
            message: format!("Invalid JSON: {}", e),
            constraint: Some("must be valid JSON".to_string()),
        })
    })?;

    // Validate activity timestamp
    validate_activity_timestamp(&request.timestamp)?;

    // Validate type-specific attributes (Requirement 5.2)
    validate_activity_type(&request.activity_type)?;

    // Authorize access via owner-scoped family get (implicit authorization) (Requirement 5.5, 4.6)
    let _family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let _family = _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Retrieve existing activity (Requirement 5.1, 5.3)
    let activity = activity_repository
        .get(family_id, dependent_id, activity_id)
        .await?;

    // Return 404 if activity doesn't exist
    let mut activity = activity.ok_or(HandlerError::NotFound(format!(
        "Activity with id {:?} not found",
        activity_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.2, 4.5)
    check_permission(
        activity.share_id.as_ref(),
        activity.permission_scope.as_ref(),
        PermissionAction::ActivityWrite,
    )?;

    // Update activity data (preserve created_at per Requirement 5.4)
    activity.timestamp = request.timestamp;
    activity.activity_type = request.activity_type;
    activity.updated_at = Timestamp::now();
    // Note: created_at is NOT modified

    // Persist to repository
    let updated_activity = activity_repository.update(activity).await?;

    // Convert to response and serialize
    let response = ActivityResponse::from(updated_activity);
    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

/// Handler for DELETE /activities/{activity_id}
/// Deletes an activity with authorization check
///
/// Requirements:
/// - 6.1: Delete an Activity by its unique identifier
/// - 6.2: Return not found error when Activity identifier does not exist
/// - 6.3: Remove the Activity from the Data_Store permanently
/// - 6.4: Verify the Identity has access to the associated Dependent's Family
pub async fn delete_activity<F: FamilyRepository, D: DependentRepository, A: ActivityRepository>(
    family_id: FamilyId,
    dependent_id: DependentId,
    activity_id: ActivityId,
    context: &RequestContext,
    family_repository: &F,
    _dependent_repository: &D,
    activity_repository: &A,
) -> Result<(u16, String), HandlerError> {
    // Authorize access via owner-scoped family get (implicit authorization) (Requirement 6.4, 4.6)
    let _family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let _family = _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Retrieve Activity (Requirement 6.2)
    let activity = activity_repository
        .get(family_id, dependent_id, activity_id)
        .await?;

    // Return 404 if activity doesn't exist (Requirement 6.2)
    let _activity = activity.ok_or(HandlerError::NotFound(format!(
        "Activity with id {:?} not found",
        activity_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.2, 4.5)
    check_permission(
        _activity.share_id.as_ref(),
        _activity.permission_scope.as_ref(),
        PermissionAction::ActivityWrite,
    )?;

    // Delete Activity from repository (Requirement 6.1, 6.3)
    activity_repository
        .delete(family_id, dependent_id, activity_id)
        .await?;

    // Return 204 No Content
    Ok((204, String::new()))
}

/// Handler for GET /dependents/{dependent_id}/activities
/// Queries activities for a dependent with filters
///
/// Requirements:
/// - 4.1: Retrieve all Activities for a Dependent within a date range
/// - 4.2: Return only Activities matching the specified type filter
/// - 4.3: Return Activities sorted by timestamp in descending order
/// - 4.4: Return an empty list when no activities match the query criteria
/// - 4.5: Return descriptive error message when date range is invalid
/// - 4.6: Verify the Identity has access to the associated Dependent's Family
/// - 10.3: Serialize response data into valid JSON format
#[allow(clippy::too_many_arguments)]
pub async fn query_activities<
    F: FamilyRepository,
    D: DependentRepository,
    A: ActivityRepository,
>(
    family_id: FamilyId,
    dependent_id: DependentId,
    start_date: Option<Timestamp>,
    end_date: Option<Timestamp>,
    activity_type: Option<ActivityType>,
    context: &RequestContext,
    family_repository: &F,
    dependent_repository: &D,
    activity_repository: &A,
    pagination: PaginationParams,
) -> Result<(u16, String), HandlerError> {
    // Authorize access via owner-scoped family get (implicit authorization) (Requirement 4.6)
    let _family = family_repository
        .get(context.identity_id.clone(), family_id)
        .await?;
    let _family = _family.ok_or(HandlerError::NotFound(format!(
        "Family with id {:?} not found",
        family_id
    )))?;

    // Enforce permission on mirrored resources (Requirement 4.1, 4.5)
    check_permission(
        _family.share_id.as_ref(),
        _family.permission_scope.as_ref(),
        PermissionAction::ActivityRead,
    )?;

    // Verify dependent exists
    let _dependent = dependent_repository.get(family_id, dependent_id).await?;
    let _dependent = _dependent.ok_or(HandlerError::NotFound(format!(
        "Dependent with id {:?} not found",
        dependent_id
    )))?;

    // Validate date range (Requirement 4.5)
    if let (Some(start), Some(end)) = (start_date, end_date) {
        if end.0 < start.0 {
            return Err(HandlerError::Validation(crate::errors::ValidationError {
                field: "date_range".to_string(),
                message: "End date cannot be before start date".to_string(),
                constraint: Some("end_date must be >= start_date".to_string()),
            }));
        }
    }

    // Query activities from repository with filters (Requirements 4.1, 4.2, 4.3, 4.4)
    let params = ActivityQueryParams {
        family_id,
        dependent_id,
        start_date,
        end_date,
        activity_type,
    };

    let paginated_result = activity_repository.query(params, pagination).await?;

    // Convert to response and wrap in list response structure (Requirement 10.3)
    let activities_response: Vec<ActivityResponse> = paginated_result
        .items
        .into_iter()
        .map(ActivityResponse::from)
        .collect();

    let response = ActivityListResponse {
        activities: activities_response,
        next_token: paginated_result.next_token,
    };

    let response_json = serde_json::to_string(&response)
        .map_err(|e| HandlerError::InternalError(format!("Failed to serialize response: {}", e)))?;

    // Return 200 OK
    Ok((200, response_json))
}

use crate::domain::{PermissionAction, PermissionScope, ShareId};
use crate::errors::HandlerError;

/// Checks whether an operation is permitted on a resource.
///
/// - If `share_id` is `None`, the resource is owned by the caller — full access is granted.
/// - If `share_id` is `Some`, the resource is mirrored via a share and the
///   `permission_scope` must contain the `required_action`.
///
/// Returns `HandlerError::Forbidden` when the required action is not in scope.
pub fn check_permission(
    resource_share_id: Option<&ShareId>,
    resource_permission_scope: Option<&PermissionScope>,
    required_action: PermissionAction,
) -> Result<(), HandlerError> {
    // Owned resource — no restrictions
    let Some(_share_id) = resource_share_id else {
        return Ok(());
    };

    let scope = resource_permission_scope.ok_or_else(|| {
        HandlerError::Forbidden("Insufficient permissions for this operation".to_string())
    })?;

    if scope.actions.contains(&required_action) {
        Ok(())
    } else {
        Err(HandlerError::Forbidden(
            "Insufficient permissions for this operation".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn owned_resource_allows_any_action() {
        let result = check_permission(None, None, PermissionAction::ActivityWrite);
        assert!(result.is_ok());
    }

    #[test]
    fn mirrored_resource_allows_action_in_scope() {
        let share_id = ShareId(Uuid::new_v4());
        let scope = PermissionScope {
            actions: vec![
                PermissionAction::FamilyRead,
                PermissionAction::DependentRead,
            ],
        };
        let result = check_permission(
            Some(&share_id),
            Some(&scope),
            PermissionAction::DependentRead,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn mirrored_resource_denies_action_not_in_scope() {
        let share_id = ShareId(Uuid::new_v4());
        let scope = PermissionScope {
            actions: vec![PermissionAction::FamilyRead],
        };
        let result = check_permission(
            Some(&share_id),
            Some(&scope),
            PermissionAction::DependentWrite,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status_code(), 403);
    }

    #[test]
    fn mirrored_resource_without_scope_is_forbidden() {
        let share_id = ShareId(Uuid::new_v4());
        let result = check_permission(Some(&share_id), None, PermissionAction::FamilyRead);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status_code(), 403);
    }
}

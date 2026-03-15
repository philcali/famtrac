use crate::domain::{Activity, Dependent, Family, IdentityId};
use crate::errors::AuthError;
use crate::repository::{DependentRepository, FamilyRepository};
use async_trait::async_trait;

/// Trait for resources that can be authorized against an identity
/// Implements hierarchical authorization:
/// - Family: identity must be the owner
/// - Dependent: identity must own the parent Family
/// - Activity: identity must own the parent Family via Dependent
#[async_trait]
pub trait Authorizable {
    /// Authorize access to this resource for the given identity
    /// Returns Ok(()) if authorized, Err(AuthError::Forbidden) otherwise
    async fn authorize<F, D>(
        &self,
        identity: &IdentityId,
        family_repo: &F,
        dependent_repo: &D,
    ) -> Result<(), AuthError>
    where
        F: FamilyRepository,
        D: DependentRepository;
}

#[async_trait]
impl Authorizable for Family {
    async fn authorize<F, D>(
        &self,
        identity: &IdentityId,
        _family_repo: &F,
        _dependent_repo: &D,
    ) -> Result<(), AuthError>
    where
        F: FamilyRepository,
        D: DependentRepository,
    {
        // Family authorization: verify owner_id matches identity
        if self.owner_id == *identity {
            Ok(())
        } else {
            Err(AuthError::Forbidden(format!(
                "Access denied to family {}",
                self.id.0
            )))
        }
    }
}

#[async_trait]
impl Authorizable for Dependent {
    async fn authorize<F, D>(
        &self,
        identity: &IdentityId,
        family_repo: &F,
        _dependent_repo: &D,
    ) -> Result<(), AuthError>
    where
        F: FamilyRepository,
        D: DependentRepository,
    {
        // Dependent authorization: verify identity owns parent Family
        let family = family_repo
            .get(self.family_id)
            .await
            .map_err(|e| AuthError::Forbidden(format!("Failed to verify family access: {}", e)))?
            .ok_or_else(|| {
                AuthError::Forbidden(format!("Parent family {} not found", self.family_id.0))
            })?;

        if family.owner_id == *identity {
            Ok(())
        } else {
            Err(AuthError::Forbidden(format!(
                "Access denied to dependent {}",
                self.id.0
            )))
        }
    }
}

#[async_trait]
impl Authorizable for Activity {
    async fn authorize<F, D>(
        &self,
        identity: &IdentityId,
        family_repo: &F,
        _dependent_repo: &D,
    ) -> Result<(), AuthError>
    where
        F: FamilyRepository,
        D: DependentRepository,
    {
        // Activity authorization: verify identity owns parent Family
        let family = family_repo
            .get(self.family_id)
            .await
            .map_err(|e| AuthError::Forbidden(format!("Failed to verify family access: {}", e)))?
            .ok_or_else(|| {
                AuthError::Forbidden(format!("Parent family {} not found", self.family_id.0))
            })?;

        if family.owner_id == *identity {
            Ok(())
        } else {
            Err(AuthError::Forbidden(format!(
                "Access denied to activity {}",
                self.id.0
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Activity, ActivityId, ActivityType, Date, Dependent, DependentId, Family, FamilyId,
        FeedingType, IdentityId, Timestamp,
    };
    use crate::errors::StoreError;
    use crate::repository::{DependentRepository, FamilyRepository};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock repositories for testing
    #[derive(Clone)]
    struct MockFamilyRepository {
        families: Arc<Mutex<HashMap<FamilyId, Family>>>,
    }

    impl MockFamilyRepository {
        fn new() -> Self {
            Self {
                families: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn add_family(&self, family: Family) {
            self.families.lock().unwrap().insert(family.id, family);
        }
    }

    #[async_trait]
    impl FamilyRepository for MockFamilyRepository {
        async fn create(&self, family: Family) -> Result<Family, StoreError> {
            self.add_family(family.clone());
            Ok(family)
        }

        async fn get(&self, id: FamilyId) -> Result<Option<Family>, StoreError> {
            Ok(self.families.lock().unwrap().get(&id).cloned())
        }

        async fn update(&self, family: Family) -> Result<Family, StoreError> {
            self.add_family(family.clone());
            Ok(family)
        }

        async fn get_by_owner(&self, owner_id: IdentityId) -> Result<Vec<Family>, StoreError> {
            Ok(self
                .families
                .lock()
                .unwrap()
                .values()
                .filter(|f| f.owner_id == owner_id)
                .cloned()
                .collect())
        }
    }

    #[derive(Clone)]
    struct MockDependentRepository {
        dependents: Arc<Mutex<HashMap<DependentId, Dependent>>>,
    }

    impl MockDependentRepository {
        fn new() -> Self {
            Self {
                dependents: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn add_dependent(&self, dependent: Dependent) {
            self.dependents
                .lock()
                .unwrap()
                .insert(dependent.id, dependent);
        }
    }

    #[async_trait]
    impl DependentRepository for MockDependentRepository {
        async fn create(&self, dependent: Dependent) -> Result<Dependent, StoreError> {
            self.add_dependent(dependent.clone());
            Ok(dependent)
        }

        async fn get(
            &self,
            _family_id: FamilyId,
            id: DependentId,
        ) -> Result<Option<Dependent>, StoreError> {
            Ok(self.dependents.lock().unwrap().get(&id).cloned())
        }

        async fn update(&self, dependent: Dependent) -> Result<Dependent, StoreError> {
            self.add_dependent(dependent.clone());
            Ok(dependent)
        }

        async fn list_by_family(&self, family_id: FamilyId) -> Result<Vec<Dependent>, StoreError> {
            Ok(self
                .dependents
                .lock()
                .unwrap()
                .values()
                .filter(|d| d.family_id == family_id)
                .cloned()
                .collect())
        }
    }

    fn create_test_family(owner_id: &str) -> Family {
        Family {
            id: FamilyId::new(),
            name: "Test Family".to_string(),
            owner_id: IdentityId::new(owner_id.to_string()),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    fn create_test_dependent(family_id: FamilyId) -> Dependent {
        Dependent {
            id: DependentId::new(),
            family_id,
            name: "Test Dependent".to_string(),
            date_of_birth: Date::from_naive_date(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    fn create_test_activity(family_id: FamilyId, dependent_id: DependentId) -> Activity {
        Activity {
            id: ActivityId::new(),
            family_id,
            dependent_id,
            timestamp: Timestamp::now(),
            activity_type: ActivityType::Feeding {
                feeding_type: FeedingType::Breast,
                volume_ml: None,
            },
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    #[tokio::test]
    async fn test_family_authorization_owner_success() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let owner_id = IdentityId::new("user-123".to_string());
        let family = create_test_family("user-123");

        let result = family
            .authorize(&owner_id, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_family_authorization_non_owner_forbidden() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let other_identity = IdentityId::new("user-456".to_string());
        let family = create_test_family("user-123");

        let result = family
            .authorize(&other_identity, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_dependent_authorization_owner_success() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let owner_id = IdentityId::new("user-123".to_string());
        let family = create_test_family("user-123");
        family_repo.add_family(family.clone());

        let dependent = create_test_dependent(family.id);

        let result = dependent
            .authorize(&owner_id, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dependent_authorization_non_owner_forbidden() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let other_identity = IdentityId::new("user-456".to_string());
        let family = create_test_family("user-123");
        family_repo.add_family(family.clone());

        let dependent = create_test_dependent(family.id);

        let result = dependent
            .authorize(&other_identity, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_dependent_authorization_family_not_found() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let owner_id = IdentityId::new("user-123".to_string());
        let dependent = create_test_dependent(FamilyId::new());

        let result = dependent
            .authorize(&owner_id, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_activity_authorization_owner_success() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let owner_id = IdentityId::new("user-123".to_string());
        let family = create_test_family("user-123");
        family_repo.add_family(family.clone());

        let dependent = create_test_dependent(family.id);
        dependent_repo.add_dependent(dependent.clone());

        let activity = create_test_activity(family.id, dependent.id);

        let result = activity
            .authorize(&owner_id, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_activity_authorization_non_owner_forbidden() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let other_identity = IdentityId::new("user-456".to_string());
        let family = create_test_family("user-123");
        family_repo.add_family(family.clone());

        let dependent = create_test_dependent(family.id);
        dependent_repo.add_dependent(dependent.clone());

        let activity = create_test_activity(family.id, dependent.id);

        let result = activity
            .authorize(&other_identity, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_activity_authorization_dependent_not_found() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let owner_id = IdentityId::new("user-123".to_string());
        let family_id = FamilyId::new();
        let activity = create_test_activity(family_id, DependentId::new());

        let result = activity
            .authorize(&owner_id, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_activity_authorization_family_not_found() {
        let family_repo = MockFamilyRepository::new();
        let dependent_repo = MockDependentRepository::new();

        let owner_id = IdentityId::new("user-123".to_string());
        let family_id = FamilyId::new();
        let dependent = create_test_dependent(family_id);
        dependent_repo.add_dependent(dependent.clone());

        let activity = create_test_activity(family_id, dependent.id);

        let result = activity
            .authorize(&owner_id, &family_repo, &dependent_repo)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::Forbidden(_)));
    }
}

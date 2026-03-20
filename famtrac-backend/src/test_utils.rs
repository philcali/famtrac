// Test utilities and mock implementations
//
// This module provides mock repositories that can be used in both
// unit tests (in src/) and integration tests (in tests/).

pub mod mocks {
    use crate::domain::{
        Activity, ActivityId, Dependent, DependentId, Family, FamilyId, IdentityId, Share, ShareId,
    };
    use crate::errors::StoreError;
    use crate::handlers::{PaginatedResponse, PaginationParams};
    use crate::repository::{
        ActivityQueryParams, ActivityRepository, DependentRepository, FamilyRepository,
        ShareRepository,
    };
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock implementation of FamilyRepository for testing
    #[derive(Clone)]
    pub struct MockFamilyRepository {
        pub should_fail: bool,
        pub families: Arc<Mutex<HashMap<FamilyId, Family>>>,
    }

    impl MockFamilyRepository {
        pub fn new() -> Self {
            MockFamilyRepository {
                should_fail: false,
                families: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn with_failure() -> Self {
            MockFamilyRepository {
                should_fail: true,
                families: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn insert(&self, family: Family) {
            self.families.lock().unwrap().insert(family.id, family);
        }
    }

    impl Default for MockFamilyRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl FamilyRepository for MockFamilyRepository {
        async fn create(&self, family: Family) -> Result<Family, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.families
                .lock()
                .unwrap()
                .insert(family.id, family.clone());
            Ok(family)
        }

        async fn get(
            &self,
            owner_id: IdentityId,
            id: FamilyId,
        ) -> Result<Option<Family>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            Ok(self
                .families
                .lock()
                .unwrap()
                .get(&id)
                .filter(|f| f.owner_id == owner_id)
                .cloned())
        }

        async fn update(&self, family: Family) -> Result<Family, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.families
                .lock()
                .unwrap()
                .insert(family.id, family.clone());
            Ok(family)
        }

        async fn get_by_owner(
            &self,
            owner_id: IdentityId,
            pagination: PaginationParams,
        ) -> Result<PaginatedResponse<Family>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            let all: Vec<Family> = self
                .families
                .lock()
                .unwrap()
                .values()
                .filter(|f| f.owner_id == owner_id)
                .cloned()
                .collect();
            let offset = pagination
                .next_token
                .as_deref()
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = pagination.effective_limit() as usize;
            let page: Vec<Family> = all.into_iter().skip(offset).take(limit + 1).collect();
            let has_more = page.len() > limit;
            let items: Vec<Family> = page.into_iter().take(limit).collect();
            let next_token = if has_more {
                Some((offset + limit).to_string())
            } else {
                None
            };
            Ok(PaginatedResponse::with_next_token(items, next_token))
        }

        async fn delete(&self, owner_id: IdentityId, id: FamilyId) -> Result<(), StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            let mut families = self.families.lock().unwrap();
            if let Some(family) = families.get(&id) {
                if family.owner_id == owner_id {
                    families.remove(&id);
                }
            }
            Ok(())
        }
    }

    /// Mock implementation of DependentRepository for testing
    #[derive(Clone)]
    pub struct MockDependentRepository {
        pub should_fail: bool,
        pub dependents: Arc<Mutex<HashMap<DependentId, Dependent>>>,
    }

    impl MockDependentRepository {
        pub fn new() -> Self {
            MockDependentRepository {
                should_fail: false,
                dependents: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn with_failure() -> Self {
            MockDependentRepository {
                should_fail: true,
                dependents: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn insert(&self, dependent: Dependent) {
            self.dependents
                .lock()
                .unwrap()
                .insert(dependent.id, dependent);
        }
    }

    impl Default for MockDependentRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl DependentRepository for MockDependentRepository {
        async fn create(&self, dependent: Dependent) -> Result<Dependent, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.dependents
                .lock()
                .unwrap()
                .insert(dependent.id, dependent.clone());
            Ok(dependent)
        }

        async fn get(
            &self,
            _family_id: FamilyId,
            id: DependentId,
        ) -> Result<Option<Dependent>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            Ok(self.dependents.lock().unwrap().get(&id).cloned())
        }

        async fn update(&self, dependent: Dependent) -> Result<Dependent, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.dependents
                .lock()
                .unwrap()
                .insert(dependent.id, dependent.clone());
            Ok(dependent)
        }

        async fn list_by_family(
            &self,
            family_id: FamilyId,
            pagination: PaginationParams,
        ) -> Result<PaginatedResponse<Dependent>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            let all: Vec<Dependent> = self
                .dependents
                .lock()
                .unwrap()
                .values()
                .filter(|d| d.family_id == family_id)
                .cloned()
                .collect();
            let offset = pagination
                .next_token
                .as_deref()
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = pagination.effective_limit() as usize;
            let page: Vec<Dependent> = all.into_iter().skip(offset).take(limit + 1).collect();
            let has_more = page.len() > limit;
            let items: Vec<Dependent> = page.into_iter().take(limit).collect();
            let next_token = if has_more {
                Some((offset + limit).to_string())
            } else {
                None
            };
            Ok(PaginatedResponse::with_next_token(items, next_token))
        }

        async fn delete(&self, _family_id: FamilyId, id: DependentId) -> Result<(), StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.dependents.lock().unwrap().remove(&id);
            Ok(())
        }
    }

    /// Mock implementation of ActivityRepository for testing
    #[derive(Clone)]
    pub struct MockActivityRepository {
        pub should_fail: bool,
        pub activities: Arc<Mutex<HashMap<ActivityId, Activity>>>,
    }

    impl MockActivityRepository {
        pub fn new() -> Self {
            MockActivityRepository {
                should_fail: false,
                activities: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn with_failure() -> Self {
            MockActivityRepository {
                should_fail: true,
                activities: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn insert(&self, activity: Activity) {
            self.activities
                .lock()
                .unwrap()
                .insert(activity.id, activity);
        }
    }

    impl Default for MockActivityRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl ActivityRepository for MockActivityRepository {
        async fn create(&self, activity: Activity) -> Result<Activity, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.activities
                .lock()
                .unwrap()
                .insert(activity.id, activity.clone());
            Ok(activity)
        }

        async fn get(
            &self,
            _family_id: FamilyId,
            _dependent_id: DependentId,
            id: ActivityId,
        ) -> Result<Option<Activity>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            Ok(self.activities.lock().unwrap().get(&id).cloned())
        }

        async fn update(&self, activity: Activity) -> Result<Activity, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.activities
                .lock()
                .unwrap()
                .insert(activity.id, activity.clone());
            Ok(activity)
        }

        async fn delete(
            &self,
            _family_id: FamilyId,
            _dependent_id: DependentId,
            id: ActivityId,
        ) -> Result<(), StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.activities.lock().unwrap().remove(&id);
            Ok(())
        }

        async fn query(
            &self,
            params: ActivityQueryParams,
            pagination: PaginationParams,
        ) -> Result<PaginatedResponse<Activity>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            let mut all: Vec<Activity> = self
                .activities
                .lock()
                .unwrap()
                .values()
                .filter(|a| a.dependent_id == params.dependent_id)
                .cloned()
                .collect();
            // Sort by timestamp descending to match GSI behavior
            all.sort_by(|a, b| b.timestamp.0.cmp(&a.timestamp.0));
            let offset = pagination
                .next_token
                .as_deref()
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = pagination.effective_limit() as usize;
            let page: Vec<Activity> = all.into_iter().skip(offset).take(limit + 1).collect();
            let has_more = page.len() > limit;
            let items: Vec<Activity> = page.into_iter().take(limit).collect();
            let next_token = if has_more {
                Some((offset + limit).to_string())
            } else {
                None
            };
            Ok(PaginatedResponse::with_next_token(items, next_token))
        }
    }

    /// Mock implementation of ShareRepository for testing
    #[derive(Clone)]
    pub struct MockShareRepository {
        pub should_fail: bool,
        pub shares: Arc<Mutex<HashMap<ShareId, Share>>>,
    }

    impl MockShareRepository {
        pub fn new() -> Self {
            MockShareRepository {
                should_fail: false,
                shares: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn with_failure() -> Self {
            MockShareRepository {
                should_fail: true,
                shares: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn insert(&self, share: Share) {
            self.shares.lock().unwrap().insert(share.id, share);
        }
    }

    impl Default for MockShareRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl ShareRepository for MockShareRepository {
        async fn create(&self, share: Share) -> Result<Share, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            // Check for duplicate (same family + email)
            let exists = self.shares.lock().unwrap().values().any(|s| {
                s.family_id == share.family_id && s.accepter_username == share.accepter_username
            });
            if exists {
                return Err(StoreError::ConflictError(
                    "Share already exists for this family and email".to_string(),
                ));
            }
            self.shares.lock().unwrap().insert(share.id, share.clone());
            Ok(share)
        }

        async fn get(
            &self,
            requester_id: IdentityId,
            share_id: ShareId,
        ) -> Result<Option<Share>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            Ok(self
                .shares
                .lock()
                .unwrap()
                .get(&share_id)
                .filter(|s| s.requester_id == requester_id)
                .cloned())
        }

        async fn get_by_email_and_share_id(
            &self,
            accepter_username: &str,
            share_id: ShareId,
        ) -> Result<Option<Share>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            Ok(self
                .shares
                .lock()
                .unwrap()
                .get(&share_id)
                .filter(|s| s.accepter_username == accepter_username)
                .cloned())
        }

        async fn update(&self, share: Share) -> Result<Share, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.shares.lock().unwrap().insert(share.id, share.clone());
            Ok(share)
        }

        async fn delete(
            &self,
            requester_id: IdentityId,
            share_id: ShareId,
        ) -> Result<(), StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            let should_remove = self
                .shares
                .lock()
                .unwrap()
                .get(&share_id)
                .map(|s| s.requester_id == requester_id)
                .unwrap_or(false);
            if should_remove {
                self.shares.lock().unwrap().remove(&share_id);
                Ok(())
            } else {
                Err(StoreError::NotFound("Share not found".to_string()))
            }
        }

        async fn list_by_family(
            &self,
            requester_id: IdentityId,
            family_id: FamilyId,
            pagination: PaginationParams,
        ) -> Result<PaginatedResponse<Share>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            let all: Vec<Share> = self
                .shares
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.requester_id == requester_id && s.family_id == family_id)
                .cloned()
                .collect();
            let offset = pagination
                .next_token
                .as_deref()
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = pagination.effective_limit() as usize;
            let page: Vec<Share> = all.into_iter().skip(offset).take(limit + 1).collect();
            let has_more = page.len() > limit;
            let items: Vec<Share> = page.into_iter().take(limit).collect();
            let next_token = if has_more {
                Some((offset + limit).to_string())
            } else {
                None
            };
            Ok(PaginatedResponse::with_next_token(items, next_token))
        }

        async fn list_by_accepter_username(
            &self,
            username: &str,
            pagination: PaginationParams,
        ) -> Result<PaginatedResponse<Share>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            let all: Vec<Share> = self
                .shares
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.accepter_username == username)
                .cloned()
                .collect();
            let offset = pagination
                .next_token
                .as_deref()
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = pagination.effective_limit() as usize;
            let page: Vec<Share> = all.into_iter().skip(offset).take(limit + 1).collect();
            let has_more = page.len() > limit;
            let items: Vec<Share> = page.into_iter().take(limit).collect();
            let next_token = if has_more {
                Some((offset + limit).to_string())
            } else {
                None
            };
            Ok(PaginatedResponse::with_next_token(items, next_token))
        }

        async fn get_by_family_and_username(
            &self,
            family_id: FamilyId,
            accepter_username: &str,
        ) -> Result<Option<Share>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            Ok(self
                .shares
                .lock()
                .unwrap()
                .values()
                .find(|s| s.family_id == family_id && s.accepter_username == accepter_username)
                .cloned())
        }
    }
}

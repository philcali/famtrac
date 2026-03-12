// Test utilities and mock implementations
//
// This module provides mock repositories that can be used in both
// unit tests (in src/) and integration tests (in tests/).

pub mod mocks {
    use crate::domain::{
        Activity, ActivityId, Dependent, DependentId, Family, FamilyId, IdentityId,
    };
    use crate::errors::StoreError;
    use crate::repository::{
        ActivityQueryParams, ActivityRepository, DependentRepository, FamilyRepository,
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

        async fn get(&self, id: FamilyId) -> Result<Option<Family>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            Ok(self.families.lock().unwrap().get(&id).cloned())
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

        async fn get_by_owner(&self, owner_id: IdentityId) -> Result<Vec<Family>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
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

        async fn get(&self, id: DependentId) -> Result<Option<Dependent>, StoreError> {
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

        async fn list_by_family(&self, family_id: FamilyId) -> Result<Vec<Dependent>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
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

        async fn get(&self, id: ActivityId) -> Result<Option<Activity>, StoreError> {
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

        async fn delete(&self, id: ActivityId) -> Result<(), StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            self.activities.lock().unwrap().remove(&id);
            Ok(())
        }

        async fn query(&self, params: ActivityQueryParams) -> Result<Vec<Activity>, StoreError> {
            if self.should_fail {
                return Err(StoreError::QueryError("Mock failure".to_string()));
            }
            Ok(self
                .activities
                .lock()
                .unwrap()
                .values()
                .filter(|a| a.dependent_id == params.dependent_id)
                .cloned()
                .collect())
        }
    }
}

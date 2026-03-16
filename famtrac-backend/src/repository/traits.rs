use crate::domain::{
    Activity, ActivityType, Date, Dependent, DependentId, Family, FamilyId, IdentityId,
};
use crate::errors::StoreError;
use async_trait::async_trait;

/// Repository trait for Family operations
#[async_trait]
pub trait FamilyRepository: Send + Sync {
    /// Create a new family
    async fn create(&self, family: Family) -> Result<Family, StoreError>;

    /// Get a family by owner ID and family ID (tenant-isolated lookup)
    async fn get(&self, owner_id: IdentityId, id: FamilyId) -> Result<Option<Family>, StoreError>;

    /// Update an existing family
    async fn update(&self, family: Family) -> Result<Family, StoreError>;

    /// Get all families owned by a specific identity
    async fn get_by_owner(&self, owner_id: IdentityId) -> Result<Vec<Family>, StoreError>;
}

/// Repository trait for Dependent operations
#[async_trait]
pub trait DependentRepository: Send + Sync {
    /// Create a new dependent
    async fn create(&self, dependent: Dependent) -> Result<Dependent, StoreError>;

    /// Get a dependent by family ID and dependent ID
    async fn get(
        &self,
        family_id: FamilyId,
        id: DependentId,
    ) -> Result<Option<Dependent>, StoreError>;

    /// Update an existing dependent
    async fn update(&self, dependent: Dependent) -> Result<Dependent, StoreError>;

    /// List all dependents for a specific family
    async fn list_by_family(&self, family_id: FamilyId) -> Result<Vec<Dependent>, StoreError>;

    /// Delete a dependent by family ID and dependent ID
    async fn delete(&self, family_id: FamilyId, id: DependentId) -> Result<(), StoreError>;
}

/// Query parameters for activity queries
#[derive(Debug, Clone)]
pub struct ActivityQueryParams {
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub activity_type: Option<ActivityType>,
}

/// Repository trait for Activity operations
#[async_trait]
pub trait ActivityRepository: Send + Sync {
    /// Create a new activity
    async fn create(&self, activity: Activity) -> Result<Activity, StoreError>;

    /// Get an activity by family ID, dependent ID, and activity ID
    async fn get(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        id: crate::domain::ActivityId,
    ) -> Result<Option<Activity>, StoreError>;

    /// Update an existing activity
    async fn update(&self, activity: Activity) -> Result<Activity, StoreError>;

    /// Delete an activity by family ID, dependent ID, and activity ID
    async fn delete(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        id: crate::domain::ActivityId,
    ) -> Result<(), StoreError>;

    /// Query activities with filters
    async fn query(&self, params: ActivityQueryParams) -> Result<Vec<Activity>, StoreError>;
}

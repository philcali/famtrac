use crate::domain::{
    Activity, ActivityType, Date, Dependent, DependentId, Family, FamilyId, IdentityId,
};
use crate::errors::StoreError;

/// Repository trait for Family operations
pub trait FamilyRepository {
    /// Create a new family
    fn create(&self, family: Family) -> Result<Family, StoreError>;

    /// Get a family by ID
    fn get(&self, id: FamilyId) -> Result<Option<Family>, StoreError>;

    /// Update an existing family
    fn update(&self, family: Family) -> Result<Family, StoreError>;

    /// Get all families owned by a specific identity
    fn get_by_owner(&self, owner_id: IdentityId) -> Result<Vec<Family>, StoreError>;
}

/// Repository trait for Dependent operations
pub trait DependentRepository {
    /// Create a new dependent
    fn create(&self, dependent: Dependent) -> Result<Dependent, StoreError>;

    /// Get a dependent by ID
    fn get(&self, id: DependentId) -> Result<Option<Dependent>, StoreError>;

    /// Update an existing dependent
    fn update(&self, dependent: Dependent) -> Result<Dependent, StoreError>;

    /// List all dependents for a specific family
    fn list_by_family(&self, family_id: FamilyId) -> Result<Vec<Dependent>, StoreError>;
}

/// Query parameters for activity queries
#[derive(Debug, Clone)]
pub struct ActivityQueryParams {
    pub dependent_id: DependentId,
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub activity_type: Option<ActivityType>,
}

/// Repository trait for Activity operations
pub trait ActivityRepository {
    /// Create a new activity
    fn create(&self, activity: Activity) -> Result<Activity, StoreError>;

    /// Get an activity by ID
    fn get(&self, id: crate::domain::ActivityId) -> Result<Option<Activity>, StoreError>;

    /// Update an existing activity
    fn update(&self, activity: Activity) -> Result<Activity, StoreError>;

    /// Delete an activity by ID
    fn delete(&self, id: crate::domain::ActivityId) -> Result<(), StoreError>;

    /// Query activities with filters
    fn query(&self, params: ActivityQueryParams) -> Result<Vec<Activity>, StoreError>;
}

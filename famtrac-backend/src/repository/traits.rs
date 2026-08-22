use crate::domain::{
    Activity, ActivityType, Dependent, DependentId, Family, FamilyId, FeedingLog, FeedingLogId,
    IdentityId, MealSlot, MealSlotId, Recipe, RecipeId, Share, ShareId, Timestamp,
};
use crate::errors::StoreError;
use crate::handlers::{PaginatedResponse, PaginationParams};
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
    async fn get_by_owner(
        &self,
        owner_id: IdentityId,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Family>, StoreError>;

    /// Delete a family by owner ID and family ID
    async fn delete(&self, owner_id: IdentityId, id: FamilyId) -> Result<(), StoreError>;
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
    async fn list_by_family(
        &self,
        family_id: FamilyId,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Dependent>, StoreError>;

    /// Delete a dependent by family ID and dependent ID
    async fn delete(&self, family_id: FamilyId, id: DependentId) -> Result<(), StoreError>;
}

/// Query parameters for activity queries
///
/// `start_date` and `end_date` are pre-computed UTC timestamps that already
/// account for the client's timezone offset. The repository layer uses them
/// directly without further timezone conversion.
#[derive(Debug, Clone)]
pub struct ActivityQueryParams {
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub start_date: Option<Timestamp>,
    pub end_date: Option<Timestamp>,
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
    async fn query(
        &self,
        params: ActivityQueryParams,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Activity>, StoreError>;
}

/// Repository trait for Share operations
#[async_trait]
pub trait ShareRepository: Send + Sync {
    /// Create a new share (dual-write to owner and email partitions)
    async fn create(&self, share: Share) -> Result<Share, StoreError>;

    /// Get a share by requester ID and share ID (owner partition lookup)
    async fn get(
        &self,
        requester_id: IdentityId,
        share_id: ShareId,
    ) -> Result<Option<Share>, StoreError>;

    /// Get a share by accepter email and share ID (email partition direct lookup)
    async fn get_by_username_and_share_id(
        &self,
        accepter_email: &str,
        share_id: ShareId,
    ) -> Result<Option<Share>, StoreError>;

    /// Update an existing share (dual-write transaction)
    async fn update(&self, share: Share) -> Result<Share, StoreError>;

    /// Delete a share by requester ID and share ID (dual-delete transaction)
    async fn delete(&self, requester_id: IdentityId, share_id: ShareId) -> Result<(), StoreError>;

    /// List all shares for a family (owner partition, filtered by family_id)
    async fn list_by_family(
        &self,
        requester_id: IdentityId,
        family_id: FamilyId,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Share>, StoreError>;

    /// List all shares by accepter username (username partition lookup)
    async fn list_by_accepter_username(
        &self,
        username: &str,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Share>, StoreError>;

    /// Get a share by family ID and accepter username (username partition, filtered by family_id)
    async fn get_by_family_and_username(
        &self,
        family_id: FamilyId,
        accepter_username: &str,
    ) -> Result<Option<Share>, StoreError>;
}

/// Repository trait for Recipe operations
#[async_trait]
pub trait RecipeRepository: Send + Sync {
    /// Create a new recipe
    async fn create(&self, recipe: Recipe) -> Result<Recipe, StoreError>;

    /// Get a recipe by family ID and recipe ID
    async fn get(&self, family_id: FamilyId, id: RecipeId) -> Result<Option<Recipe>, StoreError>;

    /// Update an existing recipe
    async fn update(&self, recipe: Recipe) -> Result<Recipe, StoreError>;

    /// List all recipes for a specific family
    async fn list_by_family(
        &self,
        family_id: FamilyId,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Recipe>, StoreError>;

    /// Delete a recipe by family ID and recipe ID
    async fn delete(&self, family_id: FamilyId, id: RecipeId) -> Result<(), StoreError>;
}

/// Query parameters for meal slot queries
#[derive(Debug, Clone)]
pub struct MealSlotQueryParams {
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub day: Option<String>,
}

/// Repository trait for MealSlot operations
#[async_trait]
pub trait MealSlotRepository: Send + Sync {
    /// Create a new meal slot
    async fn create(&self, meal_slot: MealSlot) -> Result<MealSlot, StoreError>;

    /// Get a meal slot by family ID, dependent ID, and meal slot ID
    async fn get(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        id: MealSlotId,
    ) -> Result<Option<MealSlot>, StoreError>;

    /// Update an existing meal slot
    async fn update(&self, meal_slot: MealSlot) -> Result<MealSlot, StoreError>;

    /// Delete a meal slot by family ID, dependent ID, and meal slot ID
    async fn delete(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        id: MealSlotId,
    ) -> Result<(), StoreError>;

    /// List meal slots for a dependent, optionally filtered by day
    async fn list_by_dependent(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        day: Option<String>,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<MealSlot>, StoreError>;
}

/// Query parameters for feeding log queries
#[derive(Debug, Clone)]
pub struct FeedingLogQueryParams {
    pub family_id: FamilyId,
    pub dependent_id: DependentId,
    pub date: Option<String>,
}

/// Repository trait for FeedingLog operations
#[async_trait]
pub trait FeedingLogRepository: Send + Sync {
    /// Create a new feeding log
    async fn create(&self, feeding_log: FeedingLog) -> Result<FeedingLog, StoreError>;

    /// Get a feeding log by family ID, dependent ID, and feeding log ID
    async fn get(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        id: FeedingLogId,
    ) -> Result<Option<FeedingLog>, StoreError>;

    /// Update an existing feeding log
    async fn update(&self, feeding_log: FeedingLog) -> Result<FeedingLog, StoreError>;

    /// Delete a feeding log by family ID, dependent ID, and feeding log ID
    async fn delete(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        id: FeedingLogId,
    ) -> Result<(), StoreError>;

    /// List feeding logs for a dependent, optionally filtered by date
    async fn list_by_dependent(
        &self,
        family_id: FamilyId,
        dependent_id: DependentId,
        date: Option<String>,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<FeedingLog>, StoreError>;
}

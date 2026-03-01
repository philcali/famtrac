use serde::{Deserialize, Serialize};

use super::{FamilyId, IdentityId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Family {
    pub id: FamilyId,
    pub name: String,
    pub owner_id: IdentityId,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

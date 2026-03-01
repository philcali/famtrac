use serde::{Deserialize, Serialize};

use super::{Date, DependentId, FamilyId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependent {
    pub id: DependentId,
    pub family_id: FamilyId,
    pub name: String,
    pub date_of_birth: Date,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

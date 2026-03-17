use serde::{Deserialize, Serialize};

use super::{Date, DependentId, FamilyId, PermissionScope, ShareId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependent {
    pub id: DependentId,
    pub family_id: FamilyId,
    pub name: String,
    pub date_of_birth: Date,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_id: Option<ShareId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_scope: Option<PermissionScope>,
}

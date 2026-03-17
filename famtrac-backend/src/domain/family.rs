use serde::{Deserialize, Serialize};

use super::{FamilyId, IdentityId, PermissionScope, ShareId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Family {
    pub id: FamilyId,
    pub name: String,
    pub owner_id: IdentityId,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_id: Option<ShareId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_scope: Option<PermissionScope>,
}

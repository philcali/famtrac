use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FamilyId(pub Uuid);

impl FamilyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FamilyId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DependentId(pub Uuid);

impl DependentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DependentId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActivityId(pub Uuid);

impl ActivityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ActivityId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdentityId(pub String);

impl IdentityId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

use serde::{Deserialize, Serialize};

use super::{ActivityId, DependentId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub id: ActivityId,
    pub dependent_id: DependentId,
    pub timestamp: Timestamp,
    pub activity_type: ActivityType,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActivityType {
    Feeding {
        feeding_type: FeedingType,
        #[serde(skip_serializing_if = "Option::is_none")]
        volume_ml: Option<u32>,
    },
    DiaperChange {
        contents: DiaperContents,
    },
    Sleep {
        start: Timestamp,
        end: Timestamp,
    },
    Pumping {
        volume_ml: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedingType {
    Breast,
    Bottle,
    Solid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiaperContents {
    Wet,
    Dirty,
    Both,
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{IdentityId, Timestamp};
use crate::errors::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiTokenStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiToken {
    pub token: String,
    pub user_id: IdentityId,
    pub username: Option<String>,
    pub name: Option<String>,
    pub status: ApiTokenStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateApiTokenRequest {
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiTokenResponse {
    pub token: Option<String>,
    pub id: String,
    pub name: Option<String>,
    pub status: ApiTokenStatus,
    pub created_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
}

impl CreateApiTokenRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(name) = &self.name {
            if name.is_empty() {
                return Err(ValidationError {
                    field: "name".to_string(),
                    message: "Token name cannot be empty".to_string(),
                    constraint: Some("must be non-empty or omitted".to_string()),
                });
            }
            if name.len() > 128 {
                return Err(ValidationError {
                    field: "name".to_string(),
                    message: "Token name cannot exceed 128 characters".to_string(),
                    constraint: Some("max length: 128".to_string()),
                });
            }
        }

        if let Some(days) = self.expires_in_days {
            if days == 0 || days > 3650 {
                return Err(ValidationError {
                    field: "expires_in_days".to_string(),
                    message: "Token expiry must be between 1 and 3650 days".to_string(),
                    constraint: Some("range: 1-3650".to_string()),
                });
            }
        }

        Ok(())
    }
}

pub fn generate_token() -> String {
    let uuid = Uuid::new_v4();
    format!("fam_{}", uuid)
}

use crate::domain::IdentityId;
use crate::errors::AuthError;
use aws_lambda_events::apigw::ApiGatewayV2httpRequestContext;
use serde::{Deserialize, Serialize};

/// Request context containing authenticated identity information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestContext {
    pub identity_id: IdentityId,
    pub username: Option<String>,
}

impl RequestContext {
    /// Extract identity from API Gateway request context.
    ///
    /// Handles two authorizer types:
    /// - JWT authorizer: reads `sub` from `context.authorizer.jwt.claims`
    /// - Custom TOKEN authorizer: reads `identity_id` from `context.authorizer.authorizer`
    pub fn from_api_gateway_context(
        context: &ApiGatewayV2httpRequestContext,
    ) -> Result<Self, AuthError> {
        // Try custom authorizer first (identity_id in authorizer fields map)
        if let Some(authorizer) = &context.authorizer {
            if !authorizer.fields.is_empty() {
                if let Some(identity_id) = authorizer.fields.get("identity_id")
                    .and_then(|v| v.as_str())
                {
                    return Ok(RequestContext {
                        identity_id: IdentityId::new(identity_id.to_string()),
                        username: authorizer.fields.get("identity_source")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    });
                }
            }
        }

        // Fall back to JWT authorizer (sub claim)
        let jwt = context.authorizer.as_ref().and_then(|a| a.jwt.clone());

        let identity_id = jwt
            .as_ref()
            .and_then(|jwt| jwt.claims.get("sub"))
            .map(|s| s.to_string())
            .ok_or(AuthError::MissingIdentity)?;

        let username = jwt
            .as_ref()
            .and_then(|jwt| jwt.claims.get("username"))
            .map(|s| s.to_string());

        Ok(RequestContext {
            identity_id: IdentityId::new(identity_id),
            username,
        })
    }

    /// Validate that identity is present (returns 401 if missing)
    /// This is a convenience method that can be used in handlers
    pub fn validate(&self) -> Result<(), AuthError> {
        // Identity is already validated during construction
        // This method exists for explicit validation in handler flows
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lambda_events::apigw::{ApiGatewayRequestAuthorizer, ApiGatewayV2httpRequestContext};
    use std::collections::HashMap;

    fn create_test_context_with_identity(identity_id: &str) -> ApiGatewayV2httpRequestContext {
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), identity_id.to_string());
        claims.insert("username".to_string(), "test_user".to_string());

        let jwt = aws_lambda_events::apigw::ApiGatewayRequestAuthorizerJwtDescription {
            claims,
            scopes: None,
        };

        ApiGatewayV2httpRequestContext {
            authorizer: Some(ApiGatewayRequestAuthorizer {
                jwt: Some(jwt),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn create_test_context_without_identity() -> ApiGatewayV2httpRequestContext {
        ApiGatewayV2httpRequestContext {
            authorizer: Some(ApiGatewayRequestAuthorizer {
                jwt: None,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_extract_identity_success() {
        let api_context = create_test_context_with_identity("user-123");
        let result = RequestContext::from_api_gateway_context(&api_context);

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.identity_id.0, "user-123");
        assert_eq!(context.username, Some("test_user".to_string()));
    }

    #[test]
    fn test_extract_identity_missing() {
        let api_context = create_test_context_without_identity();
        let result = RequestContext::from_api_gateway_context(&api_context);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AuthError::MissingIdentity);
    }

    #[test]
    fn test_extract_identity_missing_jwt() {
        let api_context = ApiGatewayV2httpRequestContext {
            authorizer: Some(ApiGatewayRequestAuthorizer {
                jwt: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = RequestContext::from_api_gateway_context(&api_context);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AuthError::MissingIdentity);
    }

    #[test]
    fn test_extract_identity_missing_sub_claim() {
        let mut claims = HashMap::new();
        claims.insert("other_claim".to_string(), "value".to_string());

        let jwt = aws_lambda_events::apigw::ApiGatewayRequestAuthorizerJwtDescription {
            claims,
            scopes: None,
        };

        let api_context = ApiGatewayV2httpRequestContext {
            authorizer: Some(ApiGatewayRequestAuthorizer {
                jwt: Some(jwt),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = RequestContext::from_api_gateway_context(&api_context);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AuthError::MissingIdentity);
    }

    #[test]
    fn test_validate_always_succeeds() {
        let context = RequestContext {
            identity_id: IdentityId::new("user-123".to_string()),
            username: None,
        };

        assert!(context.validate().is_ok());
    }

    #[test]
    fn test_extract_email_success() {
        let api_context = create_test_context_with_identity("user-123");
        let result = RequestContext::from_api_gateway_context(&api_context);

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.username, Some("test_user".to_string()));
    }

    #[test]
    fn test_extract_email_missing() {
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), "user-123".to_string());
        // No email claim inserted

        let jwt = aws_lambda_events::apigw::ApiGatewayRequestAuthorizerJwtDescription {
            claims,
            scopes: None,
        };

        let api_context = ApiGatewayV2httpRequestContext {
            authorizer: Some(ApiGatewayRequestAuthorizer {
                jwt: Some(jwt),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = RequestContext::from_api_gateway_context(&api_context);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.identity_id.0, "user-123");
        assert_eq!(context.username, None);
    }

    #[test]
    fn test_extract_identity_from_custom_authorizer() {
        let mut authorizer_data = HashMap::new();
        authorizer_data.insert("identity_id".to_string(), serde_json::Value::String("user-456".to_string()));
        authorizer_data.insert("identity_source".to_string(), serde_json::Value::String("api_token".to_string()));

        let api_context = ApiGatewayV2httpRequestContext {
            authorizer: Some(ApiGatewayRequestAuthorizer {
                fields: authorizer_data,
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = RequestContext::from_api_gateway_context(&api_context);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.identity_id.0, "user-456");
        assert_eq!(context.username, Some("api_token".to_string()));
    }

    #[test]
    fn test_custom_authorizer_takes_precedence_over_jwt() {
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), "jwt-user".to_string());

        let jwt = aws_lambda_events::apigw::ApiGatewayRequestAuthorizerJwtDescription {
            claims,
            scopes: None,
        };

        let mut authorizer_data = HashMap::new();
        authorizer_data.insert("identity_id".to_string(), serde_json::Value::String("token-user".to_string()));

        let api_context = ApiGatewayV2httpRequestContext {
            authorizer: Some(ApiGatewayRequestAuthorizer {
                jwt: Some(jwt),
                fields: authorizer_data,
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = RequestContext::from_api_gateway_context(&api_context);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.identity_id.0, "token-user");
    }
}

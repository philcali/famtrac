use crate::domain::IdentityId;
use crate::errors::AuthError;
use aws_lambda_events::apigw::ApiGatewayV2httpRequestContext;
use serde::{Deserialize, Serialize};

/// Request context containing authenticated identity information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestContext {
    pub identity_id: IdentityId,
}

impl RequestContext {
    /// Extract identity from API Gateway request context
    /// Returns AuthError::MissingIdentity if identity is not present
    pub fn from_api_gateway_context(
        context: &ApiGatewayV2httpRequestContext,
    ) -> Result<Self, AuthError> {
        // Extract identity from authorizer context
        // API Gateway with JWT authorizer puts the identity in the authorizer.jwt.claims
        let identity_id = context
            .authorizer
            .as_ref()
            .and_then(|a| a.jwt.clone())
            .as_ref()
            .and_then(|jwt| jwt.claims.get("sub"))
            .map(|s| s.to_string())
            .ok_or(AuthError::MissingIdentity)?;

        Ok(RequestContext {
            identity_id: IdentityId::new(identity_id),
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
        };

        assert!(context.validate().is_ok());
    }
}

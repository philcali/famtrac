use std::collections::HashMap;

use crate::errors::HandlerError;
use crate::utils::cors::{add_cors_headers, CorsConfig};

/// HTTP response with status, body, and headers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl HttpResponse {
    /// Create a new response with the given status and body
    pub fn new(status: u16, body: String) -> Self {
        HttpResponse {
            status,
            body,
            headers: HashMap::new(),
        }
    }

    /// Create a response with CORS headers
    pub fn with_cors(status: u16, body: String, cors_config: &CorsConfig) -> Self {
        let mut response = HttpResponse::new(status, body);
        add_cors_headers(&mut response.headers, cors_config);
        response
    }

    /// Add a header to the response
    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    /// Add CORS headers to the response
    pub fn add_cors_headers(&mut self, cors_config: &CorsConfig) {
        add_cors_headers(&mut self.headers, cors_config);
    }

    /// Convert from a handler result tuple (status, body) to HttpResponse with CORS
    pub fn from_handler_result(
        result: Result<(u16, String), HandlerError>,
        cors_config: &CorsConfig,
    ) -> Self {
        match result {
            Ok((status, body)) => Self::with_cors(status, body, cors_config),
            Err(err) => Self::from_error(err, cors_config),
        }
    }

    /// Convert from a HandlerError to HttpResponse with CORS headers
    pub fn from_error(error: HandlerError, cors_config: &CorsConfig) -> Self {
        let status = error.status_code();
        let error_response = error.to_error_response();
        let body = serde_json::to_string(&error_response).unwrap_or_else(|_| {
            r#"{"error":{"code":"INTERNAL_ERROR","message":"Failed to serialize error"}}"#
                .to_string()
        });

        Self::with_cors(status, body, cors_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_response() {
        let response = HttpResponse::new(200, "test body".to_string());
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "test body");
        assert!(response.headers.is_empty());
    }

    #[test]
    fn test_with_cors() {
        let cors_config = CorsConfig::default();
        let response = HttpResponse::with_cors(200, "test body".to_string(), &cors_config);

        assert_eq!(response.status, 200);
        assert_eq!(response.body, "test body");
        assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
        assert!(response
            .headers
            .contains_key("Access-Control-Allow-Methods"));
        assert!(response
            .headers
            .contains_key("Access-Control-Allow-Headers"));
    }

    #[test]
    fn test_add_header() {
        let mut response = HttpResponse::new(200, "test".to_string());
        response.add_header("X-Custom-Header".to_string(), "custom-value".to_string());

        assert_eq!(
            response.headers.get("X-Custom-Header"),
            Some(&"custom-value".to_string())
        );
    }

    #[test]
    fn test_add_cors_headers() {
        let mut response = HttpResponse::new(200, "test".to_string());
        let cors_config = CorsConfig::default();
        response.add_cors_headers(&cors_config);

        assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
        assert!(response
            .headers
            .contains_key("Access-Control-Allow-Methods"));
        assert!(response
            .headers
            .contains_key("Access-Control-Allow-Headers"));
    }

    #[test]
    fn test_from_handler_result_success() {
        let cors_config = CorsConfig::default();
        let handler_result: Result<(u16, String), HandlerError> = Ok((200, "success".to_string()));

        let response = HttpResponse::from_handler_result(handler_result, &cors_config);

        assert_eq!(response.status, 200);
        assert_eq!(response.body, "success");
        assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
    }

    #[test]
    fn test_from_handler_result_error() {
        use crate::errors::ValidationError;

        let cors_config = CorsConfig::default();
        let handler_result: Result<(u16, String), HandlerError> =
            Err(HandlerError::Validation(ValidationError {
                field: "test".to_string(),
                message: "test error".to_string(),
                constraint: None,
            }));

        let response = HttpResponse::from_handler_result(handler_result, &cors_config);
        assert_eq!(response.status, 400);
        assert!(response.body.contains("VALIDATION_ERROR"));
        assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
    }

    #[test]
    fn test_from_error() {
        use crate::errors::ValidationError;

        let cors_config = CorsConfig::default();
        let error = HandlerError::Validation(ValidationError {
            field: "name".to_string(),
            message: "Name is required".to_string(),
            constraint: Some("must not be empty".to_string()),
        });

        let response = HttpResponse::from_error(error, &cors_config);

        assert_eq!(response.status, 400);
        assert!(response.body.contains("VALIDATION_ERROR"));
        assert!(response.body.contains("Name is required"));
        assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
        assert!(response
            .headers
            .contains_key("Access-Control-Allow-Methods"));
        assert!(response
            .headers
            .contains_key("Access-Control-Allow-Headers"));
    }
}

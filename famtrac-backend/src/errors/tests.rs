use super::*;

#[test]
fn test_validation_error_status_code() {
    let err = HandlerError::Validation(ValidationError {
        field: "name".to_string(),
        message: "Invalid name".to_string(),
        constraint: None,
    });
    assert_eq!(err.status_code(), 400);
}

#[test]
fn test_auth_error_unauthorized_status_code() {
    let err = HandlerError::Auth(AuthError::Unauthorized("Invalid token".to_string()));
    assert_eq!(err.status_code(), 401);
}

#[test]
fn test_auth_error_missing_identity_status_code() {
    let err = HandlerError::Auth(AuthError::MissingIdentity);
    assert_eq!(err.status_code(), 401);
}

#[test]
fn test_not_found_error_status_code() {
    let err = HandlerError::NotFound("Resource not found".to_string());
    assert_eq!(err.status_code(), 404);
}

#[test]
fn test_store_not_found_error_status_code() {
    let err = HandlerError::Store(StoreError::NotFound("Family not found".to_string()));
    assert_eq!(err.status_code(), 404);
}

#[test]
fn test_store_error_status_code() {
    let err = HandlerError::Store(StoreError::QueryError("Query failed".to_string()));
    assert_eq!(err.status_code(), 500);
}

#[test]
fn test_internal_error_status_code() {
    let err = HandlerError::InternalError("Something went wrong".to_string());
    assert_eq!(err.status_code(), 500);
}

#[test]
fn test_validation_error_response_with_constraint() {
    let err = HandlerError::Validation(ValidationError {
        field: "name".to_string(),
        message: "Name is too long".to_string(),
        constraint: Some("must be between 1 and 100 characters".to_string()),
    });

    let response = err.to_error_response();
    assert_eq!(response.error.code, "VALIDATION_ERROR");
    assert_eq!(response.error.message, "Name is too long");
    assert!(response.error.details.is_some());

    let details = response.error.details.unwrap();
    assert_eq!(details["field"], "name");
    assert_eq!(
        details["constraint"],
        "must be between 1 and 100 characters"
    );
}

#[test]
fn test_validation_error_response_without_constraint() {
    let err = HandlerError::Validation(ValidationError {
        field: "timestamp".to_string(),
        message: "Invalid timestamp".to_string(),
        constraint: None,
    });

    let response = err.to_error_response();
    assert_eq!(response.error.code, "VALIDATION_ERROR");
    assert_eq!(response.error.message, "Invalid timestamp");
    assert!(response.error.details.is_some());

    let details = response.error.details.unwrap();
    assert_eq!(details["field"], "timestamp");
    assert!(details.get("constraint").is_none());
}

#[test]
fn test_unauthorized_error_response() {
    let err = HandlerError::Auth(AuthError::Unauthorized("Invalid credentials".to_string()));

    let response = err.to_error_response();
    assert_eq!(response.error.code, "UNAUTHORIZED");
    assert_eq!(response.error.message, "Invalid credentials");
    assert!(response.error.details.is_none());
}

#[test]
fn test_missing_identity_error_response() {
    let err = HandlerError::Auth(AuthError::MissingIdentity);

    let response = err.to_error_response();
    assert_eq!(response.error.code, "UNAUTHORIZED");
    assert_eq!(response.error.message, "Missing authentication credentials");
    assert!(response.error.details.is_none());
}

#[test]
fn test_not_found_error_response() {
    let err = HandlerError::NotFound("Family not found".to_string());

    let response = err.to_error_response();
    assert_eq!(response.error.code, "NOT_FOUND");
    assert_eq!(response.error.message, "Family not found");
    assert!(response.error.details.is_none());
}

#[test]
fn test_store_not_found_error_response() {
    let err = HandlerError::Store(StoreError::NotFound("Dependent not found".to_string()));

    let response = err.to_error_response();
    assert_eq!(response.error.code, "NOT_FOUND");
    assert_eq!(response.error.message, "Dependent not found");
    assert!(response.error.details.is_none());
}

#[test]
fn test_internal_error_response_hides_details() {
    let err = HandlerError::InternalError("Database connection failed: timeout".to_string());

    let response = err.to_error_response();
    assert_eq!(response.error.code, "INTERNAL_ERROR");
    assert_eq!(response.error.message, "An internal error occurred");
    assert!(response.error.details.is_none());
    // Verify internal details are not exposed
    assert!(!response.error.message.contains("Database"));
    assert!(!response.error.message.contains("timeout"));
}

#[test]
fn test_store_error_response_hides_details() {
    let err = HandlerError::Store(StoreError::ConnectionError(
        "Connection refused".to_string(),
    ));

    let response = err.to_error_response();
    assert_eq!(response.error.code, "INTERNAL_ERROR");
    assert_eq!(response.error.message, "An internal error occurred");
    assert!(response.error.details.is_none());
    // Verify internal details are not exposed
    assert!(!response.error.message.contains("Connection"));
}

#[test]
fn test_error_response_serialization() {
    let err = HandlerError::Validation(ValidationError {
        field: "name".to_string(),
        message: "Invalid name".to_string(),
        constraint: Some("must not be empty".to_string()),
    });

    let response = err.to_error_response();
    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("\"code\":\"VALIDATION_ERROR\""));
    assert!(json.contains("\"message\":\"Invalid name\""));
    assert!(json.contains("\"field\":\"name\""));
    assert!(json.contains("\"constraint\":\"must not be empty\""));
}

#[test]
fn test_error_response_deserialization() {
    let json = r#"{
        "error": {
            "code": "VALIDATION_ERROR",
            "message": "Invalid input",
            "details": {
                "field": "timestamp",
                "constraint": "must not be in the future"
            }
        }
    }"#;

    let response: ErrorResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.error.code, "VALIDATION_ERROR");
    assert_eq!(response.error.message, "Invalid input");
    assert!(response.error.details.is_some());
}

#[test]
fn test_validation_error_from_conversion() {
    let validation_err = ValidationError {
        field: "name".to_string(),
        message: "Invalid".to_string(),
        constraint: None,
    };

    let handler_err: HandlerError = validation_err.into();
    assert!(matches!(handler_err, HandlerError::Validation(_)));
}

#[test]
fn test_store_error_from_conversion() {
    let store_err = StoreError::QueryError("Query failed".to_string());
    let handler_err: HandlerError = store_err.into();
    assert!(matches!(handler_err, HandlerError::Store(_)));
}

#[test]
fn test_auth_error_from_conversion() {
    let auth_err = AuthError::Unauthorized("Access denied".to_string());
    let handler_err: HandlerError = auth_err.into();
    assert!(matches!(handler_err, HandlerError::Auth(_)));
}

#[test]
fn test_error_display_formatting() {
    let err = HandlerError::Validation(ValidationError {
        field: "name".to_string(),
        message: "Invalid name".to_string(),
        constraint: None,
    });
    let display = format!("{}", err);
    assert!(display.contains("Validation error"));
    assert!(display.contains("Invalid name"));
}

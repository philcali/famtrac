/// Integration tests for CORS support
/// Requirements: 11.1, 11.2, 11.3, 11.4
mod common;

use common::mocks::MockFamilyRepository;
use famtrac_backend::context::RequestContext;
use famtrac_backend::domain::{FamilyId, IdentityId};
use famtrac_backend::handlers::{create_family, get_family};
use famtrac_backend::utils::{handle_options, CorsConfig, HttpResponse};

#[test]
fn test_options_handler_returns_cors_headers() {
    // Requirement 11.1: Handle OPTIONS preflight requests
    let cors_config = CorsConfig::default();
    let (status, body, headers) = handle_options(&cors_config);

    // Should return 204 No Content
    assert_eq!(status, 204);
    assert_eq!(body, "");

    // Requirement 11.2: Include Access-Control-Allow-Origin header
    assert!(headers.contains_key("Access-Control-Allow-Origin"));
    assert_eq!(headers.get("Access-Control-Allow-Origin").unwrap(), "*");

    // Requirement 11.3: Include Access-Control-Allow-Methods header
    assert!(headers.contains_key("Access-Control-Allow-Methods"));
    assert_eq!(
        headers.get("Access-Control-Allow-Methods").unwrap(),
        "GET, POST, PUT, DELETE, OPTIONS"
    );

    // Requirement 11.4: Include Access-Control-Allow-Headers header
    assert!(headers.contains_key("Access-Control-Allow-Headers"));
    assert_eq!(
        headers.get("Access-Control-Allow-Headers").unwrap(),
        "Content-Type, Authorization"
    );
}

#[tokio::test]
async fn test_success_response_includes_cors_headers() {
    // Test that successful API responses include CORS headers
    let repository = MockFamilyRepository::new();
    let context = RequestContext {
        identity_id: IdentityId("test-user".to_string()),
        email: None,
    };

    let request_body = r#"{"name":"Test Family"}"#;
    let result = create_family(request_body, &context, &repository).await;

    // Convert handler result to HttpResponse with CORS
    let cors_config = CorsConfig::default();
    let response = HttpResponse::from_handler_result(result, &cors_config);

    // Verify status and body
    assert_eq!(response.status, 201);
    assert!(response.body.contains("Test Family"));

    // Requirement 11.2, 11.3, 11.4: All responses include CORS headers
    assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
    assert!(response
        .headers
        .contains_key("Access-Control-Allow-Methods"));
    assert!(response
        .headers
        .contains_key("Access-Control-Allow-Headers"));
}

#[tokio::test]
async fn test_error_response_includes_cors_headers() {
    // Test that error responses include CORS headers
    let repository = MockFamilyRepository::new();
    let context = RequestContext {
        identity_id: IdentityId("test-user".to_string()),
        email: None,
    };

    // Invalid JSON should trigger an error
    let request_body = r#"{"name":}"#;
    let result = create_family(request_body, &context, &repository).await;

    // Convert handler result to HttpResponse with CORS
    let cors_config = CorsConfig::default();
    let response = HttpResponse::from_handler_result(result, &cors_config);

    // Verify error status
    assert_eq!(response.status, 400);
    assert!(response.body.contains("VALIDATION_ERROR"));

    // Requirement 11.2, 11.3, 11.4: Error responses also include CORS headers
    assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
    assert!(response
        .headers
        .contains_key("Access-Control-Allow-Methods"));
    assert!(response
        .headers
        .contains_key("Access-Control-Allow-Headers"));
}

#[tokio::test]
async fn test_not_found_response_includes_cors_headers() {
    // Test that 404 responses include CORS headers
    let repository = MockFamilyRepository::new();
    let context = RequestContext {
        identity_id: IdentityId("test-user".to_string()),
        email: None,
    };

    let non_existent_id = FamilyId::new();
    let result = get_family(non_existent_id, &context, &repository).await;

    // Convert handler result to HttpResponse with CORS
    let cors_config = CorsConfig::default();
    let response = HttpResponse::from_handler_result(result, &cors_config);

    // Verify 404 status
    assert_eq!(response.status, 404);
    assert!(response.body.contains("NOT_FOUND"));

    // Requirement 11.2, 11.3, 11.4: 404 responses include CORS headers
    assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
    assert!(response
        .headers
        .contains_key("Access-Control-Allow-Methods"));
    assert!(response
        .headers
        .contains_key("Access-Control-Allow-Headers"));
}

#[test]
fn test_custom_cors_config() {
    // Test that custom CORS configuration works
    let cors_config = CorsConfig {
        allow_origin: "https://famtrac.example.com".to_string(),
        allow_methods: "GET, POST".to_string(),
        allow_headers: "Content-Type".to_string(),
    };

    let (status, _body, headers) = handle_options(&cors_config);

    assert_eq!(status, 204);
    assert_eq!(
        headers.get("Access-Control-Allow-Origin").unwrap(),
        "https://famtrac.example.com"
    );
    assert_eq!(
        headers.get("Access-Control-Allow-Methods").unwrap(),
        "GET, POST"
    );
    assert_eq!(
        headers.get("Access-Control-Allow-Headers").unwrap(),
        "Content-Type"
    );
}

#[test]
fn test_all_response_types_have_cors() {
    // Verify that all response types (2xx, 4xx, 5xx) include CORS headers
    let cors_config = CorsConfig::default();

    // 2xx response
    let success_response = HttpResponse::with_cors(200, "OK".to_string(), &cors_config);
    assert!(success_response
        .headers
        .contains_key("Access-Control-Allow-Origin"));

    // 4xx response
    let client_error_response =
        HttpResponse::with_cors(400, "Bad Request".to_string(), &cors_config);
    assert!(client_error_response
        .headers
        .contains_key("Access-Control-Allow-Origin"));

    // 5xx response
    let server_error_response =
        HttpResponse::with_cors(500, "Internal Error".to_string(), &cors_config);
    assert!(server_error_response
        .headers
        .contains_key("Access-Control-Allow-Origin"));
}

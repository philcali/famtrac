# Utilities Module

This module contains utility functions and helpers for the famtrac-backend API.

## CORS Support

The CORS utilities provide support for Cross-Origin Resource Sharing (CORS) headers, enabling web clients to make requests from different origins.

### Requirements

- 11.1: Handle OPTIONS preflight requests
- 11.2: Include Access-Control-Allow-Origin header in all responses
- 11.3: Include Access-Control-Allow-Methods header listing supported HTTP methods
- 11.4: Include Access-Control-Allow-Headers header listing accepted request headers

### Usage

#### Basic Usage with Default Configuration

```rust
use famtrac_backend::utils::{CorsConfig, HttpResponse};
use famtrac_backend::handlers::create_family;

// Create default CORS configuration
let cors_config = CorsConfig::default();

// Call a handler
let result = create_family(request_body, &context, &repository);

// Convert handler result to HttpResponse with CORS headers
let response = HttpResponse::from_handler_result(result, &cors_config);

// Response now includes:
// - Access-Control-Allow-Origin: *
// - Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
// - Access-Control-Allow-Headers: Content-Type, Authorization
```

#### Custom CORS Configuration

```rust
use famtrac_backend::utils::CorsConfig;

let cors_config = CorsConfig {
    allow_origin: "https://famtrac.example.com".to_string(),
    allow_methods: "GET, POST".to_string(),
    allow_headers: "Content-Type".to_string(),
};
```

#### Handling OPTIONS Preflight Requests

```rust
use famtrac_backend::utils::{handle_options, CorsConfig};

let cors_config = CorsConfig::default();
let (status, body, headers) = handle_options(&cors_config);

// Returns:
// - status: 204 (No Content)
// - body: empty string
// - headers: HashMap with CORS headers
```

#### Converting Handler Results

The `HttpResponse::from_handler_result` method automatically handles both success and error cases:

```rust
use famtrac_backend::utils::{CorsConfig, HttpResponse};

let cors_config = CorsConfig::default();

// Success case
let result = Ok((200, "success".to_string()));
let response = HttpResponse::from_handler_result(result, &cors_config);
// response.status = 200, response.body = "success", response.headers includes CORS

// Error case
let result = Err(HandlerError::Validation(ValidationError { ... }));
let response = HttpResponse::from_handler_result(result, &cors_config);
// response.status = 400, response.body = JSON error, response.headers includes CORS
```

### Integration with Lambda

When integrating with AWS Lambda, the routing layer should:

1. Check if the request method is OPTIONS
2. If OPTIONS, call `handle_options` and return the response
3. Otherwise, call the appropriate handler and convert the result using `HttpResponse::from_handler_result`
4. Return the HttpResponse with status, body, and headers to API Gateway

Example:

```rust
async fn lambda_handler(event: Request, context: Context) -> Result<Response, Error> {
    let cors_config = CorsConfig::default();
    
    // Handle OPTIONS preflight
    if event.request_context.http.method == "OPTIONS" {
        let (status, body, headers) = handle_options(&cors_config);
        return Ok(Response {
            status_code: status,
            body: body,
            headers: headers,
            ..Default::default()
        });
    }
    
    // Route to appropriate handler
    let handler_result = match (event.request_context.http.method.as_str(), event.path.as_str()) {
        ("POST", "/families") => create_family(...),
        ("GET", path) if path.starts_with("/families/") => get_family(...),
        // ... other routes
        _ => Err(HandlerError::NotFound("Route not found".to_string())),
    };
    
    // Convert to HttpResponse with CORS
    let response = HttpResponse::from_handler_result(handler_result, &cors_config);
    
    Ok(Response {
        status_code: response.status,
        body: response.body,
        headers: response.headers,
        ..Default::default()
    })
}
```

## Response Utilities

The `HttpResponse` struct provides a unified response structure with status, body, and headers.

### Creating Responses

```rust
use famtrac_backend::utils::HttpResponse;

// Basic response
let response = HttpResponse::new(200, "OK".to_string());

// Response with CORS headers
let response = HttpResponse::with_cors(200, "OK".to_string(), &cors_config);

// Add custom headers
let mut response = HttpResponse::new(200, "OK".to_string());
response.add_header("X-Custom-Header".to_string(), "value".to_string());
```

### Converting Errors

```rust
use famtrac_backend::utils::HttpResponse;
use famtrac_backend::errors::HandlerError;

let error = HandlerError::Validation(ValidationError { ... });
let response = HttpResponse::from_error(error, &cors_config);

// Response includes:
// - Appropriate HTTP status code (400 for validation errors)
// - JSON error body with error code and message
// - CORS headers
```

## Testing

The CORS implementation includes comprehensive tests:

- Unit tests in `src/utils/cors.rs` and `src/utils/response.rs`
- Integration tests in `tests/cors_integration_test.rs`

Run tests with:

```bash
cargo test --lib utils::cors
cargo test --lib utils::response
cargo test --test cors_integration_test
```

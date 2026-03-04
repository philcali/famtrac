# Implementation Plan: Router Module Refactoring

## Overview

This plan refactors the routing logic from `src/main.rs` into a dedicated `src/router/` module. The refactoring extracts approximately 200 lines of route matching and parameter extraction logic into a well-organized module structure that mirrors the existing `src/handlers/` organization. This is a pure refactoring with no behavior changes.

## Tasks

- [x] 1. Create router module structure
  - Create `src/router/` directory
  - Create `src/router/mod.rs` with module declarations for family, dependent, activity, and extractors submodules
  - Create empty submodules: `family.rs`, `dependent.rs`, `activity.rs`, `extractors.rs`
  - Add `pub mod router;` to `src/lib.rs` or appropriate location
  - _Requirements: 1.1, 1.2, 1.3, 1.5_

- [x] 2. Implement path parameter extractors
  - [x] 2.1 Implement extract_path_param function
    - Copy `extract_path_param()` from main.rs to `router/extractors.rs`
    - Make function public and add documentation
    - _Requirements: 6.1, 6.4_
  
  - [x] 2.2 Implement extract_uuid_param function
    - Implement `extract_uuid_param()` that uses `extract_path_param()` and parses UUID
    - Return `HandlerError::Validation` with field name if UUID parsing fails
    - Add documentation explaining error handling
    - _Requirements: 6.2, 6.3_
  
  - [x] 2.3 Write unit tests for extractors
    - Test `extract_path_param()` with various path/prefix combinations
    - Test `extract_uuid_param()` with valid and invalid UUIDs
    - Test error messages for invalid UUIDs
    - _Requirements: 6.4, 9.4_

- [x] 3. Implement family route handlers
  - [x] 3.1 Create route_family function
    - Implement `router/family.rs::route_family()` with signature from design
    - Handle POST /families → call `handlers::create_family()`
    - Handle GET /families/{id} → extract UUID and call `handlers::get_family()`
    - Handle PUT /families/{id} → extract UUID and call `handlers::update_family()`
    - Handle GET /families/{id}/dependents → extract UUID and call `handlers::list_dependents()`
    - Use `extractors::extract_uuid_param()` for UUID extraction
    - Return appropriate errors for invalid UUIDs and unknown routes
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 6.5_
  
  - [x] 3.2 Write unit tests for family routes
    - Test each route with valid inputs
    - Test UUID validation errors
    - Verify correct handler functions are called
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 3.6_

- [x] 4. Implement dependent route handlers
  - [x] 4.1 Create route_dependent function
    - Implement `router/dependent.rs::route_dependent()` with signature from design
    - Handle POST /dependents → call `handlers::create_dependent()`
    - Handle GET /dependents/{id} → extract UUID and call `handlers::get_dependent()`
    - Handle PUT /dependents/{id} → extract UUID and call `handlers::update_dependent()`
    - Use `extractors::extract_uuid_param()` for UUID extraction
    - Return appropriate errors for invalid UUIDs
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.6, 6.5_
  
  - [x] 4.2 Write unit tests for dependent routes
    - Test each route with valid inputs
    - Test UUID validation errors
    - Verify correct handler functions are called
    - _Requirements: 4.2, 4.3, 4.4, 4.6_

- [x] 5. Implement activity route handlers
  - [x] 5.1 Create route_activity function
    - Implement `router/activity.rs::route_activity()` with signature from design
    - Handle POST /activities → call `handlers::create_activity()`
    - Handle GET /activities/{id} → extract UUID and call `handlers::get_activity()`
    - Handle PUT /activities/{id} → extract UUID and call `handlers::update_activity()`
    - Handle DELETE /activities/{id} → extract UUID and call `handlers::delete_activity()`
    - Handle GET /dependents/{id}/activities → extract UUID, query params, call `handlers::query_activities()`
    - Use `extractors::extract_uuid_param()` for UUID extraction
    - Return appropriate errors for invalid UUIDs
    - _Requirements: 4.5, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.5_
  
  - [x] 5.2 Write unit tests for activity routes
    - Test each route with valid inputs
    - Test UUID validation errors
    - Test query parameter extraction for activities query
    - Verify correct handler functions are called
    - _Requirements: 5.2, 5.3, 5.4, 5.5, 5.6_

- [x] 6. Implement main router function
  - [x] 6.1 Create route_request function in router/mod.rs
    - Implement `router::route_request()` with signature from design
    - Extract HTTP method, path, and body from ApiGatewayProxyRequest
    - Add logging statement: "Routing: {method} {path}"
    - Match on path prefix to delegate to family, dependent, or activity route handlers
    - Return `HandlerError::NotFound` with method and path for unknown routes
    - Convert handler results to HttpResponse using `HttpResponse::from_handler_result()` with CORS config
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 7.4, 7.5_
  
  - [x] 6.2 Export route_request and submodules from router/mod.rs
    - Add `pub use` statements for route_request function
    - Re-export public functions from submodules as needed
    - _Requirements: 1.4_
  
  - [ ]* 6.3 Write unit tests for route_request
    - Test routing to each submodule (family, dependent, activity)
    - Test NotFound error for unknown routes
    - Test logging output
    - Test CORS header application
    - _Requirements: 2.4, 2.5, 7.4, 7.5_

- [x] 7. Checkpoint - Verify router module compiles and tests pass
  - Ensure all router module code compiles without errors
  - Ensure all unit tests pass
  - Ask the user if questions arise

- [x] 8. Update main.rs to use router module
  - [x] 8.1 Import and use router::route_request
    - Add `use crate::router;` or appropriate import in main.rs
    - Replace the call to inline `route_request()` with `router::route_request()`
    - Pass all required parameters (request, context, repositories, cors_config)
    - _Requirements: 2.6, 8.1_
  
  - [x] 8.2 Remove old routing code from main.rs
    - Delete the old `route_request()` function from main.rs
    - Delete the old `extract_path_param()` function from main.rs
    - Verify main.rs only contains Lambda setup code: main(), handle_request(), create_options_response(), to_api_gateway_response()
    - _Requirements: 8.2, 8.3, 8.4_
  
  - [ ]* 8.3 Verify main.rs line count reduction
    - Check that main.rs has been reduced by approximately 200 lines
    - _Requirements: 8.4_

- [ ] 9. Run integration tests
  - Run all existing integration tests to verify behavior preservation
  - Ensure no integration tests need modification
  - _Requirements: 7.1, 7.2, 7.3, 9.5_

- [ ]* 10. Write property-based tests for behavior preservation
  - [ ]* 10.1 Write property test for routing behavior preservation
    - **Property 1: Routing Behavior Preservation**
    - **Validates: Requirements 2.4, 3.3, 3.4, 3.5, 4.3, 4.4, 4.5, 5.3, 5.4, 5.5, 7.1, 7.3**
    - Generate random valid HTTP method and path combinations
    - Verify refactored router produces identical HttpResponse (status, body, headers)
    - Use proptest with minimum 100 iterations
  
  - [ ]* 10.2 Write property test for error handling preservation
    - **Property 2: Error Handling Preservation**
    - **Validates: Requirements 2.5, 3.6, 4.6, 5.6, 6.3, 7.2**
    - Generate random invalid inputs (bad UUIDs, unknown routes)
    - Verify refactored router returns identical HandlerError types and messages
    - Use proptest with minimum 100 iterations
  
  - [ ]* 10.3 Write property test for CORS header preservation
    - **Property 3: CORS Header Preservation**
    - **Validates: Requirements 7.5**
    - Generate random handler results (success and error cases)
    - Verify CORS headers are applied identically through HttpResponse::from_handler_result()
    - Use proptest with minimum 100 iterations
  
  - [ ]* 10.4 Write property test for path parameter extraction equivalence
    - **Property 4: Path Parameter Extraction Equivalence**
    - **Validates: Requirements 6.4**
    - Generate random path and prefix combinations
    - Verify extract_path_param() returns same results as original implementation
    - Use proptest with minimum 100 iterations

- [ ] 11. Final verification and cleanup
  - [ ] 11.1 Run all tests
    - Run unit tests for router module
    - Run integration tests
    - Run property-based tests (if implemented)
    - Verify all tests pass
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_
  
  - [ ] 11.2 Add documentation
    - Add module-level documentation to router/mod.rs explaining purpose
    - Add function documentation to all public functions
    - Document error handling behavior
    - _Requirements: 8.5_
  
  - [ ] 11.3 Final checkpoint
    - Verify main.rs focuses only on Lambda setup
    - Verify router module is independently testable
    - Verify no behavior changes in manual testing
    - Ensure all tests pass, ask the user if questions arise

## Notes

- Tasks marked with `*` are optional and can be skipped for faster completion
- This is a pure refactoring - no behavior changes allowed
- All existing integration tests must pass without modification
- Property-based tests provide strong guarantees of behavior preservation
- The router module should mirror the structure of the handlers module
- Focus on preserving exact behavior including error messages and CORS headers

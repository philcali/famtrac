# Implementation Plan: famtrac-backend

## Overview

This implementation plan breaks down the famtrac-backend REST API into discrete coding tasks. The API is a Rust binary deployed on AWS Lambda, using DynamoDB for persistence and AWS Cognito for authentication. The implementation follows a layered architecture: domain types → repository layer → handler layer → integration.

The plan prioritizes incremental validation through property-based tests using proptest, with each property test marked as optional to allow for faster MVP delivery while maintaining the option for comprehensive correctness validation.

## Tasks

- [x] 1. Set up project structure and core domain types
  - Initialize Rust project with cargo
  - Add dependencies: aws_lambda_events, lambda_runtime, serde, serde_json, uuid, chrono, aws-sdk-dynamodb
  - Add dev dependencies: proptest, tokio-test
  - Create module structure: domain, repository, handlers, errors, utils
  - Define core domain types: Family, Dependent, Activity, ActivityType, FeedingType, DiaperContents
  - Define ID types: FamilyId, DependentId, ActivityId, IdentityId
  - Define Timestamp and Date types with ISO 8601 serialization
  - _Requirements: 1.1, 2.1, 3.1, 3.2, 10.5_

- [ ]* 1.1 Write property test for JSON serialization round-trip
  - **Property 22: JSON Serialization Round-Trip**
  - **Validates: Requirements 10.1, 10.3, 10.4**
  - Implement Arbitrary instances for all domain types
  - Test that serializing and deserializing produces equivalent objects

- [x] 2. Implement error handling and validation
  - Define error types: HandlerError, StoreError, ValidationError, AuthError
  - Implement error-to-HTTP status code mapping
  - Implement JSON error response structure with code, message, and details fields
  - Create validation functions for Family name, Dependent name and date_of_birth
  - Create validation functions for Activity timestamp and type-specific attributes
  - Implement input sanitization for string fields
  - _Requirements: 1.2, 2.2, 3.3, 3.4, 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2, 8.3, 8.4, 9.1, 9.2, 9.3_

- [ ]* 2.1 Write property test for invalid input rejection
  - **Property 3: Invalid Input Rejection**
  - **Validates: Requirements 1.2, 2.2, 3.3, 3.4, 5.2, 7.5, 8.2, 8.3**
  - Generate invalid inputs (empty names, long names, future dates)
  - Verify all return 400 Bad Request with descriptive messages

- [ ]* 2.2 Write property test for error response format consistency
  - **Property 19: Error Response Format Consistency**
  - **Validates: Requirements 9.1, 9.2**
  - Generate various error conditions
  - Verify all errors return JSON with error.code and error.message fields
  - Verify HTTP status codes match error types

- [ ]* 2.3 Write property test for input sanitization
  - **Property 18: Input Sanitization**
  - **Validates: Requirements 8.4**
  - Generate inputs with potentially dangerous characters
  - Verify sanitization prevents injection while preserving legitimate content

- [ ]* 2.4 Write unit tests for type-specific validation
  - Test feeding validation (feeding_type required, volume_ml > 0 if present)
  - Test diaper change validation (contents required)
  - Test sleep validation (start and end required, end > start)
  - Test pumping validation (volume_ml required and > 0)
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 3. Implement DynamoDB repository layer
  - Define repository traits: FamilyRepository, DependentRepository, ActivityRepository
  - Implement DynamoDB single-table design with PK/SK structure
  - Implement FamilyRepository: create, get, update, get_by_owner
  - Implement DependentRepository: create, get, update, list_by_family
  - Implement ActivityRepository: create, get, update, delete, query with ActivityQueryParams
  - Implement DynamoDB item serialization/deserialization for all domain types
  - Configure GSI-1 for owner_id lookups
  - _Requirements: 1.1, 1.3, 1.4, 2.1, 2.3, 2.4, 2.5, 3.1, 4.1, 4.2, 4.3, 5.1, 6.1, 6.3_

- [x] 3.1 Write property test for resource creation round-trip
  - **Property 1: Resource Creation Round-Trip**
  - **Validates: Requirements 1.1, 1.3, 2.1, 2.3, 3.1**
  - Use DynamoDB Local for integration testing
  - Generate valid Family, Dependent, and Activity instances
  - Verify create then get returns equivalent resource

- [ ]* 3.2 Write property test for resource update persistence
  - **Property 2: Resource Update Persistence**
  - **Validates: Requirements 1.4, 2.4, 5.1**
  - Use DynamoDB Local for integration testing
  - Generate existing resources and valid update data
  - Verify update then get reflects updated values

- [ ]* 3.3 Write property test for dependent listing completeness
  - **Property 5: Dependent Listing Completeness**
  - **Validates: Requirements 2.5**
  - Use DynamoDB Local for integration testing
  - Create Family with multiple Dependents
  - Verify list_by_family returns all and only those dependents

- [ ]* 3.4 Write property test for activity query date range filtering
  - **Property 7: Activity Query Date Range Filtering**
  - **Validates: Requirements 4.1**
  - Use DynamoDB Local for integration testing
  - Create activities with various timestamps
  - Verify query returns only activities within specified range

- [ ]* 3.5 Write property test for activity query type filtering
  - **Property 8: Activity Query Type Filtering**
  - **Validates: Requirements 4.2**
  - Use DynamoDB Local for integration testing
  - Create activities of different types
  - Verify query with type filter returns only matching activities

- [ ]* 3.6 Write property test for activity query sorting
  - **Property 9: Activity Query Sorting**
  - **Validates: Requirements 4.3**
  - Use DynamoDB Local for integration testing
  - Create activities with random timestamps
  - Verify query results are sorted descending by timestamp

- [ ]* 3.7 Write property test for activity deletion completeness
  - **Property 12: Activity Deletion Completeness**
  - **Validates: Requirements 6.1, 6.3**
  - Use DynamoDB Local for integration testing
  - Create and delete activity
  - Verify get returns 404 and activity not in query results

- [ ]* 3.8 Write property test for created timestamp immutability
  - **Property 11: Created Timestamp Immutability**
  - **Validates: Requirements 5.4**
  - Use DynamoDB Local for integration testing
  - Create activity, update it, verify created_at unchanged

- [x] 3.9 Implement DynamoDB Local test utilities and setup script
  - Create `scripts/setup-dynamodb-local.sh` bash script that downloads and validates DynamoDB Local JAR with SHA256 checksum
  - Create test utilities module in `famtrac-backend/tests/common/mod.rs` with:
    - Function to check for `dynamodb/DynamoDBLocal.jar` presence
    - Function to spawn DynamoDB Local process on random available port
    - Function to create test table with proper schema (PK, SK, GSI-1)
    - Setup and teardown helpers for property tests
    - Process cleanup on test completion
  - Update property test in `property_resource_creation_roundtrip.rs` to use test utilities
  - Remove `#[ignore]` attribute from tests once utilities are working
  - Add `dynamodb/` directory to `.gitignore`

- [x] 4. Checkpoint - Ensure repository layer tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement request context and identity extraction
  - Define RequestContext struct with identity_id field
  - Implement identity extraction from API Gateway request context
  - Implement identity validation (return 401 if missing)
  - _Requirements: 12.1, 12.2, 12.4_

- [ ]* 5.1 Write property test for identity extraction
  - **Property 25: Identity Extraction**
  - **Validates: Requirements 12.1, 12.2**
  - Generate requests with identity in context
  - Verify identity_id is successfully extracted

- [ ]* 5.2 Write property test for missing identity rejection
  - **Property 26: Missing Identity Rejection**
  - **Validates: Requirements 12.4**
  - Generate requests without identity in context
  - Verify 401 Unauthorized is returned

- [x] 6. Implement authorization logic
  - Define Authorizable trait for hierarchical authorization
  - Implement authorization for Family access (verify owner_id matches identity)
  - Implement authorization for Dependent access (verify identity owns parent Family)
  - Implement authorization for Activity access (verify identity owns parent Family via Dependent)
  - Return 403 Forbidden for unauthorized access
  - _Requirements: 1.5, 2.6, 3.5, 4.6, 5.5, 6.4, 12.3, 12.5_

- [ ]* 6.1 Write property test for identity-based authorization
  - **Property 4: Identity-Based Authorization**
  - **Validates: Requirements 1.5, 2.6, 3.5, 4.6, 5.5, 6.4, 12.3, 12.5**
  - Generate resources with different owner identities
  - Verify identity can only access resources they own
  - Verify unauthorized access returns 403 Forbidden

- [ ] 7. Implement Family handlers
  - [x] 7.1 Implement POST /families handler
    - Parse CreateFamilyRequest from JSON body
    - Extract identity from request context
    - Validate family name
    - Create Family with identity as owner_id
    - Return 201 Created with Family JSON
    - _Requirements: 1.1, 1.2, 8.1, 10.1_
  
  - [x] 7.2 Implement GET /families/{family_id} handler
    - Extract family_id from path parameters
    - Extract identity from request context
    - Retrieve Family from repository
    - Authorize identity has access to Family
    - Return 200 OK with Family JSON or 404 Not Found
    - _Requirements: 1.3, 1.5, 10.3_
  
  - [x] 7.3 Implement PUT /families/{family_id} handler
    - Extract family_id from path parameters
    - Parse UpdateFamilyRequest from JSON body
    - Extract identity from request context
    - Retrieve existing Family
    - Authorize identity has access to Family
    - Validate update data
    - Update Family in repository
    - Return 200 OK with updated Family JSON
    - _Requirements: 1.4, 1.5, 8.1, 10.1_

- [ ]* 7.4 Write unit tests for Family handlers
  - Test successful family creation
  - Test family retrieval with valid and invalid IDs
  - Test family update with valid and invalid data
  - Test authorization failures return 403
  - _Requirements: 1.1, 1.3, 1.4, 1.5_

- [x] 8. Implement Dependent handlers
  - [x] 8.1 Implement POST /dependents handler
    - Parse CreateDependentRequest from JSON body
    - Extract identity from request context
    - Validate dependent name and date_of_birth
    - Retrieve parent Family and authorize access
    - Create Dependent in repository
    - Return 201 Created with Dependent JSON
    - _Requirements: 2.1, 2.2, 2.6, 8.1, 10.1_
  
  - [x] 8.2 Implement GET /dependents/{dependent_id} handler
    - Extract dependent_id from path parameters
    - Extract identity from request context
    - Retrieve Dependent from repository
    - Authorize identity has access to parent Family
    - Return 200 OK with Dependent JSON or 404 Not Found
    - _Requirements: 2.3, 2.6, 10.3_
  
  - [x] 8.3 Implement PUT /dependents/{dependent_id} handler
    - Extract dependent_id from path parameters
    - Parse UpdateDependentRequest from JSON body
    - Extract identity from request context
    - Retrieve existing Dependent
    - Authorize identity has access to parent Family
    - Validate update data
    - Update Dependent in repository
    - Return 200 OK with updated Dependent JSON
    - _Requirements: 2.4, 2.6, 8.1, 10.1_
  
  - [x] 8.4 Implement GET /families/{family_id}/dependents handler
    - Extract family_id from path parameters
    - Extract identity from request context
    - Retrieve Family and authorize access
    - List all Dependents for Family from repository
    - Return 200 OK with array of Dependent JSON
    - _Requirements: 2.5, 2.6, 10.3_

- [ ]* 8.5 Write unit tests for Dependent handlers
  - Test successful dependent creation
  - Test dependent retrieval with valid and invalid IDs
  - Test dependent update with valid and invalid data
  - Test listing dependents by family
  - Test authorization failures return 403
  - _Requirements: 2.1, 2.3, 2.4, 2.5, 2.6_

- [x] 9. Implement Activity handlers
  - [x] 9.1 Implement POST /activities handler
    - Parse CreateActivityRequest from JSON body
    - Extract identity from request context
    - Validate activity timestamp (not in future)
    - Validate type-specific attributes
    - Retrieve parent Dependent and authorize access to parent Family
    - Create Activity in repository
    - Return 201 Created with Activity JSON
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 7.1, 7.2, 7.3, 7.4, 8.1, 10.1_
  
  - [x] 9.2 Implement GET /activities/{activity_id} handler
    - Extract activity_id from path parameters
    - Extract identity from request context
    - Retrieve Activity from repository
    - Authorize identity has access to parent Family via Dependent
    - Return 200 OK with Activity JSON or 404 Not Found
    - _Requirements: 3.1, 10.3_
  
  - [x] 9.3 Implement PUT /activities/{activity_id} handler
    - Extract activity_id from path parameters
    - Parse UpdateActivityRequest from JSON body
    - Extract identity from request context
    - Retrieve existing Activity
    - Authorize identity has access to parent Family via Dependent
    - Validate update data
    - Update Activity in repository (preserve created_at)
    - Return 200 OK with updated Activity JSON
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 8.1, 10.1_
  
  - [x] 9.4 Implement DELETE /activities/{activity_id} handler
    - Extract activity_id from path parameters
    - Extract identity from request context
    - Retrieve Activity and authorize access to parent Family via Dependent
    - Delete Activity from repository
    - Return 204 No Content or 404 Not Found
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  
  - [x] 9.5 Implement GET /dependents/{dependent_id}/activities handler
    - Extract dependent_id from path parameters
    - Parse query parameters: start_date, end_date, activity_type
    - Extract identity from request context
    - Retrieve Dependent and authorize access to parent Family
    - Validate date range (end_date not before start_date)
    - Query activities from repository with filters
    - Return 200 OK with array of Activity JSON sorted by timestamp descending
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 10.3_

- [ ]* 9.6 Write property test for activity type support
  - **Property 6: Activity Type Support**
  - **Validates: Requirements 3.2**
  - Create activities of all four types (Feeding, DiaperChange, Sleep, Pumping)
  - Verify all types are successfully created and retrieved

- [ ]* 9.7 Write property test for invalid date range rejection
  - **Property 10: Invalid Date Range Rejection**
  - **Validates: Requirements 4.5**
  - Generate invalid date ranges (end before start, malformed dates)
  - Verify 400 Bad Request is returned

- [ ]* 9.8 Write property test for feeding activity validation
  - **Property 13: Feeding Activity Validation**
  - **Validates: Requirements 7.1**
  - Generate feeding activities with various valid and invalid attributes
  - Verify feeding_type is required and volume_ml > 0 if present

- [ ]* 9.9 Write property test for diaper change activity validation
  - **Property 14: Diaper Change Activity Validation**
  - **Validates: Requirements 7.2**
  - Generate diaper change activities with various valid and invalid attributes
  - Verify contents is required and valid enum value

- [ ]* 9.10 Write property test for sleep activity validation
  - **Property 15: Sleep Activity Validation**
  - **Validates: Requirements 7.3**
  - Generate sleep activities with various valid and invalid attributes
  - Verify start and end are required and end > start

- [ ]* 9.11 Write property test for pumping activity validation
  - **Property 16: Pumping Activity Validation**
  - **Validates: Requirements 7.4**
  - Generate pumping activities with various valid and invalid attributes
  - Verify volume_ml is required and > 0

- [ ]* 9.12 Write unit tests for Activity handlers
  - Test successful activity creation for all types
  - Test activity retrieval with valid and invalid IDs
  - Test activity update preserves created_at
  - Test activity deletion
  - Test activity query with date range and type filters
  - Test empty query results
  - Test authorization failures return 403
  - _Requirements: 3.1, 3.2, 4.1, 4.2, 4.3, 4.4, 5.1, 5.4, 6.1_

- [ ] 10. Checkpoint - Ensure handler tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 11. Implement CORS support
  - Add CORS headers to all responses: Access-Control-Allow-Origin, Access-Control-Allow-Methods, Access-Control-Allow-Headers
  - Implement OPTIONS handler for preflight requests
  - _Requirements: 11.1, 11.2, 11.3, 11.4_

- [ ]* 11.1 Write property test for CORS headers presence
  - **Property 24: CORS Headers Presence**
  - **Validates: Requirements 11.2, 11.3, 11.4**
  - Generate various API requests
  - Verify all responses include required CORS headers

- [ ] 12. Implement Lambda entry point and routing
  - Create main Lambda handler function
  - Implement HTTP method and path routing to handlers
  - Wire request context extraction
  - Wire error handling and logging
  - Configure DynamoDB client with table name from environment variable
  - _Requirements: 9.4, 12.1_

- [ ]* 12.1 Write property test for malformed JSON rejection
  - **Property 17: Malformed JSON Rejection**
  - **Validates: Requirements 8.1, 10.2**
  - Generate requests with malformed JSON bodies
  - Verify 400 Bad Request with descriptive message

- [ ]* 12.2 Write property test for internal error opacity
  - **Property 20: Internal Error Opacity**
  - **Validates: Requirements 9.3**
  - Simulate internal errors (database failures, panics)
  - Verify 500 responses don't expose internal details

- [ ]* 12.3 Write property test for error logging
  - **Property 21: Error Logging**
  - **Validates: Requirements 9.4**
  - Generate various error conditions
  - Verify errors are logged with sufficient context

- [ ]* 12.4 Write property test for ISO 8601 timestamp format
  - **Property 23: ISO 8601 Timestamp Format**
  - **Validates: Requirements 10.5**
  - Generate resources with timestamp fields
  - Verify JSON representation uses ISO 8601 format

- [ ]* 12.5 Write integration tests for end-to-end flows
  - Test complete request/response cycles through Lambda handler
  - Test authentication flow with identity context
  - Test CORS preflight and actual requests
  - Use DynamoDB Local for full integration testing
  - _Requirements: 11.1, 12.1, 12.2_

- [ ] 13. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP delivery
- Each task references specific requirements for traceability
- Property-based tests use proptest with minimum 100 iterations per property
- Integration tests use DynamoDB Local (not mocked) for high-fidelity validation
- All 26 correctness properties from the design document are covered by optional property tests
- Checkpoints ensure incremental validation at reasonable breaks
- The Lambda function expects a DynamoDB table name in an environment variable
- AWS SDK configuration (region, credentials) is handled by Lambda runtime environment
- **IMPORTANT**: Always read files before editing them, as they may have changed between tasks due to linting (cargo fmt, cargo clippy), auto-formatting, or other modifications

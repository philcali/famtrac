# Requirements Document

## Introduction

This document specifies requirements for refactoring the routing logic in famtrac-backend from main.rs into a dedicated router module. The refactoring will improve separation of concerns by moving the ~200 line route_request() function and related utilities into a new src/router/ module that mirrors the existing src/handlers/ structure. This is a pure refactoring with no behavior changes.

## Glossary

- **Router_Module**: The new src/router/ module containing routing logic
- **Route_Request_Function**: The main routing function that matches HTTP method and path to handler functions
- **Path_Extractor**: Utility function that extracts UUID parameters from URL paths
- **Route_Handler**: Thin wrapper function in router module that extracts parameters and delegates to handler functions
- **Handler_Function**: Existing business logic functions in src/handlers/ module
- **Main_Module**: The src/main.rs file that currently contains routing logic
- **Lambda_Setup**: Code in main.rs responsible for AWS Lambda initialization and request/response conversion

## Requirements

### Requirement 1: Create Router Module Structure

**User Story:** As a developer, I want the router module to mirror the handlers module structure, so that the codebase has consistent organization.

#### Acceptance Criteria

1. THE Router_Module SHALL contain a mod.rs file that exports the Route_Request_Function
2. THE Router_Module SHALL contain family.rs, dependent.rs, and activity.rs submodules
3. THE Router_Module SHALL contain an extractors.rs module for path parameter extraction utilities
4. THE mod.rs file SHALL re-export all public route handler functions from the submodules
5. FOR ALL submodules in src/handlers/, THE Router_Module SHALL have a corresponding submodule with the same name

### Requirement 2: Extract Route Request Function

**User Story:** As a developer, I want the route_request() function moved to the router module, so that main.rs focuses only on Lambda setup.

#### Acceptance Criteria

1. THE Router_Module SHALL export a route_request() function with the same signature as the current function in main.rs
2. THE Route_Request_Function SHALL accept ApiGatewayProxyRequest, RequestContext, repository references, and CorsConfig as parameters
3. THE Route_Request_Function SHALL return an HttpResponse
4. THE Route_Request_Function SHALL match HTTP method and path combinations to delegate to appropriate Route_Handler functions
5. WHEN an unknown route is requested, THE Route_Request_Function SHALL return a HandlerError::NotFound with the method and path
6. THE Main_Module SHALL call router::route_request() instead of containing the routing logic inline

### Requirement 3: Implement Family Route Handlers

**User Story:** As a developer, I want family routing logic in router/family.rs, so that family routes are organized separately.

#### Acceptance Criteria

1. THE family.rs submodule SHALL contain a route_family() function that handles all /families/* routes
2. WHEN the path is "/families" and method is POST, THE Route_Handler SHALL extract body and call handlers::create_family()
3. WHEN the path matches "/families/{id}" and method is GET, THE Route_Handler SHALL extract the family_id UUID and call handlers::get_family()
4. WHEN the path matches "/families/{id}" and method is PUT, THE Route_Handler SHALL extract the family_id UUID and call handlers::update_family()
5. WHEN the path matches "/families/{id}/dependents" and method is GET, THE Route_Handler SHALL extract the family_id UUID and call handlers::list_dependents()
6. IF the family_id cannot be parsed as a valid UUID, THEN THE Route_Handler SHALL return a HandlerError::Validation with field "family_id"

### Requirement 4: Implement Dependent Route Handlers

**User Story:** As a developer, I want dependent routing logic in router/dependent.rs, so that dependent routes are organized separately.

#### Acceptance Criteria

1. THE dependent.rs submodule SHALL contain a route_dependent() function that handles all /dependents/* routes
2. WHEN the path is "/dependents" and method is POST, THE Route_Handler SHALL extract body and call handlers::create_dependent()
3. WHEN the path matches "/dependents/{id}" and method is GET, THE Route_Handler SHALL extract the dependent_id UUID and call handlers::get_dependent()
4. WHEN the path matches "/dependents/{id}" and method is PUT, THE Route_Handler SHALL extract the dependent_id UUID and call handlers::update_dependent()
5. WHEN the path matches "/dependents/{id}/activities" and method is GET, THE Route_Handler SHALL extract dependent_id, query parameters (start_date, end_date, activity_type) and call handlers::query_activities()
6. IF the dependent_id cannot be parsed as a valid UUID, THEN THE Route_Handler SHALL return a HandlerError::Validation with field "dependent_id"

### Requirement 5: Implement Activity Route Handlers

**User Story:** As a developer, I want activity routing logic in router/activity.rs, so that activity routes are organized separately.

#### Acceptance Criteria

1. THE activity.rs submodule SHALL contain a route_activity() function that handles all /activities/* routes
2. WHEN the path is "/activities" and method is POST, THE Route_Handler SHALL extract body and call handlers::create_activity()
3. WHEN the path matches "/activities/{id}" and method is GET, THE Route_Handler SHALL extract the activity_id UUID and call handlers::get_activity()
4. WHEN the path matches "/activities/{id}" and method is PUT, THE Route_Handler SHALL extract the activity_id UUID and call handlers::update_activity()
5. WHEN the path matches "/activities/{id}" and method is DELETE, THE Route_Handler SHALL extract the activity_id UUID and call handlers::delete_activity()
6. IF the activity_id cannot be parsed as a valid UUID, THEN THE Route_Handler SHALL return a HandlerError::Validation with field "activity_id"

### Requirement 6: Implement Path Parameter Extractors

**User Story:** As a developer, I want reusable path parameter extraction utilities, so that route handlers can cleanly extract and validate path parameters.

#### Acceptance Criteria

1. THE extractors.rs module SHALL contain an extract_path_param() function that extracts a string parameter from a path given a prefix
2. THE extractors.rs module SHALL contain an extract_uuid_param() function that extracts and parses a UUID from a path
3. WHEN extract_uuid_param() receives an invalid UUID, THE Path_Extractor SHALL return a HandlerError::Validation with the appropriate field name
4. THE extract_path_param() function SHALL have the same behavior as the current function in main.rs
5. FOR ALL Route_Handler functions, THE implementation SHALL use extractors from extractors.rs instead of inline extraction logic

### Requirement 7: Preserve Routing Behavior

**User Story:** As a developer, I want the refactoring to preserve all existing routing behavior, so that no functionality is broken.

#### Acceptance Criteria

1. FOR ALL route patterns in the current route_request() function, THE Router_Module SHALL handle the same patterns identically
2. FOR ALL error cases in the current routing logic, THE Router_Module SHALL return the same error types and messages
3. FOR ALL successful routes, THE Router_Module SHALL call the same Handler_Function with the same parameters
4. THE Router_Module SHALL preserve the logging statement "Routing: {method} {path}"
5. THE Router_Module SHALL preserve CORS header handling through HttpResponse::from_handler_result()

### Requirement 8: Maintain Main Module Simplicity

**User Story:** As a developer, I want main.rs to focus on Lambda setup, so that the entry point is simple and clear.

#### Acceptance Criteria

1. THE Main_Module SHALL import and use router::route_request() instead of defining routing logic
2. THE Main_Module SHALL retain only Lambda_Setup code: main(), handle_request(), create_options_response(), and to_api_gateway_response()
3. THE Main_Module SHALL NOT contain any path matching or parameter extraction logic
4. THE Main_Module SHALL be reduced by approximately 200 lines after the refactoring
5. THE Main_Module SHALL maintain the same public API and Lambda handler signature

### Requirement 9: Enable Router Testing

**User Story:** As a developer, I want the router module to be independently testable, so that routing logic can be verified without Lambda infrastructure.

#### Acceptance Criteria

1. THE Route_Request_Function SHALL be a pure function that depends only on its parameters
2. THE Router_Module SHALL NOT depend on Lambda-specific types except ApiGatewayProxyRequest
3. THE Route_Handler functions SHALL be unit testable by providing mock repositories
4. THE Path_Extractor functions SHALL be unit testable with sample path strings
5. WHERE integration tests exist for routing, THE tests SHALL continue to pass after the refactoring

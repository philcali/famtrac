# Requirements Document

## Introduction

The famtrac-frontend is a single-page web application (SPA) that provides a user interface for managing families, dependents, and their activities. The application integrates with the existing famtrac-backend REST API and uses AWS Cognito for authentication. Users can create and manage family records, add dependents to families, and track various activities (feeding, diaper changes, sleep, pumping) for each dependent.

## Glossary

- **Frontend_Application**: The single-page web application that runs in the user's browser
- **Backend_API**: The existing famtrac-backend REST API service
- **Cognito_Service**: AWS Cognito authentication service
- **User**: An authenticated person using the application
- **Family**: A family entity with a name and owner
- **Dependent**: A child or person being tracked, associated with a family
- **Activity**: A tracked event for a dependent (feeding, diaper change, sleep, pumping)
- **Authentication_Token**: JWT token obtained from Cognito for API authorization
- **API_Client**: Component responsible for HTTP communication with Backend_API
- **Router**: Component that manages navigation between application views
- **Form_Validator**: Component that validates user input before submission

## Requirements

### Requirement 1: User Authentication

**User Story:** As a user, I want to authenticate using AWS Cognito, so that I can securely access my family data

#### Acceptance Criteria

1. THE Frontend_Application SHALL integrate with Cognito_Service for user authentication
2. WHEN a User accesses the application without an Authentication_Token, THE Frontend_Application SHALL redirect to the Cognito login page
3. WHEN Cognito_Service returns an Authentication_Token, THE Frontend_Application SHALL store it securely
4. THE Frontend_Application SHALL include the Authentication_Token in all Backend_API requests
5. WHEN an Authentication_Token expires, THE Frontend_Application SHALL redirect the User to re-authenticate
6. THE Frontend_Application SHALL provide a logout function that clears the Authentication_Token

### Requirement 2: Family Management - Create

**User Story:** As a user, I want to create new families, so that I can organize my dependents

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a form for creating a Family
2. THE Form_Validator SHALL require a family name with minimum 1 character
3. WHEN a User submits a valid family creation form, THE API_Client SHALL send a POST request to /families
4. WHEN Backend_API returns success, THE Frontend_Application SHALL display the newly created Family
5. WHEN Backend_API returns an error, THE Frontend_Application SHALL display the error message to the User
6. THE Frontend_Application SHALL display a loading indicator while the request is in progress

### Requirement 3: Family Management - View

**User Story:** As a user, I want to view my families, so that I can see what families I manage

#### Acceptance Criteria

1. THE Frontend_Application SHALL display a list of all families owned by the User
2. WHEN a User selects a Family, THE API_Client SHALL send a GET request to /families/{id}
3. WHEN Backend_API returns family data, THE Frontend_Application SHALL display the family name and associated dependents
4. THE Frontend_Application SHALL display creation and update timestamps for each Family
5. WHEN Backend_API returns a 404 error, THE Frontend_Application SHALL display a "Family not found" message

### Requirement 4: Family Management - Update

**User Story:** As a user, I want to update family information, so that I can correct or change family names

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a form for editing an existing Family
2. THE Form_Validator SHALL require a family name with minimum 1 character
3. WHEN a User submits a valid family update form, THE API_Client SHALL send a PUT request to /families/{id}
4. WHEN Backend_API returns success, THE Frontend_Application SHALL display the updated Family information
5. WHEN Backend_API returns an error, THE Frontend_Application SHALL display the error message to the User

### Requirement 5: Family Management - Delete

**User Story:** As a user, I want to delete families, so that I can remove families I no longer need

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a delete action for each Family
2. WHEN a User initiates family deletion, THE Frontend_Application SHALL display a confirmation dialog
3. WHEN a User confirms deletion, THE API_Client SHALL send a DELETE request to /families/{id}
4. WHEN Backend_API returns success, THE Frontend_Application SHALL remove the Family from the display
5. WHEN Backend_API returns an error indicating the family has dependents, THE Frontend_Application SHALL display a message that dependents must be removed first

### Requirement 6: Dependent Management - Create

**User Story:** As a user, I want to add dependents to families, so that I can track individuals within each family

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a form for creating a Dependent within a Family
2. THE Form_Validator SHALL require a dependent name with minimum 1 character
3. THE Form_Validator SHALL require a valid date of birth in the past
4. THE Form_Validator SHALL require a valid family_id
5. WHEN a User submits a valid dependent creation form, THE API_Client SHALL send a POST request to /dependents
6. WHEN Backend_API returns success, THE Frontend_Application SHALL display the newly created Dependent
7. WHEN Backend_API returns an error, THE Frontend_Application SHALL display the error message to the User

### Requirement 7: Dependent Management - View

**User Story:** As a user, I want to view dependents, so that I can see who I'm tracking

#### Acceptance Criteria

1. WHEN a User views a Family, THE Frontend_Application SHALL display all associated dependents
2. WHEN a User selects a Dependent, THE API_Client SHALL send a GET request to /dependents/{id}
3. WHEN Backend_API returns dependent data, THE Frontend_Application SHALL display the dependent name, date of birth, and timestamps
4. THE Frontend_Application SHALL calculate and display the Dependent's age based on date of birth
5. WHEN Backend_API returns a 404 error, THE Frontend_Application SHALL display a "Dependent not found" message

### Requirement 8: Dependent Management - Update

**User Story:** As a user, I want to update dependent information, so that I can correct or change dependent details

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a form for editing an existing Dependent
2. THE Form_Validator SHALL require a dependent name with minimum 1 character
3. THE Form_Validator SHALL require a valid date of birth in the past
4. WHEN a User submits a valid dependent update form, THE API_Client SHALL send a PUT request to /dependents/{id}
5. WHEN Backend_API returns success, THE Frontend_Application SHALL display the updated Dependent information
6. WHEN Backend_API returns an error, THE Frontend_Application SHALL display the error message to the User

### Requirement 9: Dependent Management - Delete

**User Story:** As a user, I want to delete dependents, so that I can remove dependents I no longer need to track

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a delete action for each Dependent
2. WHEN a User initiates dependent deletion, THE Frontend_Application SHALL display a confirmation dialog
3. WHEN a User confirms deletion, THE API_Client SHALL send a DELETE request to /dependents/{id}
4. WHEN Backend_API returns success, THE Frontend_Application SHALL remove the Dependent from the display
5. WHEN Backend_API returns an error, THE Frontend_Application SHALL display the error message to the User

### Requirement 10: Activity Management - Create

**User Story:** As a user, I want to record activities for dependents, so that I can track their daily care

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a form for creating an Activity for a Dependent
2. THE Form_Validator SHALL require a valid dependent_id
3. THE Form_Validator SHALL require a valid activity type (feeding, diaper_change, sleep, pumping)
4. THE Form_Validator SHALL require a timestamp not in the future
5. WHEN activity type is feeding, THE Form_Validator SHALL require a feeding_type (breast, bottle, solid)
6. WHEN activity type is diaper_change, THE Form_Validator SHALL require contents (wet, dirty, both)
7. WHEN activity type is sleep, THE Form_Validator SHALL require start and end timestamps where end is after start
8. WHEN activity type is pumping, THE Form_Validator SHALL require volume_ml as a positive integer
9. WHEN a User submits a valid activity creation form, THE API_Client SHALL send a POST request to /activities
10. WHEN Backend_API returns success, THE Frontend_Application SHALL display the newly created Activity
11. WHEN Backend_API returns an error, THE Frontend_Application SHALL display the error message to the User

### Requirement 11: Activity Management - View

**User Story:** As a user, I want to view activities for a dependent, so that I can see their care history

#### Acceptance Criteria

1. WHEN a User views a Dependent, THE Frontend_Application SHALL display recent activities
2. THE API_Client SHALL send a GET request to /dependents/{id}/activities to retrieve activities
3. THE Frontend_Application SHALL display activities in reverse chronological order (newest first)
4. THE Frontend_Application SHALL display the activity type, timestamp, and type-specific details for each Activity
5. THE Frontend_Application SHALL provide filtering by date range
6. THE Frontend_Application SHALL provide filtering by activity type
7. WHEN a User selects an Activity, THE API_Client SHALL send a GET request to /activities/{id}
8. WHEN Backend_API returns activity data, THE Frontend_Application SHALL display all activity details

### Requirement 12: Activity Management - Update

**User Story:** As a user, I want to update activities, so that I can correct mistakes in recorded data

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a form for editing an existing Activity
2. THE Form_Validator SHALL apply the same validation rules as activity creation
3. WHEN a User submits a valid activity update form, THE API_Client SHALL send a PUT request to /activities/{id}
4. WHEN Backend_API returns success, THE Frontend_Application SHALL display the updated Activity information
5. WHEN Backend_API returns an error, THE Frontend_Application SHALL display the error message to the User

### Requirement 13: Activity Management - Delete

**User Story:** As a user, I want to delete activities, so that I can remove incorrect or duplicate entries

#### Acceptance Criteria

1. THE Frontend_Application SHALL provide a delete action for each Activity
2. WHEN a User initiates activity deletion, THE Frontend_Application SHALL display a confirmation dialog
3. WHEN a User confirms deletion, THE API_Client SHALL send a DELETE request to /activities/{id}
4. WHEN Backend_API returns success, THE Frontend_Application SHALL remove the Activity from the display
5. WHEN Backend_API returns an error, THE Frontend_Application SHALL display the error message to the User

### Requirement 14: Navigation and Routing

**User Story:** As a user, I want to navigate between different views, so that I can access different features

#### Acceptance Criteria

1. THE Router SHALL support navigation to a families list view
2. THE Router SHALL support navigation to a family detail view with URL parameter for family_id
3. THE Router SHALL support navigation to a dependent detail view with URL parameter for dependent_id
4. THE Router SHALL maintain browser history for back/forward navigation
5. WHEN a User navigates to an invalid route, THE Frontend_Application SHALL display a "Page not found" message
6. THE Router SHALL preserve the current URL when redirecting to authentication

### Requirement 15: Error Handling

**User Story:** As a user, I want clear error messages, so that I understand what went wrong

#### Acceptance Criteria

1. WHEN Backend_API returns a 400 error, THE Frontend_Application SHALL display validation error messages
2. WHEN Backend_API returns a 401 error, THE Frontend_Application SHALL redirect to authentication
3. WHEN Backend_API returns a 403 error, THE Frontend_Application SHALL display an "Access denied" message
4. WHEN Backend_API returns a 404 error, THE Frontend_Application SHALL display a "Resource not found" message
5. WHEN Backend_API returns a 500 error, THE Frontend_Application SHALL display a "Server error" message
6. WHEN a network error occurs, THE Frontend_Application SHALL display a "Connection failed" message
7. THE Frontend_Application SHALL log all errors to the browser console for debugging

### Requirement 16: API Client Implementation

**User Story:** As a developer, I want a robust API client, so that the application reliably communicates with the backend

#### Acceptance Criteria

1. THE API_Client SHALL construct request URLs using a configurable base URL
2. THE API_Client SHALL include the Authentication_Token in the Authorization header for all requests
3. THE API_Client SHALL set Content-Type header to application/json for requests with body
4. THE API_Client SHALL parse JSON responses from Backend_API
5. WHEN Backend_API returns an error response, THE API_Client SHALL extract and return the error message
6. THE API_Client SHALL implement timeout handling with a configurable timeout value
7. WHEN a request times out, THE API_Client SHALL return a timeout error

### Requirement 17: Responsive Design

**User Story:** As a user, I want the application to work on different devices, so that I can use it on mobile or desktop

#### Acceptance Criteria

1. THE Frontend_Application SHALL display correctly on screen widths from 320px to 1920px
2. THE Frontend_Application SHALL use responsive layout techniques for forms and lists
3. WHEN screen width is below 768px, THE Frontend_Application SHALL use a mobile-optimized layout
4. THE Frontend_Application SHALL ensure touch targets are at least 44x44 pixels on mobile devices
5. THE Frontend_Application SHALL be usable with touch input on mobile devices

### Requirement 18: Data Validation and User Feedback

**User Story:** As a user, I want immediate feedback on form inputs, so that I can correct errors before submission

#### Acceptance Criteria

1. THE Form_Validator SHALL validate form fields on blur (when user leaves the field)
2. WHEN a field fails validation, THE Frontend_Application SHALL display an error message below the field
3. WHEN a field passes validation, THE Frontend_Application SHALL remove any error message
4. THE Frontend_Application SHALL disable submit buttons while validation errors exist
5. THE Frontend_Application SHALL display field-level validation errors in red text
6. THE Frontend_Application SHALL mark required fields with a visual indicator

### Requirement 19: Loading States and User Experience

**User Story:** As a user, I want to know when the application is working, so that I don't think it's frozen

#### Acceptance Criteria

1. WHEN an API request is in progress, THE Frontend_Application SHALL display a loading indicator
2. THE Frontend_Application SHALL disable form submit buttons while a request is in progress
3. WHEN data is being loaded, THE Frontend_Application SHALL display a skeleton or placeholder UI
4. THE Frontend_Application SHALL display success messages for create, update, and delete operations
5. THE Frontend_Application SHALL automatically dismiss success messages after 3 seconds

### Requirement 20: Configuration Management

**User Story:** As a developer, I want configurable settings, so that the application can work in different environments

#### Acceptance Criteria

1. THE Frontend_Application SHALL read the Backend_API base URL from configuration
2. THE Frontend_Application SHALL read the Cognito_Service configuration from environment variables
3. THE Frontend_Application SHALL support different configurations for development, staging, and production
4. THE Frontend_Application SHALL validate required configuration values at startup
5. WHEN required configuration is missing, THE Frontend_Application SHALL display an error message and prevent operation

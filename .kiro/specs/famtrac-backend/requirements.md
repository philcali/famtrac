# Requirements Document

## Introduction

The Family Activity Tracking (famtrac) backend is a REST API service that enables caregivers, parents, and extended family members to coordinate and track daily activities for dependents, with a primary focus on infant care. The system is designed with a minimal initial surface area to maximize future flexibility, organizing around three primary resources: Family, Dependent, and Activity.

The backend leverages AWS services for simplicity and scalability. Authentication and authorization are handled by AWS Cognito integrated with API Gateway authorizers, enabling multi-tenancy through authenticated identities. The API itself is implemented as a Rust binary deployed on AWS Lambda, exposing HTTP endpoints through API Gateway.

## Architectural Principles

- **Simplicity First**: Leverage AWS managed services where appropriate to reduce operational complexity
- **Minimal Surface Area**: Start with three core resources to maintain flexibility for future expansion
- **AWS-Native Authentication**: Cognito and API Gateway handle all auth concerns; the API trusts authenticated identities
- **Multi-Tenancy**: Tenant isolation is achieved through Cognito identity-based access control

## Glossary

- **API**: The famtrac backend REST API service
- **Family**: The top-level resource representing a family unit; the central coordination point for caregivers, parents, and extended family members
- **Dependent**: An individual being cared for within a Family (e.g., newborn, toddler, potentially pets in the future)
- **Activity**: A recorded event or journal entry representing noteworthy occurrences on a day-to-day basis (e.g., feeding, diaper change, sleep, pumping)
- **Caregiver**: A person with access to a Family resource (parent, extended family member, or other authorized individual)
- **Identity**: An authenticated user identity provided by AWS Cognito
- **Data_Store**: The persistent storage system for Family, Dependent, and Activity resources
- **Client**: The application or user making authenticated requests to the API

## Requirements

### Requirement 1: Manage Family Resources

**User Story:** As a caregiver, I want to create and manage a Family resource, so that I can coordinate care activities with other family members.

#### Acceptance Criteria

1. THE API SHALL create a new Family with a unique identifier and name
2. WHEN a Family creation request contains invalid data, THE API SHALL return a descriptive error message
3. THE API SHALL retrieve a Family by its unique identifier
4. THE API SHALL update Family information
5. WHEN an authenticated Identity requests a Family, THE API SHALL verify the Identity has access to that Family

### Requirement 2: Manage Dependent Resources

**User Story:** As a caregiver, I want to create and manage profiles for dependents within my Family, so that I can track activities for each dependent separately.

#### Acceptance Criteria

1. THE API SHALL create a new Dependent with a unique identifier, name, date of birth, and associated Family identifier
2. WHEN a Dependent creation request contains invalid data, THE API SHALL return a descriptive error message
3. THE API SHALL retrieve a Dependent by its unique identifier
4. THE API SHALL update Dependent information
5. THE API SHALL list all Dependents associated with a specific Family
6. WHEN an authenticated Identity requests a Dependent, THE API SHALL verify the Identity has access to the associated Family

### Requirement 3: Record Activity Resources

**User Story:** As a caregiver, I want to log activities for dependents, so that I can maintain an audit log and journal of noteworthy daily events.

#### Acceptance Criteria

1. WHEN an activity occurs, THE API SHALL create an Activity with a unique identifier, timestamp, Dependent identifier, activity type, and type-specific attributes
2. THE API SHALL support activity types including feeding, diaper change, sleep, and pumping
3. WHEN an Activity is created, THE API SHALL validate that the timestamp is not in the future
4. WHEN an Activity creation request contains invalid data, THE API SHALL return a descriptive error message
5. WHEN an authenticated Identity creates an Activity, THE API SHALL verify the Identity has access to the associated Dependent's Family

### Requirement 4: Query Activity Resources

**User Story:** As a caregiver, I want to retrieve activities by date range and type, so that I can review patterns and share information with healthcare providers.

#### Acceptance Criteria

1. WHEN a date range is specified, THE API SHALL retrieve all Activities for a Dependent within that range
2. WHERE an activity type filter is provided, THE API SHALL return only Activities matching that type
3. THE API SHALL return Activities sorted by timestamp in descending order
4. WHEN no activities match the query criteria, THE API SHALL return an empty list
5. WHEN a date range is invalid, THE API SHALL return a descriptive error message
6. WHEN an authenticated Identity queries Activities, THE API SHALL verify the Identity has access to the associated Dependent's Family

### Requirement 5: Update Activity Resources

**User Story:** As a caregiver, I want to correct mistakes in logged activities, so that my tracking data remains accurate.

#### Acceptance Criteria

1. THE API SHALL update an existing Activity by its unique identifier
2. WHEN an Activity update request contains invalid data, THE API SHALL return a descriptive error message
3. WHEN an Activity identifier does not exist, THE API SHALL return a not found error
4. THE API SHALL preserve the original creation timestamp when updating an Activity
5. WHEN an authenticated Identity updates an Activity, THE API SHALL verify the Identity has access to the associated Dependent's Family

### Requirement 6: Delete Activity Resources

**User Story:** As a caregiver, I want to remove incorrectly logged activities, so that my tracking data reflects only actual events.

#### Acceptance Criteria

1. THE API SHALL delete an Activity by its unique identifier
2. WHEN an Activity identifier does not exist, THE API SHALL return a not found error
3. WHEN an Activity is deleted, THE API SHALL remove it from the Data_Store permanently
4. WHEN an authenticated Identity deletes an Activity, THE API SHALL verify the Identity has access to the associated Dependent's Family

### Requirement 7: Validate Activity Type-Specific Attributes

**User Story:** As a caregiver, I want the API to validate activity-specific data, so that I can ensure data quality for different activity types.

#### Acceptance Criteria

1. WHERE an Activity type is feeding, THE API SHALL validate that feeding type and volume attributes are present and valid
2. WHERE an Activity type is diaper change, THE API SHALL validate that diaper contents type attribute is present and valid
3. WHERE an Activity type is sleep, THE API SHALL validate that start and end timestamps are present, and end timestamp is after start timestamp
4. WHERE an Activity type is pumping, THE API SHALL validate that volume attribute is present and greater than zero
5. WHEN type-specific validation fails, THE API SHALL return a descriptive error message

### Requirement 8: Validate Request Data

**User Story:** As a system administrator, I want the API to validate all incoming requests, so that data integrity is maintained.

#### Acceptance Criteria

1. WHEN a request contains malformed JSON, THE API SHALL return a 400 Bad Request error with a descriptive message
2. WHEN required fields are missing, THE API SHALL return a 400 Bad Request error listing the missing fields
3. WHEN field values exceed valid ranges, THE API SHALL return a 400 Bad Request error with the validation constraint
4. THE API SHALL sanitize all string inputs to prevent injection attacks

### Requirement 9: Handle Errors Gracefully

**User Story:** As a client application developer, I want consistent error responses, so that I can handle errors appropriately in the UI.

#### Acceptance Criteria

1. WHEN an error occurs, THE API SHALL return an HTTP status code appropriate to the error type
2. WHEN an error occurs, THE API SHALL return a JSON response containing an error message and error code
3. IF an unexpected error occurs, THEN THE API SHALL return a 500 Internal Server Error without exposing internal details
4. WHEN an error occurs, THE API SHALL log the error details for debugging purposes

### Requirement 10: Serialize and Deserialize API Payloads

**User Story:** As a client application developer, I want consistent JSON request and response formats, so that I can reliably integrate with the API.

#### Acceptance Criteria

1. THE API SHALL parse incoming JSON request bodies into strongly-typed Rust structures
2. WHEN a request body cannot be parsed, THE API SHALL return a 400 Bad Request error with details about the parsing failure
3. THE API SHALL serialize response data into valid JSON format
4. FOR ALL valid resource objects, serializing then deserializing SHALL produce an equivalent object (round-trip property)
5. THE API SHALL use ISO 8601 format for all timestamp fields in JSON payloads

### Requirement 11: Support CORS for Web Clients

**User Story:** As a frontend developer, I want the API to support CORS, so that the web console can make requests from a different origin.

#### Acceptance Criteria

1. WHEN a preflight OPTIONS request is received, THE API SHALL respond with appropriate CORS headers
2. THE API SHALL include Access-Control-Allow-Origin header in all responses
3. THE API SHALL include Access-Control-Allow-Methods header listing supported HTTP methods
4. THE API SHALL include Access-Control-Allow-Headers header listing accepted request headers

### Requirement 12: Trust AWS Cognito Authentication

**User Story:** As a system architect, I want the API to trust AWS Cognito authentication, so that authentication and authorization complexity is handled by AWS managed services.

#### Acceptance Criteria

1. THE API SHALL trust authenticated Identity information provided by API Gateway from AWS Cognito
2. THE API SHALL extract Identity information from request context provided by API Gateway
3. THE API SHALL use Identity information to enforce multi-tenant data isolation
4. WHEN Identity information is missing from a request, THE API SHALL return a 401 Unauthorized error
5. WHEN an Identity attempts to access resources outside their authorized Family, THE API SHALL return a 403 Forbidden error

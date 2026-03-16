# Requirements Document

## Introduction

Resource Sharing enables a Family resource owner (the requester) to share access with other users (accepters) at configurable permission scopes. This allows caregivers to collaborate on family management with varying levels of access — for example, a spouse may receive full mutation capabilities while a babysitter receives read-only access with limited write permissions (e.g., activity logging). The requester initiates a share by specifying the target user's email address; the share remains in a pending state until the accepter logs in and accepts it, or it expires. The requester retains full control and can revoke shared permissions at any time. Email is used as the canonical user identifier, providing consistency across the backend and frontend and enabling future external notifications.

## Glossary

- **API**: The famtrac backend REST API service
- **Family**: The top-level resource representing a family unit; the central coordination point for caregivers
- **Requester**: The owner of a Family resource who initiates a share request to grant access to another user
- **Accepter**: An authenticated user who receives shared access to a Family resource from the Requester
- **Share**: A record representing the granted access relationship between a Requester and an Accepter for a specific Family, including the assigned Permission_Scope and Share_Status
- **Share_Status**: The current state of a Share record: `pending` (awaiting acceptance), `active` (accepted and in effect), or `expired` (timed out without acceptance)
- **Permission_Scope**: A defined set of permissions granted to an Accepter on a shared Family (e.g., full, read-only, or a custom combination of resource-level permissions)
- **Identity**: An authenticated user identity provided by AWS Cognito, identified by a canonical sub claim
- **Accepter_Email**: The email address used as the canonical identifier for the target Accepter when creating a Share; corresponds to the Cognito user's email and serves as a consistent identifier across backend and frontend
- **Permission_Action**: A granular permission on a specific resource type (e.g., `family:read`, `dependent:read`, `activity:read`, `activity:write`)
- **Data_Store**: The persistent storage system (DynamoDB) for Share records
- **Stream_Handler**: A Lambda function triggered by DynamoDB Streams that processes resource changes and mirrors (copies/rekeys) records into Accepter partitions based on active Share records
- **Mirrored_Record**: A copy of a Family, Dependent, or Activity record rekeyed into an Accepter's owner partition, annotated with the source Share identifier and Permission_Scope
- **Client**: The application or user making authenticated requests to the API

## Requirements

### Requirement 1: Create a Share

**User Story:** As a family owner, I want to share my Family resource with another user at a specific permission scope, so that they can collaborate on family care activities.

#### Acceptance Criteria

1. WHEN the Requester submits a share request with an Accepter_Email and Permission_Scope, THE API SHALL create a new Share record with Share_Status set to `pending`
2. WHEN the Requester submits a share request, THE API SHALL verify that the Requester is the owner of the specified Family
3. WHEN a Share already exists for the same Family and Accepter_Email combination, THE API SHALL return a conflict error
4. WHEN the Requester attempts to share a Family using the Requester's own email, THE API SHALL return a validation error
5. THE API SHALL store the Share record with the Requester identity, Accepter_Email, Family identifier, Permission_Scope, Share_Status, and creation timestamp
6. THE API SHALL not validate whether the Accepter_Email corresponds to an existing user in the identity provider

### Requirement 2: Define Permission Scopes

**User Story:** As a family owner, I want to assign specific permission scopes when sharing, so that each collaborator has the appropriate level of access for their role.

#### Acceptance Criteria

1. THE API SHALL support the following Permission_Actions: `family:read`, `dependent:read`, `dependent:write`, `activity:read`, `activity:write`
2. THE API SHALL validate that a Permission_Scope contains at least one Permission_Action
3. THE API SHALL validate that a Permission_Scope always includes `family:read` (shared users must be able to read the Family they have access to)
4. WHEN a Permission_Scope contains `dependent:write`, THE API SHALL validate that `dependent:read` is also included
5. WHEN a Permission_Scope contains `activity:write`, THE API SHALL validate that `activity:read` and `dependent:read` are also included
6. THE API SHALL reject a Permission_Scope containing unrecognized Permission_Actions with a descriptive validation error

### Requirement 3: Mirror Shared Resources via DynamoDB Streams

**User Story:** As a shared user, I want shared Family resources to appear in my own account partition, so that I can access them using the same data access patterns as my own resources without cross-tenant queries.

#### Acceptance Criteria

1. WHEN a Share transitions to `active` status, THE Stream_Handler SHALL copy the shared Family record into the Accepter's owner partition with a rekeyed PK of `OWNER#{accepter_identity_id}`
2. WHEN a Share transitions to `active` status, THE Stream_Handler SHALL copy all Dependent records associated with the shared Family into the Accepter's partition
3. WHEN a Share transitions to `active` status, THE Stream_Handler SHALL copy all Activity records associated with the shared Family into the Accepter's partition
4. THE Stream_Handler SHALL store the Permission_Scope from the Share record on each mirrored resource so that permission enforcement can occur at read/write time
5. THE Stream_Handler SHALL store a reference to the source Share identifier on each mirrored record so that revocation can identify and remove mirrored copies
6. WHEN the Requester creates, updates, or deletes a resource on the original Family, THE Stream_Handler SHALL propagate the change to all mirrored copies in Accepter partitions that have an `active` Share

### Requirement 4: Enforce Shared Permissions on Mirrored Resources

**User Story:** As a shared user, I want to access mirrored family resources according to my granted permissions, so that I can perform my caregiving responsibilities within the allowed scope.

#### Acceptance Criteria

1. WHEN an Accepter performs a read operation on a mirrored resource, THE API SHALL verify that the mirrored resource's Permission_Scope includes the required read Permission_Action
2. WHEN an Accepter performs a write operation on a mirrored resource, THE API SHALL verify that the mirrored resource's Permission_Scope includes the required write Permission_Action
3. WHEN an Accepter attempts an operation not covered by the mirrored resource's Permission_Scope, THE API SHALL return a 403 Forbidden error
4. WHEN an Accepter writes to a mirrored resource (e.g., creating an Activity), THE Stream_Handler SHALL propagate the write back to the original Family partition
5. THE API SHALL continue to grant the Requester (owner) full access to all resources on the Family regardless of any Share records

### Requirement 5: List Shares for a Family

**User Story:** As a family owner, I want to see all users who have shared access to my Family, so that I can manage collaborator permissions.

#### Acceptance Criteria

1. WHEN the Requester requests the list of Shares for a Family, THE API SHALL return all Share records for that Family including Accepter identity and Permission_Scope
2. WHEN the Requester requests Shares for a Family the Requester does not own, THE API SHALL return a 404 Not Found error
3. WHEN no Shares exist for a Family, THE API SHALL return an empty list

### Requirement 6: Update a Share Permission Scope

**User Story:** As a family owner, I want to change the permission scope of an existing share, so that I can adjust a collaborator's access level as needs change.

#### Acceptance Criteria

1. WHEN the Requester submits an update to an existing Share with a new Permission_Scope, THE API SHALL replace the existing Permission_Scope with the new one
2. WHEN a Share's Permission_Scope is updated, THE Stream_Handler SHALL update the Permission_Scope on all Mirrored_Records associated with that Share
3. WHEN the Requester attempts to update a Share that does not exist, THE API SHALL return a 404 Not Found error
4. WHEN the Requester attempts to update a Share on a Family the Requester does not own, THE API SHALL return a 404 Not Found error
5. THE API SHALL validate the updated Permission_Scope using the same rules as Share creation

### Requirement 7: Revoke a Share

**User Story:** As a family owner, I want to revoke a user's shared access to my Family, so that I can remove collaborators who no longer need access.

#### Acceptance Criteria

1. WHEN the Requester revokes a Share, THE API SHALL delete the Share record from the Data_Store
2. WHEN a Share is revoked, THE Stream_Handler SHALL delete all Mirrored_Records associated with that Share from the Accepter's partition
3. WHEN the Requester attempts to revoke a Share that does not exist, THE API SHALL return a 404 Not Found error
4. WHEN the Requester attempts to revoke a Share on a Family the Requester does not own, THE API SHALL return a 404 Not Found error

### Requirement 8: List Shared Families for an Accepter

**User Story:** As a shared user, I want to see all Families that have been shared with me, so that I can access the families I collaborate on.

#### Acceptance Criteria

1. WHEN an Accepter requests their shared Families, THE API SHALL return all Share records where the authenticated Identity is the Accepter, including the Family identifier and Permission_Scope
2. WHEN no Families are shared with the Accepter, THE API SHALL return an empty list

### Requirement 9: Accept a Pending Share

**User Story:** As a user who has been invited to collaborate on a Family, I want to accept the share invitation, so that I gain access to the shared Family resources.

#### Acceptance Criteria

1. WHEN an authenticated user accepts a pending Share where the Accepter_Email matches the authenticated user's email, THE API SHALL update the Share_Status to `active` and store the Accepter's canonical Identity identifier on the Share record
2. WHEN an authenticated user attempts to accept a Share where the Accepter_Email does not match the authenticated user's email, THE API SHALL return a 403 Forbidden error
3. WHEN an authenticated user attempts to accept a Share that is not in `pending` status, THE API SHALL return a validation error
4. WHEN an authenticated user attempts to accept a Share that does not exist, THE API SHALL return a 404 Not Found error

### Requirement 10: Expire Unaccepted Shares

**User Story:** As a system operator, I want pending shares to expire after a defined period, so that stale invitations do not accumulate.

#### Acceptance Criteria

1. WHEN a Share remains in `pending` status beyond a configurable expiration period, THE API SHALL treat the Share as expired and exclude the Share from active permission checks
2. WHEN an Accepter attempts to accept an expired Share, THE API SHALL return a validation error indicating the Share has expired
3. WHEN listing Shares for a Family, THE API SHALL include the Share_Status so the Requester can see which Shares are pending, active, or expired

### Requirement 11: Serialize and Deserialize Share Payloads

**User Story:** As a client application developer, I want consistent JSON request and response formats for share operations, so that I can reliably integrate sharing into the UI.

#### Acceptance Criteria

1. THE API SHALL parse incoming JSON request bodies for share operations into strongly-typed Rust structures
2. WHEN a share request body cannot be parsed, THE API SHALL return a 400 Bad Request error with details about the parsing failure
3. THE API SHALL serialize Share response data into valid JSON format
4. FOR ALL valid Share objects, serializing then deserializing SHALL produce an equivalent object (round-trip property)

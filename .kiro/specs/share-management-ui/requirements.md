# Requirements Document

## Introduction

Share Management UI adds frontend screens and components to the famtrac-frontend React/TypeScript application so that family owners can create, list, update, and revoke shares, and accepters can view pending invitations and accept them. The UI integrates with the existing famtrac-backend share API endpoints (POST/GET /families/{fid}/shares, PUT/DELETE /shares/{sid}, POST /shares/{sid}/accept, GET /shares). The feature follows the existing frontend patterns: domain-organized components, API client functions, custom hooks (useApi, useApiMutation, useForm, useValidation), and React Bootstrap for layout.

## Glossary

- **Share_Management_UI**: The set of React components, pages, API client functions, and hooks that enable share management in the famtrac-frontend application
- **Share_API_Client**: The TypeScript module providing typed functions for calling the backend share endpoints
- **Share_List_Component**: A React component that renders a list of Share records for a given family
- **Share_Form_Component**: A React component that collects accepter email and permission scope to create or update a share
- **Share_Card_Component**: A React component that displays a single Share record with status, permissions, and action buttons
- **Pending_Shares_Page**: A React page that displays all shares where the authenticated user is the accepter, including pending invitations
- **Permission_Scope_Selector**: A React component that renders checkboxes for selecting permission actions with dependency enforcement
- **Family_Detail_Page**: The existing page for viewing a single family; extended to include a shares section
- **Navigation_Component**: The existing top-level navigation bar; extended to include a link to pending shares
- **Share_Status_Badge**: A React component that renders a colored badge indicating share status (pending, active, expired)
- **Permission_Action**: One of: `family_read`, `dependent_read`, `dependent_write`, `activity_read`, `activity_write`
- **Confirm_Dialog**: The existing reusable confirmation modal component

## Requirements

### Requirement 1: Share API Client Layer

**User Story:** As a frontend developer, I want typed API client functions for all share endpoints, so that UI components can call the backend share API with type safety.

#### Acceptance Criteria

1. THE Share_API_Client SHALL export a `createShare` function that sends a POST request to `/families/{familyId}/shares` with `accepter_email` and `permission_scope` in the request body and returns a typed `ShareResponse`
2. THE Share_API_Client SHALL export a `getShares` function that sends a GET request to `/families/{familyId}/shares` and returns a typed `ShareListResponse`
3. THE `getShares` function SHALL accept optional pagination parameters (`limit`, `next_token`) and append them as query parameters to the request URL
4. THE Share_API_Client SHALL export an `updateShare` function that sends a PUT request to `/shares/{shareId}` with `permission_scope` in the request body and returns a typed `ShareResponse`
5. THE Share_API_Client SHALL export a `revokeShare` function that sends a DELETE request to `/shares/{shareId}` and returns void
6. THE Share_API_Client SHALL export an `acceptShare` function that sends a POST request to `/shares/{shareId}/accept` and returns a typed `ShareResponse`
7. THE Share_API_Client SHALL export a `getSharesForAccepter` function that sends a GET request to `/shares` and returns a typed `ShareListResponse`
8. THE `getSharesForAccepter` function SHALL accept optional pagination parameters (`limit`, `next_token`) and append them as query parameters to the request URL
9. THE Share_API_Client SHALL define TypeScript interfaces for `ShareResponse`, `ShareListResponse`, `CreateShareRequest`, and `UpdateShareRequest` that match the backend JSON schema
10. THE `ShareListResponse` interface SHALL include an optional `next_token` field for pagination, matching the existing `FamilyListResponse` and `DependentListResponse` patterns

### Requirement 2: Share Domain Types

**User Story:** As a frontend developer, I want TypeScript types for share-related domain objects, so that the application has consistent type definitions across components.

#### Acceptance Criteria

1. THE Share_Management_UI SHALL define a `Share` interface in the domain types module with fields: `id`, `family_id`, `requester_id`, `accepter_email`, `accepter_id` (optional), `permission_scope`, `status`, `created_at`, `updated_at`, and `expires_at` (optional)
2. THE Share_Management_UI SHALL define a `PermissionScope` interface with an `actions` field containing an array of `PermissionAction` values
3. THE Share_Management_UI SHALL define a `PermissionAction` type as a union of string literals: `family_read`, `dependent_read`, `dependent_write`, `activity_read`, `activity_write`
4. THE Share_Management_UI SHALL define a `ShareStatus` type as a union of string literals: `pending`, `active`, `expired`

### Requirement 3: Permission Scope Selector Component

**User Story:** As a family owner, I want to select which permissions to grant when creating or updating a share, so that I can control the level of access for each collaborator.

#### Acceptance Criteria

1. THE Permission_Scope_Selector SHALL render a checkbox for each Permission_Action: `family_read`, `dependent_read`, `dependent_write`, `activity_read`, `activity_write`
2. THE Permission_Scope_Selector SHALL keep the `family_read` checkbox checked and disabled because `family_read` is always required
3. WHEN the user checks `dependent_write`, THE Permission_Scope_Selector SHALL automatically check `dependent_read` and disable unchecking `dependent_read` while `dependent_write` remains checked
4. WHEN the user checks `activity_write`, THE Permission_Scope_Selector SHALL automatically check `activity_read` and `dependent_read` and disable unchecking those while `activity_write` remains checked
5. WHEN the user unchecks `activity_write`, THE Permission_Scope_Selector SHALL re-enable `activity_read` for unchecking (unless another dependency still requires `activity_read`)
6. THE Permission_Scope_Selector SHALL emit the selected Permission_Actions as a `PermissionScope` object via an `onChange` callback
7. THE Permission_Scope_Selector SHALL accept an optional initial `PermissionScope` prop for pre-populating selections when editing an existing share

### Requirement 4: Create Share Form

**User Story:** As a family owner, I want a form to invite a user by email with specific permissions, so that I can share my family with collaborators.

#### Acceptance Criteria

1. THE Share_Form_Component SHALL render an email input field for the accepter email address
2. THE Share_Form_Component SHALL validate that the email field is not empty and contains a valid email format before submission
3. THE Share_Form_Component SHALL embed the Permission_Scope_Selector for selecting permission actions
4. WHEN the user submits the form with valid data, THE Share_Form_Component SHALL call the `createShare` API client function with the family ID, accepter email, and selected permission scope
5. WHEN the API returns a conflict error (duplicate share), THE Share_Form_Component SHALL display an error message indicating a share already exists for that email
6. WHEN the API returns a validation error (e.g., self-sharing), THE Share_Form_Component SHALL display the error message from the API response
7. WHEN the share is created successfully, THE Share_Form_Component SHALL call an `onSuccess` callback so the parent component can refresh the share list and close the form

### Requirement 5: Share Card Component

**User Story:** As a family owner, I want to see share details at a glance, so that I can quickly understand who has access and at what level.

#### Acceptance Criteria

1. THE Share_Card_Component SHALL display the accepter email address
2. THE Share_Card_Component SHALL display a Share_Status_Badge showing the current status (pending, active, expired)
3. THE Share_Card_Component SHALL display the granted permission actions in a readable format
4. THE Share_Card_Component SHALL display an "Edit" button that triggers an `onEdit` callback for active and pending shares
5. THE Share_Card_Component SHALL display a "Revoke" button that triggers an `onRevoke` callback
6. WHILE the share status is `expired`, THE Share_Card_Component SHALL disable the "Edit" button

### Requirement 6: Share List Component

**User Story:** As a family owner, I want to see all shares for my family in a list, so that I can manage collaborator access.

#### Acceptance Criteria

1. THE Share_List_Component SHALL render a Share_Card_Component for each share in the provided list
2. WHEN the share list is loading, THE Share_List_Component SHALL display a loading indicator
3. WHEN the share list is empty, THE Share_List_Component SHALL display a message indicating no shares exist
4. WHEN an API error occurs while loading shares, THE Share_List_Component SHALL display an error message
5. WHEN the API response includes a `next_token`, THE Share_List_Component SHALL display a "Load More" button
6. WHEN the user clicks "Load More", THE Share_List_Component SHALL fetch the next page of shares using the `next_token` and append the results to the existing list
7. WHILE the next page is loading, THE Share_List_Component SHALL disable the "Load More" button and show a loading indicator on it

### Requirement 7: Share Management Section on Family Detail Page

**User Story:** As a family owner, I want to manage shares directly from the family detail page, so that I can handle sharing without navigating away.

#### Acceptance Criteria

1. THE Family_Detail_Page SHALL include a "Shares" section below the existing dependents section
2. THE Family_Detail_Page SHALL display the Share_List_Component in the shares section, populated by calling `getShares` for the current family
3. THE Family_Detail_Page SHALL display an "Invite User" button that opens a modal containing the Share_Form_Component
4. WHEN the user clicks "Edit" on a Share_Card_Component, THE Family_Detail_Page SHALL open a modal with the Permission_Scope_Selector pre-populated with the share's current permissions and call `updateShare` on submission
5. WHEN the user clicks "Revoke" on a Share_Card_Component, THE Family_Detail_Page SHALL open the Confirm_Dialog asking for confirmation and call `revokeShare` on confirmation
6. WHEN a share is created, updated, or revoked successfully, THE Family_Detail_Page SHALL refresh the share list and display a success message

### Requirement 8: Pending Shares Page for Accepters

**User Story:** As a user who has been invited to collaborate on a family, I want to see my pending share invitations, so that I can accept them and gain access.

#### Acceptance Criteria

1. THE Pending_Shares_Page SHALL call `getSharesForAccepter` to fetch all shares for the authenticated user
2. THE Pending_Shares_Page SHALL display each share with the family ID, requester identity, permission scope, and status
3. WHEN a share has status `pending`, THE Pending_Shares_Page SHALL display an "Accept" button
4. WHEN the user clicks "Accept", THE Pending_Shares_Page SHALL call `acceptShare` and refresh the list on success
5. WHEN the accept API returns a validation error (expired share), THE Pending_Shares_Page SHALL display the error message
6. WHEN no shares exist for the accepter, THE Pending_Shares_Page SHALL display a message indicating no shared families
7. WHEN the API response includes a `next_token`, THE Pending_Shares_Page SHALL display a "Load More" button to fetch additional shares
8. WHEN the user clicks "Load More", THE Pending_Shares_Page SHALL fetch the next page using the `next_token` and append the results to the existing list

### Requirement 9: Navigation and Routing

**User Story:** As a user, I want to navigate to the pending shares page from the main navigation, so that I can easily find and manage my share invitations.

#### Acceptance Criteria

1. THE Navigation_Component SHALL include a "Shared With Me" link that navigates to the Pending_Shares_Page
2. THE Share_Management_UI SHALL register a `/shares` route in the application router that renders the Pending_Shares_Page wrapped in ProtectedRoute
3. WHEN the user is not authenticated, THE Share_Management_UI SHALL redirect the `/shares` route to the login flow via the existing ProtectedRoute component

### Requirement 10: Share Status Badge Component

**User Story:** As a user, I want a visual indicator of share status, so that I can quickly distinguish between pending, active, and expired shares.

#### Acceptance Criteria

1. WHEN the share status is `pending`, THE Share_Status_Badge SHALL render a yellow/warning badge with the text "Pending"
2. WHEN the share status is `active`, THE Share_Status_Badge SHALL render a green/success badge with the text "Active"
3. WHEN the share status is `expired`, THE Share_Status_Badge SHALL render a gray/secondary badge with the text "Expired"

### Requirement 11: Client-Side Permission Scope Validation

**User Story:** As a family owner, I want immediate feedback when I select an invalid permission combination, so that I do not submit invalid share requests.

#### Acceptance Criteria

1. THE Share_Form_Component SHALL validate that the selected permission scope contains at least `family_read` before allowing submission
2. THE Share_Form_Component SHALL validate that `dependent_write` is accompanied by `dependent_read` before allowing submission
3. THE Share_Form_Component SHALL validate that `activity_write` is accompanied by `activity_read` and `dependent_read` before allowing submission
4. WHEN the permission scope is invalid, THE Share_Form_Component SHALL display a descriptive validation error and disable the submit button

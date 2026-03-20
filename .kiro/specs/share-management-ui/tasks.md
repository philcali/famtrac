# Implementation Plan: Share Management UI

## Overview

Add share management to the famtrac application: backend pagination support for share list endpoints, frontend domain types, API client, permission utilities, share components (form, card, list, status badge, permission selector), a Pending Shares page, and integration into the existing Family Detail Page and Navigation. Backend changes come first since the frontend depends on paginated API responses.

## Tasks

- [x] 1. Backend pagination for share list endpoints
  - [x] 1.1 Update `ShareRepository` trait in `famtrac-backend/src/repository/traits.rs`
    - Change `list_by_family` to accept `PaginationParams` and return `PaginatedResponse<Share>` instead of `Vec<Share>`
    - Change `list_by_accepter_email` to accept `PaginationParams` and return `PaginatedResponse<Share>` instead of `Vec<Share>`
    - Import `PaginationParams` and `PaginatedResponse` from `crate::handlers::pagination`
    - _Requirements: 1.3, 1.8_

  - [x] 1.2 Update `DynamoDbShareRepository` in `famtrac-backend/src/repository/dynamodb.rs`
    - Update `list_by_family` to use DynamoDB `Limit` and `ExclusiveStartKey` for cursor-based pagination
    - Update `list_by_accepter_email` to use DynamoDB `Limit` and `ExclusiveStartKey` for cursor-based pagination
    - Encode `LastEvaluatedKey` as base64 JSON for `next_token`; decode `ExclusiveStartKey` from `next_token`
    - Use `PaginationParams::effective_limit()` to apply default (50) and max (100) constraints
    - _Requirements: 1.3, 1.8_

  - [x] 1.3 Update `MockShareRepository` in `famtrac-backend/src/test_utils.rs`
    - Update `list_by_family` and `list_by_accepter_email` to match new trait signatures with `PaginationParams` and `PaginatedResponse<Share>`
    - Implement basic in-memory pagination (slice + next_token) for test support
    - _Requirements: 1.3, 1.8_

  - [x] 1.4 Update `ShareListResponse` in `famtrac-backend/src/handlers/share.rs`
    - Add `#[serde(skip_serializing_if = "Option::is_none")] pub next_token: Option<String>` field to `ShareListResponse`
    - _Requirements: 1.10_

  - [x] 1.5 Update `list_shares` handler in `famtrac-backend/src/handlers/share.rs`
    - Accept `PaginationParams` parameter
    - Pass pagination to `share_repo.list_by_family`
    - Populate `ShareListResponse.next_token` from `PaginatedResponse.next_token`
    - _Requirements: 1.2, 1.3_

  - [x] 1.6 Update `list_shares_for_accepter` handler in `famtrac-backend/src/handlers/share.rs`
    - Accept `PaginationParams` parameter
    - Pass pagination to `share_repo.list_by_accepter_email`
    - Populate `ShareListResponse.next_token` from `PaginatedResponse.next_token`
    - _Requirements: 1.7, 1.8_

  - [x] 1.7 Update share router in `famtrac-backend/src/router/share.rs`
    - Parse `limit` and `next_token` query string parameters from the request into `PaginationParams`
    - Pass `PaginationParams` to `list_shares` and `list_shares_for_accepter` handlers
    - Update `route_family_shares` and `route_shares` to accept query params (from `ApiGatewayV2httpRequest`)
    - Update `famtrac-backend/src/router/mod.rs` to pass query params through to share route functions
    - _Requirements: 1.3, 1.8_

- [ ] 2. Checkpoint - Ensure backend compiles and all existing tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Frontend share domain types and API types
  - [x] 3.1 Add share domain types to `famtrac-frontend/src/types/domain.ts`
    - Add `PermissionAction` type union: `'family_read' | 'dependent_read' | 'dependent_write' | 'activity_read' | 'activity_write'`
    - Add `PermissionScope` interface with `actions: PermissionAction[]`
    - Add `ShareStatus` type union: `'pending' | 'active' | 'expired'`
    - Add `Share` interface with all fields: `id`, `family_id`, `requester_id`, `accepter_email`, `accepter_id?`, `permission_scope`, `status`, `created_at`, `updated_at`, `expires_at?`
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

  - [x] 3.2 Add share API types to `famtrac-frontend/src/api/types.ts`
    - Add `CreateShareRequest` interface with `accepter_email` and `permission_scope`
    - Add `UpdateShareRequest` interface with `permission_scope`
    - Add `ShareResponse` interface matching backend JSON schema
    - Add `ShareListResponse` interface with `shares: ShareResponse[]` and optional `next_token`
    - _Requirements: 1.9, 1.10_

- [x] 4. Frontend API client and utilities
  - [x] 4.1 Create `famtrac-frontend/src/api/shares.ts` with all 6 API functions
    - `createShare(client, familyId, request)` → POST `/families/{familyId}/shares`
    - `getShares(client, familyId, options?)` → GET `/families/{familyId}/shares` with optional `limit`/`next_token` query params
    - `updateShare(client, shareId, request)` → PUT `/shares/{shareId}`
    - `revokeShare(client, shareId)` → DELETE `/shares/{shareId}`
    - `acceptShare(client, shareId)` → POST `/shares/{shareId}/accept`
    - `getSharesForAccepter(client, options?)` → GET `/shares` with optional `limit`/`next_token` query params
    - Follow the same pattern as `api/families.ts` and `api/dependents.ts`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8_

  - [ ]* 4.2 Write property tests for API client functions (Property 1, Property 9)
    - **Property 1: API client functions use correct HTTP method and path**
    - **Property 9: Pagination query parameter construction**
    - Test file: `famtrac-frontend/src/api/shares.test.ts`
    - Mock `ApiClient`, generate random UUIDs, verify correct method/path/query params
    - **Validates: Requirements 1.1, 1.2, 1.4, 1.5, 1.6, 1.3, 1.8**

  - [x] 4.3 Create `famtrac-frontend/src/utils/permissions.ts` with pure permission dependency logic
    - Export `ALWAYS_REQUIRED`, `PERMISSION_DEPENDENCIES`, `PERMISSION_LABELS` constants
    - Implement `getLockedActions(selected)` → returns `Set<PermissionAction>` of non-uncheckable actions
    - Implement `addActionWithDependencies(current, action)` → returns new action array with auto-selected dependencies
    - Implement `removeAction(current, action)` → returns new action array without removed action and orphaned dependents
    - Implement `validatePermissionScope(actions)` → returns null if valid, error string if invalid
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 11.1, 11.2, 11.3_

  - [ ]* 4.4 Write property tests for permission utilities (Property 2, 3, 4, 5, 7)
    - **Property 2: Permission dependency enforcement**
    - **Property 3: Adding an action auto-selects its dependencies**
    - **Property 4: Removing an action preserves unrelated selections**
    - **Property 5: Permission scope validation**
    - **Property 7: Permission action label completeness**
    - Test file: `famtrac-frontend/src/utils/permissions.test.ts`
    - **Validates: Requirements 3.2, 3.3, 3.4, 3.5, 11.1, 11.2, 11.3, 5.3**

  - [x] 4.5 Add `email()` validator to `famtrac-frontend/src/utils/validation.ts`
    - Validate non-empty string with standard email format (one `@`, non-empty local and domain parts)
    - Return `ValidationResult` following existing validator pattern
    - _Requirements: 4.2_

  - [ ]* 4.6 Write property test for email validation (Property 8)
    - **Property 8: Email validation**
    - Test file: `famtrac-frontend/src/utils/validation.test.ts`
    - Generate random strings (valid emails, invalid strings, whitespace), verify correct accept/reject
    - **Validates: Requirements 4.2**

- [x] 5. Share UI components
  - [x] 5.1 Create `famtrac-frontend/src/components/shares/ShareStatusBadge.tsx`
    - Render React Bootstrap `Badge` with variant mapping: `pending` → `warning`, `active` → `success`, `expired` → `secondary`
    - Display capitalized status text
    - _Requirements: 10.1, 10.2, 10.3_

  - [ ]* 5.2 Write property test for ShareStatusBadge (Property 6)
    - **Property 6: Status badge mapping**
    - Test file: `famtrac-frontend/src/components/shares/ShareStatusBadge.test.tsx`
    - Generate random `ShareStatus` values, render component, verify correct variant and text
    - **Validates: Requirements 10.1, 10.2, 10.3**

  - [x] 5.3 Create `famtrac-frontend/src/components/shares/PermissionScopeSelector.tsx`
    - Render `Form.Check` checkbox for each of the 5 permission actions
    - `family_read` always checked and disabled
    - Use `getLockedActions` from `utils/permissions.ts` to determine disabled state
    - Use `addActionWithDependencies` and `removeAction` for toggle logic
    - Accept `value: PermissionAction[]` and `onChange` callback props
    - Accept optional `disabled` prop
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

  - [x] 5.4 Create `famtrac-frontend/src/components/shares/ShareCard.tsx`
    - Display accepter email, `ShareStatusBadge`, permission labels from `PERMISSION_LABELS`
    - Render "Edit" button (calls `onEdit`, disabled when `status === 'expired'`)
    - Render "Revoke" button (calls `onRevoke`)
    - Render "Accept" button only when `onAccept` prop is provided (for PendingSharesPage)
    - Follow `DependentCard` pattern
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

  - [x] 5.5 Create `famtrac-frontend/src/components/shares/ShareForm.tsx`
    - Email input with `required` and `email` validators via `useValidation`
    - Embed `PermissionScopeSelector`
    - Validate permission scope via `validatePermissionScope` before submission
    - Call `onSubmit` with `CreateShareRequest` data
    - Follow `DependentForm` pattern
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 11.1, 11.2, 11.3, 11.4_

  - [x] 5.6 Create `famtrac-frontend/src/components/shares/ShareList.tsx`
    - Render `ShareCard` for each share in a `Row`/`Col` grid
    - Show `SkeletonCard` when loading, `ErrorMessage` on error, empty state message when no shares
    - Render "Load More" button when `hasMore` is true and list is non-empty
    - Disable "Load More" and show spinner when `loadingMore` is true
    - Call `onLoadMore` when "Load More" is clicked
    - Follow `DependentList` pattern
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_

  - [ ]* 5.7 Write property tests for ShareList (Property 10, Property 11)
    - **Property 10: Load More button visibility**
    - **Property 11: Load More button disabled during loading**
    - Test file: `famtrac-frontend/src/components/shares/ShareList.test.tsx`
    - **Validates: Requirements 6.5, 6.7, 8.7**

- [x] 6. Page integration and routing
  - [x] 6.1 Update `famtrac-frontend/src/pages/FamilyDetailPage.tsx` to add shares section
    - Add "Shares" section below the existing dependents section
    - Add "Invite User" button that opens a modal with `ShareForm`
    - Use `useApi` to fetch shares via `getShares(apiClient, familyId)`
    - Manage local state for accumulated shares and `nextToken` for pagination
    - Handle "Load More" by calling `getShares` with `next_token` and appending results
    - Handle "Edit" by opening a modal with `PermissionScopeSelector` pre-populated, calling `updateShare` on submit
    - Handle "Revoke" by opening `ConfirmDialog`, calling `revokeShare` on confirmation
    - Refresh share list and show success message after create/update/revoke
    - Use `useApiMutation` for `createShare`, `updateShare`, `revokeShare`
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_

  - [x] 6.2 Create `famtrac-frontend/src/pages/PendingSharesPage.tsx`
    - Call `getSharesForAccepter` to fetch shares for the authenticated user
    - Manage local state for accumulated shares and `nextToken` for pagination
    - Render each share with `ShareCard` (with `onAccept` prop)
    - Handle "Accept" by calling `acceptShare` and refreshing the list
    - Handle "Load More" by calling `getSharesForAccepter` with `next_token` and appending results
    - Show loading, error, and empty states
    - Display error messages from API (e.g., expired share)
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_

  - [x] 6.3 Update `famtrac-frontend/src/components/common/Navigation.tsx`
    - Add "Shared With Me" `Nav.Link` pointing to `/shares`, next to the existing "Families" link
    - _Requirements: 9.1_

  - [x] 6.4 Update `famtrac-frontend/src/App.tsx` to add `/shares` route
    - Add `<Route path="/shares" element={<ProtectedRoute><PendingSharesPage /></ProtectedRoute>} />`
    - Import `PendingSharesPage`
    - _Requirements: 9.2, 9.3_

- [ ] 7. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- **IMPORTANT**: Files may change between task runs. Always read a file before editing it — never assume its contents match what was seen in a previous task.
- Tasks marked with `*` are optional and can be skipped for faster MVP
- Backend pagination changes (task 1) must be completed before frontend work since the frontend depends on paginated API responses
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties from the design document
- The frontend follows existing patterns: `api/families.ts` for API client, `DependentCard`/`DependentList` for components, `useApi`/`useApiMutation` for data fetching

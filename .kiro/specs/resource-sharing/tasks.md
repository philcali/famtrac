# Implementation Plan: Resource Sharing

## Overview

Implement resource sharing for the famtrac backend, allowing family owners to share access with other users at configurable permission scopes. The implementation follows the existing Rust/DynamoDB patterns, extending the single-table schema with Share records, adding share handlers and routes, extending existing domain types with optional share metadata, adding permission enforcement on mirrored resources, and creating a separate stream handler Lambda for the DynamoDB Streams mirroring pipeline.

## Tasks

- [x] 1. Define share domain types and permission validation
  - [x] 1.1 Create `famtrac-backend/src/domain/share.rs` with `ShareId`, `ShareStatus`, `PermissionAction`, `PermissionScope`, and `Share` structs
    - `ShareId` wraps `Uuid` (like `FamilyId`)
    - `ShareStatus` enum: `Pending`, `Active`, `Expired`
    - `PermissionAction` enum: `FamilyRead`, `DependentRead`, `DependentWrite`, `ActivityRead`, `ActivityWrite`
    - `PermissionScope` struct with `actions: Vec<PermissionAction>`
    - `Share` struct with all fields from design: `id`, `family_id`, `requester_id`, `accepter_email`, `accepter_id` (Option), `permission_scope`, `status`, `created_at`, `updated_at`, `expires_at` (Option)
    - Derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq` as appropriate
    - _Requirements: 1.5, 2.1_

  - [x] 1.2 Implement `PermissionScope::validate()` method on `PermissionScope`
    - Must contain at least one action
    - Must include `FamilyRead`
    - `DependentWrite` requires `DependentRead`
    - `ActivityWrite` requires `ActivityRead` and `DependentRead`
    - Return `ValidationError` with descriptive messages on failure
    - _Requirements: 2.2, 2.3, 2.4, 2.5, 2.6_

  - [x] 1.3 Register the share module in `famtrac-backend/src/domain/mod.rs` and re-export the new types
    - _Requirements: 1.5, 2.1_

  - [ ]* 1.4 Write property test for permission scope validation (Property 3)
    - **Property 3: Permission scope validation rules**
    - Generate random subsets of `PermissionAction`, call `validate()`, verify result matches the rules
    - **Validates: Requirements 2.2, 2.3, 2.4, 2.5**

  - [ ]* 1.5 Write property test for share serialization round-trip (Property 17)
    - **Property 17: Share serialization round-trip**
    - Generate random valid `Share` objects, serialize to JSON, deserialize, assert equality
    - **Validates: Requirements 11.4**

- [x] 2. Extend error types for sharing
  - [x] 2.1 Add `Conflict(String)` and `Forbidden(String)` variants to `HandlerError` in `famtrac-backend/src/errors/mod.rs`
    - `Conflict` maps to HTTP 409 with code `CONFLICT_ERROR`
    - `Forbidden` maps to HTTP 403 with code `FORBIDDEN`
    - Update `status_code()` and `to_error_response()` implementations
    - Update `Display` impl
    - _Requirements: 1.3, 4.3, 9.2_

- [x] 3. Extend existing domain types with share metadata
  - [x] 3.1 Add optional `share_id: Option<ShareId>` and `permission_scope: Option<PermissionScope>` fields to `Family`, `Dependent`, and `Activity` structs in `famtrac-backend/src/domain/`
    - Use `#[serde(skip_serializing_if = "Option::is_none")]` to keep existing API responses unchanged for owned resources
    - Default to `None` for owned resources
    - _Requirements: 3.4, 3.5, 4.1, 4.2_

  - [x] 3.2 Update `DynamoDbFamilyRepository`, `DynamoDbDependentRepository`, and `DynamoDbActivityRepository` `to_item` and `parse_item` methods in `famtrac-backend/src/repository/dynamodb.rs` to handle the new optional `share_id` and `permission_scope` fields
    - Write `share_id` and `permission_scope` to DynamoDB only when `Some`
    - Parse them from DynamoDB items, defaulting to `None` when absent
    - _Requirements: 3.4, 3.5_

  - [x] 3.3 Update `MockFamilyRepository`, `MockDependentRepository`, and `MockActivityRepository` in `famtrac-backend/src/test_utils.rs` to handle the new optional fields
    - _Requirements: 3.4, 3.5_

- [x] 4. Extend RequestContext with email claim
  - [x] 4.1 Add `email: Option<String>` field to `RequestContext` in `famtrac-backend/src/context.rs`
    - Extract from JWT `email` claim in `from_api_gateway_context`
    - Needed for the accept share flow to verify email match
    - _Requirements: 9.1, 9.2_

- [ ] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement ShareRepository trait and DynamoDB implementation
  - [x] 6.1 Define `ShareRepository` trait in `famtrac-backend/src/repository/traits.rs`
    - Methods: `create`, `get`, `get_by_email_and_share_id`, `update`, `delete`, `list_by_family`, `list_by_accepter_email`, `list_by_accepter_id`, `get_by_family_and_email`
    - Follow existing trait patterns (async_trait, Send + Sync)
    - _Requirements: 1.1, 1.3, 5.1, 6.1, 7.1, 8.1, 9.1_

  - [x] 6.2 Implement `DynamoDbShareRepository` in `famtrac-backend/src/repository/dynamodb.rs`
    - Dual-write pattern: each Share writes to both `OWNER#{requester_id}/SHARE#{share_id}` and `SHARE_EMAIL#{accepter_email}/SHARE#{share_id}` using TransactWriteItems
    - `create` uses condition expression to prevent duplicates (conflict detection)
    - `get` retrieves from owner partition
    - `get_by_email_and_share_id` does a direct GetItem on the email partition (`SHARE_EMAIL#{accepter_email}/SHARE#{share_id}`)
    - `list_by_family` queries owner partition with SK prefix `SHARE#` and filters by `family_id`
    - `list_by_accepter_email` queries `SHARE_EMAIL#{email}` partition
    - `list_by_accepter_id` queries GSI-AccepterShares
    - `get_by_family_and_email` queries email partition and filters by `family_id`
    - `update` updates both items in a transaction
    - `delete` deletes both items in a transaction
    - _Requirements: 1.1, 1.3, 1.5, 5.1, 6.1, 7.1, 8.1_

  - [x] 6.3 Register `DynamoDbShareRepository` in `famtrac-backend/src/repository/mod.rs` and re-export
    - _Requirements: 1.1_

  - [x] 6.4 Create `MockShareRepository` in `famtrac-backend/src/test_utils.rs` for unit testing
    - In-memory implementation matching the trait
    - _Requirements: 1.1_

- [x] 7. Implement permission enforcement utility
  - [x] 7.1 Create `check_permission` function (in `famtrac-backend/src/handlers/` or a shared utils module)
    - If `share_id` is `None`, resource is owned — allow full access
    - If `share_id` is `Some`, check `permission_scope` contains the required `PermissionAction`
    - Return `HandlerError::Forbidden` if not permitted
    - _Requirements: 4.1, 4.2, 4.3, 4.5_

  - [ ]* 7.2 Write property test for permission enforcement (Property 6)
    - **Property 6: Permission enforcement on mirrored resources**
    - Generate random permission scopes and required actions, verify `check_permission` result matches scope contents
    - **Validates: Requirements 4.1, 4.2, 4.3**

  - [ ]* 7.3 Write property test for owner full access (Property 7)
    - **Property 7: Owner retains full access regardless of shares**
    - Generate resources with `share_id=None`, verify all operations succeed
    - **Validates: Requirements 4.5**

- [x] 8. Integrate permission checks into existing handlers
  - [x] 8.1 Add `check_permission` calls to existing Family, Dependent, and Activity handlers in `famtrac-backend/src/handlers/`
    - After fetching a resource, check if it's mirrored (has `share_id`) and enforce permissions
    - Read operations require the corresponding read action (`family:read`, `dependent:read`, `activity:read`)
    - Write operations require the corresponding write action (`dependent:write`, `activity:write`)
    - Owner resources (`share_id=None`) bypass permission checks
    - _Requirements: 4.1, 4.2, 4.3, 4.5_

- [x] 9. Implement share handlers
  - [x] 9.1 Create `famtrac-backend/src/handlers/share.rs` with `create_share` handler
    - Parse JSON request body into `CreateShareRequest` (accepter_email, permission_scope)
    - Validate permission scope
    - Verify requester owns the family via `FamilyRepository::get`
    - Reject self-sharing (requester email = accepter email)
    - Check for duplicate share via `get_by_family_and_email`
    - Create Share with status `Pending`, generate `ShareId`, set timestamps and expiration
    - Return 201 with created Share
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 11.1, 11.2, 11.3_

  - [ ]* 9.2 Write property test for share creation (Property 1)
    - **Property 1: Share creation produces complete pending record**
    - Generate random valid emails and permission scopes, call `create_share`, verify all fields
    - **Validates: Requirements 1.1, 1.5**

  - [x] 9.3 Implement `list_shares` handler
    - Verify requester owns the family
    - Query shares by family, include status (pending/active/expired)
    - Return empty list when no shares exist
    - _Requirements: 5.1, 5.2, 5.3, 10.3_

  - [ ]* 9.4 Write property test for list shares (Property 8)
    - **Property 8: List shares returns complete set with status**
    - Generate random sets of shares for a family, list them, verify count and data completeness
    - **Validates: Requirements 5.1, 10.3**

  - [x] 9.5 Implement `update_share` handler
    - Verify requester owns the family associated with the share
    - Validate new permission scope using same rules as creation
    - Replace existing permission scope
    - Return 404 if share or family not found
    - _Requirements: 6.1, 6.3, 6.4, 6.5_

  - [ ]* 9.6 Write property test for share update (Property 9)
    - **Property 9: Share update replaces permission scope**
    - Generate random shares and new valid scopes, update, verify scope is replaced
    - **Validates: Requirements 6.1, 6.5**

  - [x] 9.7 Implement `revoke_share` handler
    - Verify requester owns the family associated with the share
    - Delete the share record
    - Return 404 if share or family not found
    - _Requirements: 7.1, 7.3, 7.4_

  - [ ]* 9.8 Write property test for share revocation (Property 10)
    - **Property 10: Share revocation deletes share record**
    - Generate random shares, revoke, verify share is deleted
    - **Validates: Requirements 7.1**

  - [x] 9.9 Implement `accept_share` handler
    - Look up share by accepter email (from RequestContext) and share ID via `get_by_email_and_share_id`
    - Verify accepter email matches authenticated user's email (from RequestContext)
    - Verify share is in `Pending` status (reject `Active`/`Expired`)
    - Check expiration: if past expiration period, return validation error
    - Set status to `Active`, store accepter's `IdentityId`
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 10.1, 10.2_

  - [ ]* 9.10 Write property test for share acceptance (Property 13)
    - **Property 13: Share acceptance transitions to active with identity**
    - Generate random pending shares with matching emails, accept, verify status=active and identity stored
    - **Validates: Requirements 9.1**

  - [ ]* 9.11 Write property test for email mismatch on acceptance (Property 14)
    - **Property 14: Email mismatch on acceptance returns 403**
    - Generate random shares with non-matching emails, attempt accept, verify 403
    - **Validates: Requirements 9.2**

  - [ ]* 9.12 Write property test for non-pending acceptance (Property 15)
    - **Property 15: Non-pending share acceptance returns validation error**
    - Generate random shares in active/expired status, attempt accept, verify validation error
    - **Validates: Requirements 9.3**

  - [x] 9.13 Implement `list_shared_families` handler
    - Query shares by accepter identity ID (via GSI)
    - Return share records with family identifier and permission scope
    - Return empty list when no shares exist
    - _Requirements: 8.1, 8.2_

  - [ ]* 9.14 Write property test for accepter listing (Property 12)
    - **Property 12: Accepter share listing returns all shares**
    - Generate random shares across families for an accepter, list, verify count and data
    - **Validates: Requirements 8.1**

  - [x] 9.15 Register share handlers in `famtrac-backend/src/handlers/mod.rs`
    - _Requirements: 1.1_

  - [ ]* 9.16 Write property test for ownership enforcement (Property 2)
    - **Property 2: Ownership enforcement on share operations**
    - Generate random identity pairs (owner ≠ requester), attempt share operations, assert 404
    - **Validates: Requirements 1.2, 5.2, 6.4, 7.4**

  - [ ]* 9.17 Write property test for invalid JSON (Property 18)
    - **Property 18: Invalid JSON returns 400**
    - Generate random non-JSON strings, attempt parse, verify 400 error
    - **Validates: Requirements 11.2**

- [ ] 10. Add share routes to the router
  - [ ] 10.1 Create `famtrac-backend/src/router/share.rs` with route matching for share endpoints
    - `POST /families/{fid}/shares` → `create_share`
    - `GET /families/{fid}/shares` → `list_shares`
    - `PUT /shares/{sid}` → `update_share`
    - `DELETE /shares/{sid}` → `revoke_share`
    - `POST /shares/{sid}/accept` → `accept_share`
    - `GET /shared-families` → `list_shared_families`
    - Follow existing router patterns (extract path params, delegate to handlers)
    - _Requirements: 1.1, 5.1, 6.1, 7.1, 8.1, 9.1_

  - [ ] 10.2 Update `famtrac-backend/src/router/mod.rs` to wire in share routes
    - Add `DynamoDbShareRepository` parameter to `route_request`
    - Add path matching for `/shares` and `/shared-families` prefixes
    - Wire `/families/{fid}/shares` through the existing family path prefix
    - _Requirements: 1.1, 5.1, 6.1, 7.1, 8.1, 9.1_

  - [ ] 10.3 Update `famtrac-backend/src/main.rs` to instantiate `DynamoDbShareRepository` and pass it to `route_request`
    - _Requirements: 1.1_

- [ ] 11. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 12. Add GSI for accepter share lookup
  - [ ] 12.1 Add `GSI-AccepterShares` to the DynamoDB table definition in `famtrac-infra/lib/backend/FamtracApi.ts`
    - GSI PK: `accepter_id`, GSI SK: `SK` (table SK)
    - Projection: ALL
    - _Requirements: 8.1_

  - [ ] 12.2 Update `DynamoDbShareRepository` to use the new GSI in `list_by_accepter_id`
    - _Requirements: 8.1_

- [ ] 13. Implement stream handler Lambda for mirroring
  - [ ] 13.1 Create `famtrac-stream-handler/` Rust project with Cargo.toml
    - Add dependencies: `aws-lambda-events`, `lambda_runtime`, `aws-sdk-dynamodb`, `serde`, `serde_json`, `tokio`
    - Share domain types from `famtrac-backend` or duplicate minimal types needed
    - _Requirements: 3.1, 3.2, 3.3, 3.6_

  - [ ] 13.2 Implement stream event classification in `famtrac-stream-handler/src/main.rs`
    - Parse DynamoDB Stream records
    - Classify into: `ShareActivated`, `ShareRevoked`, `SharePermissionUpdated`, `ResourceChanged`, `Ignored`
    - _Requirements: 3.1, 3.6, 6.2, 7.2_

  - [ ] 13.3 Implement `mirror_resources` for share activation
    - On share activation, query the original family, all dependents, and all activities
    - Copy each record into the accepter's partition with rekeyed PK (`OWNER#{accepter_id}` for Family)
    - Annotate each mirrored record with `share_id` and `permission_scope`
    - Use conditional writes for idempotency
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [ ]* 13.4 Write property test for share activation mirroring (Property 4)
    - **Property 4: Share activation mirrors all resources with metadata**
    - Generate random families with dependents/activities, activate a share, verify all mirrored records exist with correct metadata
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**

  - [ ] 13.5 Implement `propagate_change` for resource change propagation
    - On resource create/update/delete, find all active shares for the affected family
    - Propagate the change to all mirrored copies in accepter partitions
    - Handle write-back: changes on mirrored resources propagate to original partition
    - _Requirements: 3.6, 4.4_

  - [ ]* 13.6 Write property test for resource change propagation (Property 5)
    - **Property 5: Resource change propagation to mirrored copies**
    - Generate random resource changes on shared families, verify all mirrored copies are updated
    - **Validates: Requirements 3.6**

  - [ ]* 13.7 Write property test for write-back propagation (Property 19)
    - **Property 19: Write-back propagation from mirrored to original**
    - Generate random writes on mirrored resources, verify propagation to original partition
    - **Validates: Requirements 4.4**

  - [ ] 13.8 Implement `cleanup_mirrored` for share revocation
    - On share deletion, query all mirrored records with matching `share_id`
    - Delete all mirrored records from the accepter's partition
    - _Requirements: 7.2_

  - [ ]* 13.9 Write property test for revocation cleanup (Property 11)
    - **Property 11: Revocation cleans up mirrored records**
    - Generate random shares with mirrored records, revoke, verify all mirrored records deleted
    - **Validates: Requirements 7.2**

  - [ ] 13.10 Implement `update_mirrored_permissions` for permission scope updates
    - On share permission_scope change, update all mirrored records with matching `share_id`
    - _Requirements: 6.2_

  - [ ] 13.11 Add error handling with `ReportBatchItemFailures` support
    - Return partial failure responses so only failed records are retried
    - Ensure all operations are idempotent
    - _Requirements: 3.6_

- [ ] 14. Add stream handler infrastructure
  - [ ] 14.1 Add stream handler Lambda definition in `famtrac-infra/lib/backend/FamtracApi.ts`
    - Enable DynamoDB Streams on the table (StreamViewType: `NEW_AND_OLD_IMAGES`)
    - Create stream handler Lambda function
    - Add event source mapping from DynamoDB Stream to Lambda
    - Grant DynamoDB read/write permissions
    - Configure `ReportBatchItemFailures` on the event source mapping
    - _Requirements: 3.1, 3.6_

- [ ] 15. Handle share expiration
  - [ ] 15.1 Implement expiration check logic in share handlers
    - When retrieving shares, check if `pending` shares have exceeded the configurable expiration period
    - Treat expired shares as `Expired` status regardless of DynamoDB TTL
    - Exclude expired shares from active permission checks
    - Set `expires_in` TTL attribute on pending shares for DynamoDB auto-cleanup
    - _Requirements: 10.1, 10.2, 10.3_

  - [ ]* 15.2 Write property test for expired shares (Property 16)
    - **Property 16: Expired shares excluded from active checks**
    - Generate random shares with past expiration, verify they're excluded from active checks
    - **Validates: Requirements 10.1**

- [ ] 16. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- **IMPORTANT**: Files may change between task runs. Always read a file before editing it — never assume its contents match what was seen in a previous task.
- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- The stream handler (tasks 13-14) is a separate Lambda and can be developed in parallel with the API handlers (tasks 9-10) after the shared domain types are in place
- Existing handler tests may need updates in task 3 and task 8 due to new optional fields on domain structs

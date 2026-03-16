# Implementation Tasks

## Task 1: Update FamilyRepository trait to require owner_id on get
- [x] 1.1 In `famtrac-backend/src/repository/traits.rs`, change `FamilyRepository::get` signature from `get(&self, id: FamilyId)` to `get(&self, owner_id: IdentityId, id: FamilyId)` and add `IdentityId` to the imports
- [x] 1.2 In `famtrac-backend/src/repository/traits.rs`, update the doc comment on `get` to reflect that it now requires both owner_id and family_id
**Requirements:** Req 1.3, Req 3.1, Req 3.2
**Design Reference:** FamilyRepository Trait (Changed)

## Task 2: Update DynamoDbFamilyRepository for owner-scoped keys
- [x] 2.1 In `famtrac-backend/src/repository/dynamodb.rs`, update `DynamoDbFamilyRepository::to_item` to produce `PK=OWNER#{owner_id}` and `SK=FAMILY#{family_id}` instead of `PK=FAMILY#{family_id}` and `SK=METADATA`
- [x] 2.2 In `famtrac-backend/src/repository/dynamodb.rs`, update the `FamilyRepository::get` implementation to accept `owner_id: IdentityId` and use `PK=OWNER#{owner_id}`, `SK=FAMILY#{family_id}` for the GetItem call
- [x] 2.3 In `famtrac-backend/src/repository/dynamodb.rs`, update `FamilyRepository::get_by_owner` to query the main table with `PK=OWNER#{owner_id}` and SK prefix `FAMILY#` instead of using GSI-1
- [x] 2.4 In `famtrac-backend/src/repository/dynamodb.rs`, update `FamilyRepository::create` and `FamilyRepository::update` to use the new PK/SK format (they already derive from `family.owner_id`, just need the key format change)
**Requirements:** Req 1.1, Req 1.2, Req 1.4, Req 1.5, Req 1.6, Req 3.3, Req 3.4, Req 3.5
**Design Reference:** DynamoDbFamilyRepository (Changed), Key Schema Changes Summary

## Task 3: Update MockFamilyRepository for new get signature
- [x] 3.1 In `famtrac-backend/src/test_utils.rs`, update `MockFamilyRepository`'s `FamilyRepository::get` implementation to accept `owner_id: IdentityId` and return the family only when both owner_id and family_id match
- [x] 3.2 In `famtrac-backend/src/test_utils.rs`, verify `MockFamilyRepository::get_by_owner` filters by owner_id correctly (already does, no change needed beyond confirming)
**Requirements:** Req 6.1, Req 6.2, Req 6.3
**Design Reference:** FamilyRepository Trait (Changed), Testing Strategy

## Task 4: Update DynamoDbActivityRepository for UUID-based sort key and GSI query
- [x] 4.1 In `famtrac-backend/src/repository/dynamodb.rs`, update `DynamoDbActivityRepository::to_item` to produce `SK=ACTIVITY#{activity_id}` instead of `SK=ACTIVITY#{timestamp}#{activity_id}`
- [x] 4.2 In `famtrac-backend/src/repository/dynamodb.rs`, update `ActivityRepository::get` to use a direct GetItem call with `PK=FAMILY#{fid}#DEPENDENT#{did}` and `SK=ACTIVITY#{aid}` instead of query+filter
- [x] 4.3 In `famtrac-backend/src/repository/dynamodb.rs`, update `ActivityRepository::delete` to use a direct DeleteItem call with `PK=FAMILY#{fid}#DEPENDENT#{did}` and `SK=ACTIVITY#{aid}` instead of query-then-delete
- [x] 4.4 In `famtrac-backend/src/repository/dynamodb.rs`, update `ActivityRepository::query` to use the `Activity-Timestamp-GSI` with `timestamp` as the sort key for chronological ordering, moving date range filtering from filter expressions to key conditions
- [x] 4.5 In `famtrac-backend/src/repository/dynamodb.rs`, update `ActivityRepository::update` to use the new SK format `ACTIVITY#{activity_id}` in the PutItem call
**Requirements:** Req 2.1, Req 2.2, Req 2.3, Req 2.4, Req 2.5, Req 2.6
**Design Reference:** DynamoDbActivityRepository (Changed), Activity GSI, Activity Record (DynamoDB)

## Task 5: Update MockActivityRepository (no signature changes needed)
- [x] 5.1 In `famtrac-backend/src/test_utils.rs`, verify `MockActivityRepository::get` performs a direct lookup by activity_id (already does, confirm no changes needed)
- [x] 5.2 In `famtrac-backend/src/test_utils.rs`, verify `MockActivityRepository::delete` performs a direct removal by activity_id (already does, confirm no changes needed)
- [x] 5.3 In `famtrac-backend/src/test_utils.rs`, update `MockActivityRepository::query` to return activities sorted by timestamp in descending order to match the new GSI behavior
**Requirements:** Req 6.4, Req 6.5
**Design Reference:** ActivityRepository Trait (Unchanged Signatures), Testing Strategy

## Task 6: Update family handlers to use owner-scoped get and remove Authorizable
- [x] 6.1 In `famtrac-backend/src/handlers/family.rs`, update `get_family` to call `repository.get(context.identity_id.clone(), family_id)` instead of `repository.get(family_id)` and remove the `family.authorize(...)` call; remove the `D: DependentRepository` generic parameter
- [x] 6.2 In `famtrac-backend/src/handlers/family.rs`, update `update_family` to call `repository.get(context.identity_id.clone(), family_id)` instead of `repository.get(family_id)` and remove the `family.authorize(...)` call; remove the `D: DependentRepository` generic parameter
- [x] 6.3 In `famtrac-backend/src/handlers/family.rs`, remove the `use crate::authorization::Authorizable;` import
**Requirements:** Req 4.1, Req 4.2, Req 5.1, Req 5.2, Req 5.5
**Design Reference:** Handler Changes

## Task 7: Update dependent handlers to use owner-scoped get and remove Authorizable
- [x] 7.1 In `famtrac-backend/src/handlers/dependent.rs`, update `create_dependent` to call `family_repository.get(context.identity_id.clone(), request.family_id)` and remove the `family.authorize(...)` call; treat None as 404
- [x] 7.2 In `famtrac-backend/src/handlers/dependent.rs`, update `get_dependent` to call `family_repository.get(context.identity_id.clone(), family_id)` for authorization instead of `dependent.authorize(...)`, and treat None as 404
- [x] 7.3 In `famtrac-backend/src/handlers/dependent.rs`, update `update_dependent` to call `family_repository.get(context.identity_id.clone(), family_id)` for authorization instead of `dependent.authorize(...)`, and treat None as 404
- [x] 7.4 In `famtrac-backend/src/handlers/dependent.rs`, update `delete_dependent` to call `family_repository.get(context.identity_id.clone(), family_id)` for authorization instead of `dependent.authorize(...)`, and treat None as 404
- [x] 7.5 In `famtrac-backend/src/handlers/dependent.rs`, update `list_dependents` to call `family_repository.get(context.identity_id.clone(), family_id)` and remove the `family.authorize(...)` call
- [x] 7.6 In `famtrac-backend/src/handlers/dependent.rs`, remove the `use crate::authorization::Authorizable;` import
**Requirements:** Req 4.3, Req 4.4, Req 4.5, Req 5.3, Req 5.5
**Design Reference:** Handler Changes

## Task 8: Update activity handlers to use owner-scoped get and remove Authorizable
- [x] 8.1 In `famtrac-backend/src/handlers/activity.rs`, update `create_activity` to call `family_repository.get(context.identity_id.clone(), request.family_id)` for authorization instead of `dependent.authorize(...)`, and treat None as 404
- [x] 8.2 In `famtrac-backend/src/handlers/activity.rs`, update `get_activity` to call `family_repository.get(context.identity_id.clone(), family_id)` for authorization instead of `activity.authorize(...)`, and treat None as 404
- [x] 8.3 In `famtrac-backend/src/handlers/activity.rs`, update `update_activity` to call `family_repository.get(context.identity_id.clone(), family_id)` for authorization instead of `activity.authorize(...)`, and treat None as 404
- [x] 8.4 In `famtrac-backend/src/handlers/activity.rs`, update `delete_activity` to call `family_repository.get(context.identity_id.clone(), family_id)` for authorization instead of `activity.authorize(...)`, and treat None as 404
- [x] 8.5 In `famtrac-backend/src/handlers/activity.rs`, update `query_activities` to call `family_repository.get(context.identity_id.clone(), family_id)` for authorization instead of `dependent.authorize(...)`, and treat None as 404
- [x] 8.6 In `famtrac-backend/src/handlers/activity.rs`, remove the `use crate::authorization::Authorizable;` import
**Requirements:** Req 4.6, Req 5.3, Req 5.5
**Design Reference:** Handler Changes

## Task 9: Remove Authorizable trait and authorization module
- [x] 9.1 Delete or empty `famtrac-backend/src/authorization.rs`, removing the `Authorizable` trait and all its implementations for Family, Dependent, and Activity
- [x] 9.2 In `famtrac-backend/src/lib.rs`, remove `pub mod authorization;` from the module declarations
**Requirements:** Req 5.4
**Design Reference:** Authorization Module

## Task 10: Update all tests for new authorization flow
- [x] 10.1 In `famtrac-backend/src/handlers/family.rs` tests, update `test_get_family_success` and `test_update_family_success` to remove the `dependent_repo` parameter from handler calls
- [x] 10.2 In `famtrac-backend/src/handlers/family.rs` tests, update `test_get_family_unauthorized` and `test_update_family_unauthorized` to assert `HandlerError::NotFound` instead of `HandlerError::Auth(AuthError::Forbidden(_))`, and remove the `dependent_repo` parameter
- [x] 10.3 In `famtrac-backend/src/handlers/family.rs` tests, update all remaining family handler tests that pass `dependent_repo` to remove that parameter
- [x] 10.4 In `famtrac-backend/src/handlers/dependent.rs` tests, update `test_create_dependent_unauthorized`, `test_get_dependent_unauthorized`, `test_update_dependent_unauthorized`, and `test_list_dependents_unauthorized` to assert `HandlerError::NotFound` instead of `HandlerError::Auth(AuthError::Forbidden(_))`
- [x] 10.5 In `famtrac-backend/src/handlers/dependent.rs` tests, update mock setup in authorization tests so that the MockFamilyRepository enforces owner matching on `get` (families stored under owner A should not be returned when get is called with owner B)
- [x] 10.6 In `famtrac-backend/src/authorization.rs` tests, remove all tests (the file is being deleted in Task 9)
- [x] 10.7 Update the local mock `MockFamilyRepository` in `famtrac-backend/src/authorization.rs` tests to match the new trait signature (this is moot if the file is deleted first, but if tests are updated before deletion, the local mock must also accept `owner_id` in `get`)
**Requirements:** Req 6.1, Req 6.2
**Design Reference:** Testing Strategy, Error Handling - Handler Layer

## Task 11: Cleanup dead imports, unused code, and verify compilation
- [x] 11.1 Remove any remaining `use crate::authorization::Authorizable;` imports across the codebase
- [x] 11.2 Remove any unused `use crate::repository::DependentRepository;` imports from family handler files (no longer needed after removing the `D` generic parameter)
- [x] 11.3 Remove the `AuthError::Forbidden` variant if it is no longer used anywhere, or leave it if other code paths still reference it
- [x] 11.4 Run `cargo check` in `famtrac-backend/` to verify the project compiles without errors
- [x] 11.5 Run `cargo test` in `famtrac-backend/` to verify all tests pass
**Requirements:** Req 7.1, Req 7.2, Req 7.3
**Design Reference:** Architecture - Target State

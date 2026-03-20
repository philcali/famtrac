# Implementation Plan: Stream Handler Improvements

## Overview

Refactor the `famtrac-stream-handler` monolith into a modular classify-and-route architecture. Tasks are ordered to minimize rework: foundational module structure and shared utilities first, then feature work (GSI lookup, revocation cleanup, sync_token), then integration wiring.

## Tasks

- [x] 1. Create module structure and move DynamoDB utilities
  - [x] 1.1 Create `dynamo_util.rs` and extract all DynamoDB utility functions
    - Move `get_item`, `query_items`, `put_item`, `delete_item`, `conditional_put`, `conditional_update_permission`, `rekey_item`, `annotate_item`, `convert_image` from `main.rs` into `famtrac-stream-handler/src/dynamo_util.rs`
    - Add `find_active_shares_by_family_id` that queries the `family_id-index` GSI (replaces `find_owner_for_family` scan and `find_active_shares_for_family`)
    - Add `extract_family_id`, `extract_owner_id`, `is_mirrored_resource` helper functions
    - Relocate existing unit tests for `extract_family_id`, `extract_owner_id`, `is_mirrored_resource`, `rekey_item`, `annotate_item` into this module
    - _Requirements: 2.1, 2.2, 2.3, 5.1_

  - [ ]* 1.2 Write property test for owner ID extraction (Property 3)
    - **Property 3: Owner ID extraction returns None when absent**
    - Generate random `HashMap` images without `owner_id` and random PK strings not starting with `OWNER#`; assert `extract_owner_id` returns `None`
    - **Validates: Requirements 2.2**

  - [x] 1.3 Create unified share parser in `parser.rs`
    - Create `famtrac-stream-handler/src/parser.rs`
    - Implement `parse_share(image: &serde_dynamo::Item) -> Option<Share>` and `parse_share_from_attrs(item: &HashMap<String, DdbAttributeValue>) -> Option<Share>`
    - Both delegate to a shared `parse_share_from_strings(map: &HashMap<String, String>) -> Option<Share>` for identical logic
    - Remove `parse_share_from_image` and `parse_share_from_ddb_item` from `main.rs`
    - _Requirements: 4.1, 4.2_

  - [ ]* 1.4 Write property test for missing field yields None (Property 6)
    - **Property 6: Missing required field yields None from parser**
    - Generate valid Share data, remove one random required field, assert both `parse_share` and `parse_share_from_attrs` return `None`
    - **Validates: Requirements 4.3**

  - [ ]* 1.5 Write property test for parser equivalence (Property 7)
    - **Property 7: Parser equivalence across input formats**
    - Generate random valid `Share` values, convert to both `serde_dynamo::Item` and `HashMap<String, DdbAttributeValue>`, parse both, assert identical results
    - **Validates: Requirements 4.4**

- [ ] 2. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Extract classifier and implement RecordChange expansion
  - [x] 3.1 Create `classify.rs` with classifier logic
    - Move `classify_record`, `classify_share_record`, `get_str`, `record_type_from_sk`, `is_owner_partition`, `convert_image` into `famtrac-stream-handler/src/classify.rs`
    - Expand `RecordChange::ShareRevoked` from `ShareRevoked(ShareId)` to `ShareRevoked { share_id, family_id, accepter_id }` extracting fields from old image
    - Add `ChangeKind` enum and `change_kind(rc: &RecordChange) -> Option<ChangeKind>` function
    - Add semantic no-op detection: when producing `ResourceChanged`, strip `sync_token` from both images and compare; return `Ignored` if identical
    - Relocate all existing classifier unit tests from `main.rs` to `classify.rs`
    - _Requirements: 1.1, 3.6, 5.1, 5.3, 5.4, 6.3_

  - [ ]* 3.2 Write property test for revocation field extraction (Property 5)
    - **Property 5: Revocation extracts family_id and accepter_id from old image**
    - Generate random share old images with/without `family_id` and `accepter_id`; assert `ShareRevoked` carries extracted values or `Ignored` when fields missing
    - **Validates: Requirements 3.6**

  - [ ]* 3.3 Write property test for classifier equivalence (Property 8)
    - **Property 8: Refactored classifier produces identical output**
    - Generate random `EventRecord` values; assert refactored `classify_record` produces the same `RecordChange` variant as the original
    - **Validates: Requirements 5.3**

  - [ ]* 3.4 Write property test for semantic no-op detection (Property 10)
    - **Property 10: Semantic no-op changes are classified as Ignored**
    - Generate random `ResourceChanged` events where old and new images differ only by `sync_token`; assert classifier produces `Ignored`
    - **Validates: Requirements 6.3**

  - [ ]* 3.5 Write property test for strip function (Property 12)
    - **Property 12: Strip function removes only sync_token**
    - Generate random `HashMap` images with random `sync_token` values; assert stripping produces a map identical to original minus `sync_token` key
    - **Validates: Requirements 6.5**

- [ ] 4. Implement Router
  - [ ] 4.1 Create `router.rs` with Router struct
    - Create `famtrac-stream-handler/src/router.rs`
    - Implement `Router` with `HashMap<ChangeKind, Vec<HandlerFn>>` dispatch table
    - Implement `register(kind, handler)` and `dispatch(client, table_name, change)` methods
    - `dispatch` skips `Ignored` variants, invokes all handlers for a given `ChangeKind`, collects errors, returns first error if any
    - _Requirements: 1.2, 1.3, 1.5_

  - [ ]* 4.2 Write property test for router dispatch (Property 1)
    - **Property 1: Router dispatches to all matching handlers**
    - Register random sets of handlers for random `ChangeKind` values; dispatch random `RecordChange` variants; assert exactly the matching handlers were invoked
    - **Validates: Requirements 1.2, 1.3**

  - [ ]* 4.3 Write property test for router error isolation (Property 2)
    - **Property 2: Router error isolation**
    - Register handlers with random Ok/Err outcomes; assert all handlers execute regardless of failures
    - **Validates: Requirements 1.5**

- [ ] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Extract handler modules
  - [ ] 6.1 Create `handlers/mirror.rs` with share activation handler
    - Create `famtrac-stream-handler/src/handlers/mod.rs` and `famtrac-stream-handler/src/handlers/mirror.rs`
    - Move `mirror_resources` logic from `main.rs` into `handle_share_activated`
    - Stamp `sync_token` on every mirrored item written (via `rekey_item` and `annotate_item`)
    - Use functions from `dynamo_util.rs`
    - _Requirements: 5.1, 6.2_

  - [ ] 6.2 Create `handlers/revoke.rs` with share revocation cleanup handler
    - Create `famtrac-stream-handler/src/handlers/revoke.rs`
    - Implement `handle_share_revoked(client, table_name, share_id, family_id, accepter_id)` per design algorithm:
      1. Delete mirrored Family record at `PK=OWNER#{accepter_id}, SK=FAMILY#{family_id}`
      2. Query and delete all Dependents with matching `share_id`
      3. For each dependent, query and delete all Activities with matching `share_id`
    - All deletes are idempotent (delete on non-existent key succeeds)
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [ ]* 6.3 Write property test for revocation cleanup completeness (Property 4)
    - **Property 4: Revocation cleanup deletes all mirrored records for a share**
    - Generate random sets of items with various `share_id` values; assert revocation deletes exactly those matching the target `share_id`
    - **Validates: Requirements 3.2, 3.3, 3.4**

  - [ ] 6.4 Create `handlers/permission.rs` with permission update handler
    - Create `famtrac-stream-handler/src/handlers/permission.rs`
    - Move `update_mirrored_permissions` logic from `main.rs` into `handle_permission_updated`
    - Use functions from `dynamo_util.rs`
    - _Requirements: 5.1_

  - [ ] 6.5 Create `handlers/propagate.rs` with resource change propagation handler
    - Create `famtrac-stream-handler/src/handlers/propagate.rs`
    - Move `propagate_change`, `propagate_to_mirrors`, `propagate_writeback` from `main.rs`
    - Replace `find_owner_for_family` (scan) with `find_active_shares_by_family_id` (GSI query); derive owner from `requester_id` on returned shares
    - Add `sync_token` stamping on all writes in `propagate_to_mirrors` and `propagate_writeback`
    - Add `sync_token` presence check: skip propagation if new image contains `sync_token`
    - Add semantic diff before write-back: compare stripped new image against existing owner record, skip if identical
    - _Requirements: 2.1, 2.2, 2.3, 5.1, 6.1, 6.2, 6.4, 6.6, 6.7, 6.8_

  - [ ]* 6.6 Write property test for sync_token on handler writes (Property 9)
    - **Property 9: Handler writes always include sync_token**
    - Generate random items passed through the stamp function; assert every output contains a non-empty `sync_token` attribute
    - **Validates: Requirements 6.1, 6.2**

  - [ ]* 6.7 Write property test for sync_token propagation skip (Property 11)
    - **Property 11: sync_token presence determines propagation skip**
    - Generate random `ResourceChanged` events with/without `sync_token`; assert propagation is skipped iff `sync_token` is present
    - **Validates: Requirements 6.4, 6.6**

  - [ ]* 6.8 Write property test for no write-back on identical images (Property 13)
    - **Property 13: No write-back on semantically identical images**
    - Generate random image pairs (some identical, some different after stripping `sync_token`); assert `propagate_writeback` skips `put_item` when images match
    - **Validates: Requirements 6.7, 6.8**

- [ ] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Wire everything together in main.rs
  - [ ] 8.1 Rewrite `main.rs` as thin bootstrap
    - Reduce `main.rs` to: Lambda bootstrap, DDB client init, `sync_token` generation from request ID, Router construction with handler registration, top-level event dispatch loop
    - Register handlers: `mirror::handle_share_activated`, `revoke::handle_share_revoked`, `permission::handle_permission_updated`, `propagate::handle_resource_changed`
    - Remove all moved functions — `main.rs` should only contain bootstrap, Router setup, and the `handle_stream_event` / `process_record` dispatch loop
    - Preserve `StreamHandlerResponse` and `BatchItemFailure` structs (or move to a `types.rs` if cleaner)
    - Add `proptest` as a dev-dependency in `Cargo.toml`
    - Declare all new modules (`mod classify; mod router; mod parser; mod dynamo_util; mod handlers;`)
    - _Requirements: 1.2, 1.4, 5.2, 5.3_

  - [ ] 8.2 Remove `find_owner_for_family` table scan function
    - Delete the `find_owner_for_family` function and `find_active_shares_for_family` function from the codebase
    - Ensure all call sites now use `find_active_shares_by_family_id` from `dynamo_util.rs`
    - _Requirements: 2.3_

- [ ] 9. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests use the `proptest` crate with a minimum of 100 iterations each
- The `family_id-index` GSI must exist on the DynamoDB table before integration testing; table creation/update is outside the scope of this implementation (infrastructure concern)
- Existing unit tests are preserved and relocated to their owning modules throughout tasks 1, 3, and 6

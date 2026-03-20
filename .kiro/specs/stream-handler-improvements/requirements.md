# Requirements Document

## Introduction

The DynamoDB stream handler (`famtrac-stream-handler/src/main.rs`) is a monolithic ~1500-line Lambda function that classifies DynamoDB Stream records and processes them inline. It handles share activation mirroring, permission propagation, resource change fan-out, and write-back logic all in a single file. This spec covers a set of improvements: refactoring into a classify-and-route architecture with composable handlers, eliminating a full table scan used to resolve family ownership, implementing the missing share-revocation cleanup, and deduplicating share-parsing logic.

## Glossary

- **Stream_Handler**: The AWS Lambda function (`famtrac-stream-handler`) that processes DynamoDB Stream events from the FamtracData table.
- **Classifier**: The component responsible for inspecting a DynamoDB Stream `EventRecord` and producing a `RecordChange` variant that describes the event semantics.
- **Router**: The component that receives a classified `RecordChange` and dispatches it to one or more registered handler functions.
- **Handler_Function**: A composable, single-responsibility async function that processes one category of classified stream event (e.g. `share_activated`, `resource_changed`, `cleanup_mirrored`).
- **Mirrored_Record**: A copy of a Family, Dependent, or Activity record written into an accepter's DynamoDB partition, annotated with `share_id` and `permission_scope` attributes.
- **Owner_Partition**: The DynamoDB partition keyed by `PK = OWNER#{identity_id}` that holds a user's owned resources and share records.
- **Write_Back**: The process of propagating a change made on a Mirrored_Record back to the original Owner_Partition.
- **Table_Scan**: A DynamoDB `Scan` operation that reads every item in the table, filtering client-side. Expensive and does not scale.
- **GSI**: A Global Secondary Index on the DynamoDB table that enables efficient queries on non-key attributes.
- **Share_Parser**: The logic that converts a raw DynamoDB item (either `serde_dynamo::Item` or `HashMap<String, DdbAttributeValue>`) into a domain `Share` struct.
- **Handler_Origin_Marker**: A reserved attribute (e.g. `_stream_epoch`) stamped on every item written by the Stream_Handler. Used to distinguish handler-originated writes from user-originated writes and break infinite propagation cycles.

## Requirements

### Requirement 1: Classify-and-Route Architecture

**User Story:** As a developer, I want the stream handler to separate classification from processing through a router that dispatches to composable handler functions, so that new concerns (audit logging, notifications) can subscribe to stream events without growing the monolith.

#### Acceptance Criteria

1. THE Classifier SHALL produce a `RecordChange` variant for each DynamoDB Stream `EventRecord` without performing any side effects.
2. THE Router SHALL accept a classified `RecordChange` and dispatch it to all registered Handler_Functions that match the variant.
3. WHEN a new Handler_Function is registered with the Router, THE Router SHALL invoke the new Handler_Function for matching `RecordChange` variants without modifying existing Handler_Functions.
4. THE Stream_Handler SHALL organize classification, routing, and handler logic into separate Rust modules within the `famtrac-stream-handler/src/` directory.
5. WHEN a Handler_Function returns an error, THE Router SHALL report the failure for that record without preventing other Handler_Functions from executing for the same record.

### Requirement 2: Eliminate Table Scan in Family Owner Lookup

**User Story:** As a developer, I want the stream handler to resolve family ownership without a full table scan, so that the handler performs efficiently as the table grows.

#### Acceptance Criteria

1. THE Stream_Handler SHALL resolve the owner of a family using a DynamoDB Query operation instead of a Scan operation.
2. WHEN a Dependent or Activity change event lacks an `owner_id` attribute in the stream image, THE Stream_Handler SHALL look up the owner by querying a GSI or by querying the family record directly using the `family_id`.
3. THE Stream_Handler SHALL remove the `find_owner_for_family` function that performs a table Scan.

### Requirement 3: Implement Share Revocation Cleanup

**User Story:** As a developer, I want the stream handler to delete all Mirrored_Records when a share is revoked, so that accepters lose access to data they are no longer authorized to see.

#### Acceptance Criteria

1. WHEN a share record is removed from the Owner_Partition, THE Stream_Handler SHALL delete the Mirrored_Record of the family from the accepter's Owner_Partition.
2. WHEN a share record is removed from the Owner_Partition, THE Stream_Handler SHALL delete all Mirrored_Records of dependents that were created for that share.
3. WHEN a share record is removed from the Owner_Partition, THE Stream_Handler SHALL delete all Mirrored_Records of activities that were created for that share.
4. THE Stream_Handler SHALL identify Mirrored_Records belonging to a revoked share by matching the `share_id` attribute on each record.
5. IF a Mirrored_Record targeted for deletion does not exist, THEN THE Stream_Handler SHALL treat the deletion as successful (idempotent).
6. WHEN performing share revocation cleanup, THE Stream_Handler SHALL retrieve the `family_id` and `accepter_id` from the old image of the removed share record.

### Requirement 4: Deduplicate Share Parsing Logic

**User Story:** As a developer, I want a single share-parsing function that works with both DynamoDB image formats, so that parsing logic is maintained in one place and stays consistent.

#### Acceptance Criteria

1. THE Share_Parser SHALL provide a single entry point that parses a `Share` from either a `serde_dynamo::Item` or a `HashMap<String, DdbAttributeValue>`.
2. THE Stream_Handler SHALL remove the duplicated `parse_share_from_image` and `parse_share_from_ddb_item` functions and replace all call sites with the unified Share_Parser.
3. WHEN a share image is missing a required field, THE Share_Parser SHALL return `None` regardless of which input format was provided.
4. FOR ALL valid Share representations, parsing from `serde_dynamo::Item` and parsing from the equivalent `HashMap<String, DdbAttributeValue>` SHALL produce identical `Share` values (round-trip equivalence).

### Requirement 5: Module Decomposition of the Monolith

**User Story:** As a developer, I want the stream handler split into focused modules, so that each concern is isolated, testable, and navigable.

#### Acceptance Criteria

1. THE Stream_Handler SHALL separate the following concerns into distinct Rust modules: classification, mirroring, permission updates, change propagation, write-back, and DynamoDB utility functions.
2. THE Stream_Handler SHALL keep `main.rs` limited to Lambda bootstrap, DynamoDB client initialization, and top-level event dispatch.
3. WHEN all modules are compiled together, THE Stream_Handler SHALL produce the same observable behavior as the current monolithic implementation for all existing `RecordChange` variants.
4. THE Stream_Handler SHALL preserve all existing unit tests, relocating each test to the module that owns the function under test.

### Requirement 6: Break Infinite Write-Back Cycle in Change Propagation

**User Story:** As a developer, I want the stream handler to detect and suppress re-processing of its own writes during change propagation, so that mirrored-resource write-backs and owner-to-mirror fan-outs do not trigger an infinite stream processing loop.

#### Acceptance Criteria

1. WHEN `propagate_writeback` writes a record back to the Owner_Partition, THE Stream_Handler SHALL include a handler-origin marker attribute (e.g. `_stream_epoch`) on the written item that identifies the write as handler-originated.
2. WHEN `propagate_to_mirrors` writes a Mirrored_Record into an accepter's Owner_Partition, THE Stream_Handler SHALL include the same handler-origin marker attribute on the written item.
3. WHEN the Classifier produces a `ResourceChanged` variant, THE Stream_Handler SHALL compare the old and new images of the stream record and classify the change as `Ignored` if the images are semantically identical (no-op change).
4. WHEN `propagate_change` receives a `ResourceChanged` event whose new image contains a handler-origin marker attribute matching the current processing epoch, THE Stream_Handler SHALL skip propagation for that record.
5. THE Stream_Handler SHALL strip the handler-origin marker attribute before returning any item through a client-facing API or before comparing images for semantic equality.
6. IF the handler-origin marker attribute is missing from a stream record image, THEN THE Stream_Handler SHALL treat the record as a user-originated change and process it normally.
7. WHEN `propagate_writeback` processes a Family record in the Owner_Partition, THE Stream_Handler SHALL verify that the write-back produces a meaningful change by comparing the stripped new image against the existing owner record before issuing a `put_item`.
8. WHEN `propagate_writeback` processes a Dependent or Activity Mirrored_Record, THE Stream_Handler SHALL avoid re-writing the item with identical share metadata to prevent a stream event that re-triggers the write-back path.

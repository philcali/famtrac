# Requirements Document

## Introduction

Refactor the DynamoDB table design for the famtrac-backend to enforce tenant isolation at the data model level. Currently, the system relies on application-level authorization checks (the `Authorizable` trait) to verify that an identity owns a family before granting access. This refactor restructures the DynamoDB key schema so that data access patterns inherently enforce isolation — you cannot read or write data without providing the correct owner context. This eliminates the need for separate authorization lookups and simplifies the repository and handler layers.

## Glossary

- **Family_Table**: The DynamoDB table storing Family records, keyed by owner identity
- **Dependent_Table**: The DynamoDB table (or partition) storing Dependent records under a Family
- **Activity_Table**: The DynamoDB table (or partition) storing Activity records under a Family+Dependent
- **Owner_Id**: The authenticated identity identifier extracted from the request context (`identity_id`)
- **Family_Id**: A UUID uniquely identifying a Family within an owner's partition
- **Dependent_Id**: A UUID uniquely identifying a Dependent within a Family
- **Activity_Id**: A UUID uniquely identifying an Activity within a Family+Dependent partition
- **Partition_Key (PK)**: The DynamoDB partition key that determines data distribution and access scope
- **Sort_Key (SK)**: The DynamoDB sort key that determines ordering within a partition
- **Activity_Timestamp_GSI**: A Global Secondary Index on the Activity_Table that enables chronological ordering of activities
- **FamilyRepository**: The repository trait defining Family data access operations
- **DependentRepository**: The repository trait defining Dependent data access operations
- **ActivityRepository**: The repository trait defining Activity data access operations
- **RequestContext**: The struct carrying the authenticated identity_id for the current request
- **Authorizable_Trait**: The existing authorization trait that performs application-level ownership checks (to be simplified/removed)

## Requirements

### Requirement 1: Family Table Key Schema Refactor

**User Story:** As a developer, I want the Family table to use owner-scoped partition keys, so that listing families by owner is a natural partition query and cross-tenant access is structurally impossible.

#### Acceptance Criteria

1. THE Family_Table SHALL use `PK=OWNER#{owner_id}` and `SK=FAMILY#{family_id}` as the key schema for all Family records
2. WHEN a Family is created, THE DynamoDbFamilyRepository SHALL store the record with `PK=OWNER#{owner_id}` derived from the Family's owner_id field and `SK=FAMILY#{family_id}` derived from the Family's id field
3. WHEN a Family is retrieved by identifier, THE FamilyRepository::get method SHALL require both an Owner_Id parameter and a Family_Id parameter
4. WHEN families are listed for an owner, THE FamilyRepository::get_by_owner method SHALL execute a single partition query on `PK=OWNER#{owner_id}` with SK prefix `FAMILY#` without requiring a Global Secondary Index
5. WHEN a Family is updated, THE DynamoDbFamilyRepository SHALL use `PK=OWNER#{owner_id}` and `SK=FAMILY#{family_id}` to locate and overwrite the record
6. IF a get request specifies an Owner_Id that does not match the Family's partition, THEN THE DynamoDbFamilyRepository SHALL return None (item not found)

### Requirement 2: Activity Table Key Schema Refactor

**User Story:** As a developer, I want the Activity table to use a UUID-based sort key with a GSI for chronological ordering, so that single-activity lookups are direct GetItem calls and time-range queries use the GSI.

#### Acceptance Criteria

1. THE Activity_Table SHALL use `PK=FAMILY#{family_id}#DEPENDENT#{dependent_id}` and `SK=ACTIVITY#{activity_id}` as the key schema for all Activity records
2. WHEN an Activity is retrieved by identifier, THE ActivityRepository::get method SHALL execute a direct DynamoDB GetItem call using the composite PK and the `ACTIVITY#{activity_id}` sort key
3. WHEN an Activity is deleted, THE ActivityRepository::delete method SHALL execute a direct DynamoDB DeleteItem call using the composite PK and the `ACTIVITY#{activity_id}` sort key without requiring a preceding query
4. THE Activity_Timestamp_GSI SHALL have `PK=FAMILY#{family_id}#DEPENDENT#{dependent_id}` as the partition key and the activity timestamp as the sort key
5. WHEN activities are queried with date range filters, THE ActivityRepository::query method SHALL use the Activity_Timestamp_GSI to retrieve activities in chronological order
6. WHEN activities are queried, THE ActivityRepository::query method SHALL return results sorted by timestamp in descending order

### Requirement 3: FamilyRepository Trait Signature Changes

**User Story:** As a developer, I want the FamilyRepository trait to require owner context on all operations, so that the type system enforces that callers always provide tenant scope.

#### Acceptance Criteria

1. THE FamilyRepository::get method SHALL accept an Owner_Id parameter in addition to the Family_Id parameter
2. THE FamilyRepository::get method SHALL return None when no Family exists for the given Owner_Id and Family_Id combination
3. THE FamilyRepository::create method SHALL derive the partition key from the Family struct's owner_id field
4. THE FamilyRepository::update method SHALL derive the partition key from the Family struct's owner_id field
5. THE FamilyRepository::get_by_owner method SHALL query the main table partition `PK=OWNER#{owner_id}` instead of using a Global Secondary Index

### Requirement 4: Handler Layer Owner Context Propagation

**User Story:** As a developer, I want all handlers to pass the authenticated identity_id as owner context to repository calls, so that tenant isolation is enforced at every data access point.

#### Acceptance Criteria

1. WHEN the get_family handler retrieves a Family, THE get_family handler SHALL pass the RequestContext identity_id as the Owner_Id parameter to FamilyRepository::get
2. WHEN the update_family handler retrieves a Family for update, THE update_family handler SHALL pass the RequestContext identity_id as the Owner_Id parameter to FamilyRepository::get
3. WHEN the create_dependent handler verifies the parent Family, THE create_dependent handler SHALL pass the RequestContext identity_id as the Owner_Id parameter to FamilyRepository::get
4. WHEN the get_dependent handler authorizes access, THE get_dependent handler SHALL pass the RequestContext identity_id as the Owner_Id parameter to FamilyRepository::get
5. WHEN the list_dependents handler authorizes access, THE list_dependents handler SHALL pass the RequestContext identity_id as the Owner_Id parameter to FamilyRepository::get
6. WHEN any activity handler authorizes access to the parent Family, THE activity handler SHALL pass the RequestContext identity_id as the Owner_Id parameter to FamilyRepository::get

### Requirement 5: Authorization Simplification

**User Story:** As a developer, I want to simplify the authorization model since data access patterns now enforce tenant isolation, so that the codebase has less indirection and fewer repository lookups.

#### Acceptance Criteria

1. WHEN a Family is retrieved using the owner-scoped FamilyRepository::get, THE handler SHALL treat a successful retrieval as implicit authorization (the owner context in the key guarantees ownership)
2. WHEN a handler retrieves a Family and receives None, THE handler SHALL return a 404 Not Found response without a separate authorization check
3. WHEN a Dependent or Activity handler needs to verify Family ownership, THE handler SHALL call FamilyRepository::get with the RequestContext identity_id and treat a None result as access denied
4. THE Authorizable_Trait implementations for Family, Dependent, and Activity SHALL be removed or simplified since the data access pattern enforces isolation
5. THE get_family, update_family, get_dependent, update_dependent, list_dependents, and delete_dependent handlers SHALL remove explicit Authorizable::authorize calls

### Requirement 6: Mock Repository Updates

**User Story:** As a developer, I want the mock repositories to match the updated trait signatures, so that unit tests continue to compile and correctly test the new access patterns.

#### Acceptance Criteria

1. THE MockFamilyRepository SHALL implement the updated FamilyRepository::get method accepting both Owner_Id and Family_Id parameters
2. WHEN MockFamilyRepository::get is called, THE MockFamilyRepository SHALL return the Family only when both the Owner_Id and Family_Id match a stored record
3. THE MockFamilyRepository::get_by_owner method SHALL filter stored families by owner_id without using a GSI simulation
4. THE MockActivityRepository SHALL implement the updated ActivityRepository::get method that performs a direct lookup by Family_Id, Dependent_Id, and Activity_Id
5. THE MockActivityRepository SHALL implement the updated ActivityRepository::delete method that performs a direct removal by Family_Id, Dependent_Id, and Activity_Id

### Requirement 7: Dependent Table Key Schema Preservation

**User Story:** As a developer, I want to confirm the Dependent table key schema remains unchanged, so that the refactor scope is clear and the Dependent access pattern is preserved.

#### Acceptance Criteria

1. THE Dependent_Table SHALL continue to use `PK=FAMILY#{family_id}` and `SK=DEPENDENT#{dependent_id}` as the key schema
2. THE DependentRepository trait method signatures SHALL remain unchanged
3. THE DynamoDbDependentRepository implementation SHALL remain unchanged

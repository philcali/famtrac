# Common Test Utilities

This module provides shared test utilities to avoid code duplication across test files.

## Mock Repositories (`mocks.rs`)

Reusable mock implementations of repository traits for testing without DynamoDB.

### Available Mocks

- `MockFamilyRepository` - Mock implementation of `FamilyRepository`
- `MockDependentRepository` - Mock implementation of `DependentRepository`
- `MockActivityRepository` - Mock implementation of `ActivityRepository`

### Usage

```rust
mod common;

use common::mocks::{MockFamilyRepository, MockDependentRepository};

#[test]
fn test_something() {
    let family_repo = MockFamilyRepository::new();
    let dependent_repo = MockDependentRepository::new();
    
    // Use the mocks in your test...
}
```

### Features

Each mock repository provides:

- `new()` - Create a new mock with default behavior
- `with_failure()` - Create a mock that returns errors for all operations
- `insert()` - Pre-populate the mock with test data
- Full trait implementation with in-memory storage using `HashMap`

### Example: Testing with Pre-populated Data

```rust
use common::mocks::MockFamilyRepository;
use famtrac_backend::domain::{Family, FamilyId, IdentityId, Timestamp};

#[test]
fn test_with_existing_family() {
    let repo = MockFamilyRepository::new();
    
    // Pre-populate with test data
    let family = Family {
        id: FamilyId::new(),
        name: "Test Family".to_string(),
        owner_id: IdentityId::new("user-123".to_string()),
        created_at: Timestamp::now(),
        updated_at: Timestamp::now(),
    };
    repo.insert(family.clone());
    
    // Now test operations on existing data
    let result = repo.get(family.id);
    assert!(result.is_ok());
}
```

### Example: Testing Error Handling

```rust
use common::mocks::MockFamilyRepository;

#[test]
fn test_error_handling() {
    let repo = MockFamilyRepository::with_failure();
    
    // All operations will return StoreError::QueryError
    let result = repo.get(FamilyId::new());
    assert!(result.is_err());
}
```

## DynamoDB Local (`mod.rs`)

Utilities for integration testing with DynamoDB Local. See the main module documentation for details.

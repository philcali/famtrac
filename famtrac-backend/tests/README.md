# Integration Tests

This directory contains integration tests for the famtrac-backend API, including property-based tests that validate correctness properties.

## Running Property-Based Tests

The property-based tests require DynamoDB Local to be installed in the workspace.

### One-Time Setup

Run the setup script to download and install DynamoDB Local:

```bash
./scripts/setup-dynamodb-local.sh
```

This script will:
- Download DynamoDB Local from AWS
- Extract it to the `dynamodb/` directory
- Display the SHA256 checksum for verification
- Provide instructions for running tests

### Running the Tests

Once DynamoDB Local is installed, run the property tests with:

```bash
# Run all property tests
cargo test --test property_resource_creation_roundtrip

# Run with verbose output
cargo test --test property_resource_creation_roundtrip -- --nocapture
```

The test utilities automatically:
- Start DynamoDB Local on a random available port
- Create the test table with proper schema
- Run the property tests
- Clean up and stop DynamoDB Local

Note: Tests run sequentially by default to avoid port conflicts.

## Test Structure

- `common/mod.rs` - Test utilities for DynamoDB Local management
  - `DynamoDbLocalInstance` - Manages DynamoDB Local process lifecycle
  - Automatic table creation and cleanup
  - Random port allocation for test isolation

- `property_resource_creation_roundtrip.rs` - Property 1: Resource Creation Round-Trip
  - Tests that creating and retrieving Family, Dependent, and Activity resources returns equivalent data
  - Uses proptest with 100 iterations per property
  - Validates Requirements 1.1, 1.3, 2.1, 2.3, 3.1

## Property-Based Testing

Property-based tests use the `proptest` crate to generate random test cases. Each test:
- Generates valid domain objects using `Arbitrary` strategies
- Performs operations (create, retrieve, update, etc.)
- Verifies correctness properties hold across all generated inputs

The tests are configured to run 100 iterations per property to ensure adequate coverage.

## Requirements

- Java Runtime Environment (JRE) - Required to run DynamoDB Local
- DynamoDB Local JAR - Installed via `scripts/setup-dynamodb-local.sh`

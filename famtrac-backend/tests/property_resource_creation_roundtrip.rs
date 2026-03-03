// Feature: famtrac-backend, Property 1: Resource Creation Round-Trip
// Validates: Requirements 1.1, 1.3, 2.1, 2.3, 3.1

mod common;

use common::DynamoDbLocalInstance;
use famtrac_backend::domain::{
    Activity, ActivityId, ActivityType, Date, Dependent, DependentId, DiaperContents, Family,
    FamilyId, FeedingType, IdentityId, Timestamp,
};
use famtrac_backend::repository::{
    ActivityRepository, DependentRepository, DynamoDbActivityRepository,
    DynamoDbDependentRepository, DynamoDbFamilyRepository, FamilyRepository,
};
use proptest::prelude::*;
use std::sync::Arc;

// Test configuration
const PROPTEST_CASES: u32 = 100;

// Arbitrary instance for FamilyId
fn arbitrary_family_id() -> impl Strategy<Value = FamilyId> {
    any::<[u8; 16]>().prop_map(|bytes| FamilyId(uuid::Uuid::from_bytes(bytes)))
}

// Arbitrary instance for DependentId
fn arbitrary_dependent_id() -> impl Strategy<Value = DependentId> {
    any::<[u8; 16]>().prop_map(|bytes| DependentId(uuid::Uuid::from_bytes(bytes)))
}

// Arbitrary instance for ActivityId
fn arbitrary_activity_id() -> impl Strategy<Value = ActivityId> {
    any::<[u8; 16]>().prop_map(|bytes| ActivityId(uuid::Uuid::from_bytes(bytes)))
}

// Arbitrary instance for IdentityId
fn arbitrary_identity_id() -> impl Strategy<Value = IdentityId> {
    "[a-zA-Z0-9_-]{10,50}".prop_map(IdentityId::new)
}

// Arbitrary instance for Timestamp
fn arbitrary_timestamp() -> impl Strategy<Value = Timestamp> {
    use chrono::{Duration, Utc};
    // Generate timestamps within a reasonable range (past year to now)
    let now = Utc::now();
    let year_ago = now - Duration::days(365);

    (year_ago.timestamp()..=now.timestamp())
        .prop_map(|ts| Timestamp::from_datetime(chrono::DateTime::from_timestamp(ts, 0).unwrap()))
}

// Arbitrary instance for Date
fn arbitrary_date() -> impl Strategy<Value = Date> {
    use chrono::Utc;
    // Generate dates within a reasonable range (past 10 years to today)
    let today = Utc::now().date_naive();
    let ten_years_ago = today - chrono::Days::new(3650);

    // Convert to timestamps for range generation
    let start_ts = ten_years_ago
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let end_ts = today.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

    (start_ts..=end_ts).prop_map(|ts| {
        let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        Date::from_naive_date(dt.date_naive())
    })
}

// Arbitrary instance for valid family name (1-100 characters, non-empty after trimming)
fn arbitrary_family_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,100}".prop_filter("name must be non-empty after trimming", |s| {
        !s.trim().is_empty()
    })
}

// Arbitrary instance for Family
fn arbitrary_family() -> impl Strategy<Value = Family> {
    (
        arbitrary_family_id(),
        arbitrary_family_name(),
        arbitrary_identity_id(),
        arbitrary_timestamp(),
        arbitrary_timestamp(),
    )
        .prop_map(|(id, name, owner_id, created_at, updated_at)| Family {
            id,
            name,
            owner_id,
            created_at,
            updated_at,
        })
}

// Arbitrary instance for valid dependent name (1-100 characters, non-empty after trimming)
fn arbitrary_dependent_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,100}".prop_filter("name must be non-empty after trimming", |s| {
        !s.trim().is_empty()
    })
}

// Arbitrary instance for Dependent
fn arbitrary_dependent() -> impl Strategy<Value = Dependent> {
    (
        arbitrary_dependent_id(),
        arbitrary_family_id(),
        arbitrary_dependent_name(),
        arbitrary_date(),
        arbitrary_timestamp(),
        arbitrary_timestamp(),
    )
        .prop_map(
            |(id, family_id, name, date_of_birth, created_at, updated_at)| Dependent {
                id,
                family_id,
                name,
                date_of_birth,
                created_at,
                updated_at,
            },
        )
}

// Arbitrary instance for FeedingType
fn arbitrary_feeding_type() -> impl Strategy<Value = FeedingType> {
    prop_oneof![
        Just(FeedingType::Breast),
        Just(FeedingType::Bottle),
        Just(FeedingType::Solid),
    ]
}

// Arbitrary instance for DiaperContents
fn arbitrary_diaper_contents() -> impl Strategy<Value = DiaperContents> {
    prop_oneof![
        Just(DiaperContents::Wet),
        Just(DiaperContents::Dirty),
        Just(DiaperContents::Both),
    ]
}

// Arbitrary instance for ActivityType
fn arbitrary_activity_type() -> impl Strategy<Value = ActivityType> {
    prop_oneof![
        // Feeding with optional volume
        (arbitrary_feeding_type(), proptest::option::of(1u32..1000)).prop_map(
            |(feeding_type, volume_ml)| ActivityType::Feeding {
                feeding_type,
                volume_ml,
            }
        ),
        // DiaperChange
        arbitrary_diaper_contents().prop_map(|contents| ActivityType::DiaperChange { contents }),
        // Sleep with start < end
        arbitrary_timestamp().prop_flat_map(|start| {
            let start_ts = start.0.timestamp();
            (start_ts + 60..=start_ts + 86400).prop_map(move |end_ts| {
                let end =
                    Timestamp::from_datetime(chrono::DateTime::from_timestamp(end_ts, 0).unwrap());
                ActivityType::Sleep { start, end }
            })
        }),
        // Pumping with required volume
        (1u32..1000).prop_map(|volume_ml| ActivityType::Pumping { volume_ml }),
    ]
}

// Arbitrary instance for Activity
fn arbitrary_activity() -> impl Strategy<Value = Activity> {
    (
        arbitrary_activity_id(),
        arbitrary_dependent_id(),
        arbitrary_timestamp(),
        arbitrary_activity_type(),
        arbitrary_timestamp(),
        arbitrary_timestamp(),
    )
        .prop_map(
            |(id, dependent_id, timestamp, activity_type, created_at, updated_at)| Activity {
                id,
                dependent_id,
                timestamp,
                activity_type,
                created_at,
                updated_at,
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_family_creation_roundtrip() {
        // Create a single runtime for the entire test
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Start DynamoDB Local instance
        let instance = rt
            .block_on(DynamoDbLocalInstance::start(
                "famtrac-test-family".to_string(),
            ))
            .expect("Failed to start DynamoDB Local. Run scripts/setup-dynamodb-local.sh first.");

        let repo = Arc::new(DynamoDbFamilyRepository::new(
            instance.client.clone(),
            instance.config.table_name.clone(),
        ));

        // Enter the runtime context so Handle::current() works in proptest
        let _guard = rt.enter();

        proptest!(ProptestConfig::with_cases(PROPTEST_CASES), |(family in arbitrary_family())| {
            let repo = Arc::clone(&repo);
            // Now we're in the runtime context, so Handle::current().block_on() will work
            // Create the family
            let created = repo.create(family.clone()).expect("Failed to create family");

            // Retrieve the family
            let retrieved = repo.get(family.id).expect("Failed to get family");

            // Verify the family was retrieved and matches
            assert!(retrieved.is_some(), "Family should exist after creation");
            let retrieved = retrieved.unwrap();

            // Verify all fields match
            assert_eq!(retrieved.id, created.id);
            assert_eq!(retrieved.name, created.name);
            assert_eq!(retrieved.owner_id, created.owner_id);
            assert_eq!(retrieved.created_at, created.created_at);
            assert_eq!(retrieved.updated_at, created.updated_at);
        });

        // Drop the guard to exit runtime context before cleanup
        drop(_guard);

        // Cleanup
        rt.block_on(instance.delete_test_table())
            .expect("Failed to cleanup test table");
    }

    #[test]
    #[ignore]
    fn test_dependent_creation_roundtrip() {
        // Create a single runtime for the entire test
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Start DynamoDB Local instance
        let instance = rt
            .block_on(DynamoDbLocalInstance::start(
                "famtrac-test-dependent".to_string(),
            ))
            .expect("Failed to start DynamoDB Local. Run scripts/setup-dynamodb-local.sh first.");

        let repo = Arc::new(DynamoDbDependentRepository::new(
            instance.client.clone(),
            instance.config.table_name.clone(),
        ));

        // Enter the runtime context so Handle::current() works in proptest
        let _guard = rt.enter();

        proptest!(ProptestConfig::with_cases(PROPTEST_CASES), |(dependent in arbitrary_dependent())| {
            let repo = Arc::clone(&repo);
            // Now we're in the runtime context, so Handle::current().block_on() will work
            // Create the dependent
            let created = repo.create(dependent.clone()).expect("Failed to create dependent");

            // Retrieve the dependent
            let retrieved = repo.get(dependent.id).expect("Failed to get dependent");

            // Verify the dependent was retrieved and matches
            assert!(retrieved.is_some(), "Dependent should exist after creation");
            let retrieved = retrieved.unwrap();

            // Verify all fields match
            assert_eq!(retrieved.id, created.id);
            assert_eq!(retrieved.family_id, created.family_id);
            assert_eq!(retrieved.name, created.name);
            assert_eq!(retrieved.date_of_birth, created.date_of_birth);
            assert_eq!(retrieved.created_at, created.created_at);
            assert_eq!(retrieved.updated_at, created.updated_at);
        });

        // Drop the guard to exit runtime context before cleanup
        drop(_guard);

        // Cleanup
        rt.block_on(instance.delete_test_table())
            .expect("Failed to cleanup test table");
    }

    #[test]
    #[ignore]
    fn test_activity_creation_roundtrip() {
        // Create a single runtime for the entire test
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Start DynamoDB Local instance
        let instance = rt
            .block_on(DynamoDbLocalInstance::start(
                "famtrac-test-activity".to_string(),
            ))
            .expect("Failed to start DynamoDB Local. Run scripts/setup-dynamodb-local.sh first.");

        let repo = Arc::new(DynamoDbActivityRepository::new(
            instance.client.clone(),
            instance.config.table_name.clone(),
        ));

        // Enter the runtime context so Handle::current() works in proptest
        let _guard = rt.enter();

        proptest!(ProptestConfig::with_cases(PROPTEST_CASES), |(activity in arbitrary_activity())| {
            let repo = Arc::clone(&repo);
            // Now we're in the runtime context, so Handle::current().block_on() will work
            // Create the activity
            let created = repo.create(activity.clone()).expect("Failed to create activity");

            // Retrieve the activity
            let retrieved = repo.get(activity.id).expect("Failed to get activity");

            // Verify the activity was retrieved and matches
            assert!(retrieved.is_some(), "Activity should exist after creation");
            let retrieved = retrieved.unwrap();

            // Verify all fields match
            assert_eq!(retrieved.id, created.id);
            assert_eq!(retrieved.dependent_id, created.dependent_id);
            assert_eq!(retrieved.timestamp, created.timestamp);
            assert_eq!(retrieved.activity_type, created.activity_type);
            assert_eq!(retrieved.created_at, created.created_at);
            assert_eq!(retrieved.updated_at, created.updated_at);
        });

        // Drop the guard to exit runtime context before cleanup
        drop(_guard);

        // Cleanup
        rt.block_on(instance.delete_test_table())
            .expect("Failed to cleanup test table");
    }
}

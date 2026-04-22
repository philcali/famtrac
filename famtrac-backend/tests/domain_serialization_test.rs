use chrono::{DateTime, NaiveDate, Utc};
use famtrac_backend::domain::*;
use famtrac_backend::handlers::{CreateActivityRequest, UpdateActivityRequest};

#[test]
fn test_timestamp_iso8601_serialization() {
    let dt = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let timestamp = Timestamp::from_datetime(dt);

    let json = serde_json::to_string(&timestamp).unwrap();
    assert_eq!(json, "\"2024-01-15T10:30:00Z\"");

    let deserialized: Timestamp = serde_json::from_str(&json).unwrap();
    assert_eq!(timestamp, deserialized);
}

#[test]
fn test_date_serialization() {
    let date = Date::from_naive_date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

    let json = serde_json::to_string(&date).unwrap();
    assert_eq!(json, "\"2024-01-15\"");

    let deserialized: Date = serde_json::from_str(&json).unwrap();
    assert_eq!(date, deserialized);
}

#[test]
fn test_family_serialization() {
    let family = Family {
        id: FamilyId::new(),
        name: "Test Family".to_string(),
        owner_id: IdentityId::new("test-owner-123".to_string()),
        created_at: Timestamp::from_datetime(
            DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ),
        updated_at: Timestamp::from_datetime(
            DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ),
        share_id: None,
        permission_scope: None,
    };

    let json = serde_json::to_string(&family).unwrap();
    let deserialized: Family = serde_json::from_str(&json).unwrap();

    assert_eq!(family.id, deserialized.id);
    assert_eq!(family.name, deserialized.name);
    assert_eq!(family.owner_id, deserialized.owner_id);
    assert_eq!(family.created_at, deserialized.created_at);
    assert_eq!(family.updated_at, deserialized.updated_at);
}

#[test]
fn test_activity_type_serialization() {
    let feeding = ActivityType::Feeding {
        feeding_type: FeedingType::Bottle,
        volume_ml: Some(120),
        medicine_added: Some(true),
    };

    let json = serde_json::to_string(&feeding).unwrap();
    assert!(json.contains("\"type\":\"feeding\""));
    assert!(json.contains("\"feeding_type\":\"bottle\""));
    assert!(json.contains("\"volume_ml\":120"));

    let deserialized: ActivityType = serde_json::from_str(&json).unwrap();
    assert_eq!(feeding, deserialized);
}

#[test]
fn test_sleep_activity_serialization() {
    let sleep = ActivityType::Sleep {
        start_time: Timestamp::from_datetime(
            DateTime::parse_from_rfc3339("2024-01-15T22:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ),
        end_time: Some(Timestamp::from_datetime(
            DateTime::parse_from_rfc3339("2024-01-16T06:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        )),
    };

    let json = serde_json::to_string(&sleep).unwrap();
    assert!(json.contains("\"type\":\"sleep\""));
    assert!(json.contains("\"start_time\":\"2024-01-15T22:00:00Z\""));
    assert!(json.contains("\"end_time\":\"2024-01-16T06:00:00Z\""));

    let deserialized: ActivityType = serde_json::from_str(&json).unwrap();
    assert_eq!(sleep, deserialized);
}

#[test]
fn test_feeding_with_medicine_deserialize() {
    let json = r#"{"family_id":"00000000-0000-0000-0000-000000000001","dependent_id":"00000000-0000-0000-0000-000000000002","timestamp":"2024-01-15T10:30:00Z","type":"feeding","feeding_type":"bottle","medicine_added":true,"volume_ml":120}"#;
    let request: CreateActivityRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(
        request.activity_type,
        ActivityType::Feeding {
            feeding_type: FeedingType::Bottle,
            medicine_added: Some(true),
            ..
        }
    ));
}

#[test]
fn test_feeding_without_medicine_deserialize() {
    let json = r#"{"family_id":"00000000-0000-0000-0000-000000000001","dependent_id":"00000000-0000-0000-0000-000000000002","timestamp":"2024-01-15T10:30:00Z","type":"feeding","feeding_type":"bottle","volume_ml":null}"#;
    let request: CreateActivityRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(
        request.activity_type,
        ActivityType::Feeding {
            feeding_type: FeedingType::Bottle,
            medicine_added: None,
            ..
        }
    ));
}

#[test]
fn test_update_activity_with_medicine() {
    let json = r#"{"timestamp":"2024-01-15T10:30:00Z","type":"feeding","feeding_type":"bottle","medicine_added":true}"#;
    let request: UpdateActivityRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(
        request.activity_type,
        ActivityType::Feeding {
            feeding_type: FeedingType::Bottle,
            medicine_added: Some(true),
            ..
        }
    ));
}

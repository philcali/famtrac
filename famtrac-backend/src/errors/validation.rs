use super::ValidationError;
use crate::domain::{ActivityType, Date, Timestamp};

/// Validate Family name
/// Requirements: 1.2 - Return descriptive error messages for invalid Family data
pub fn validate_family_name(name: &str) -> Result<(), ValidationError> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(ValidationError {
            field: "name".to_string(),
            message: "Family name cannot be empty".to_string(),
            constraint: Some("must be non-empty after trimming".to_string()),
        });
    }

    if name.len() > 100 {
        return Err(ValidationError {
            field: "name".to_string(),
            message: "Family name is too long".to_string(),
            constraint: Some("must be between 1 and 100 characters".to_string()),
        });
    }

    Ok(())
}

/// Validate Dependent name
/// Requirements: 2.2 - Return descriptive error messages for invalid Dependent data
pub fn validate_dependent_name(name: &str) -> Result<(), ValidationError> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(ValidationError {
            field: "name".to_string(),
            message: "Dependent name cannot be empty".to_string(),
            constraint: Some("must be non-empty after trimming".to_string()),
        });
    }

    if name.len() > 100 {
        return Err(ValidationError {
            field: "name".to_string(),
            message: "Dependent name is too long".to_string(),
            constraint: Some("must be between 1 and 100 characters".to_string()),
        });
    }

    Ok(())
}

/// Validate Dependent date of birth
/// Requirements: 2.2 - Return descriptive error messages for invalid Dependent data
pub fn validate_date_of_birth(date_of_birth: &Date) -> Result<(), ValidationError> {
    let today = Date::today();

    if date_of_birth.0 > today.0 {
        return Err(ValidationError {
            field: "date_of_birth".to_string(),
            message: "Date of birth cannot be in the future".to_string(),
            constraint: Some("must not be in the future".to_string()),
        });
    }

    Ok(())
}

/// Validate Activity timestamp
/// Requirements: 3.3 - Validate Activity timestamp is not in the future
/// Requirements: 3.4 - Return descriptive error messages for invalid Activity data
pub fn validate_activity_timestamp(timestamp: &Timestamp) -> Result<(), ValidationError> {
    let now = Timestamp::now();

    if timestamp.0 > now.0 {
        return Err(ValidationError {
            field: "timestamp".to_string(),
            message: "Activity timestamp cannot be in the future".to_string(),
            constraint: Some("must not be in the future".to_string()),
        });
    }

    Ok(())
}

/// Validate Activity type-specific attributes
/// Requirements: 7.1, 7.2, 7.3, 7.4 - Validate type-specific attributes for each activity type
/// Requirements: 7.5 - Return descriptive error messages when type-specific validation fails
pub fn validate_activity_type(activity_type: &ActivityType) -> Result<(), ValidationError> {
    match activity_type {
        ActivityType::Feeding {
            feeding_type: _,
            volume_ml,
        } => {
            // feeding_type is required by the enum structure
            // Validate volume_ml if present
            if let Some(volume) = volume_ml {
                if *volume == 0 {
                    return Err(ValidationError {
                        field: "volume_ml".to_string(),
                        message: "Feeding volume must be greater than zero".to_string(),
                        constraint: Some("must be > 0".to_string()),
                    });
                }
            }
            Ok(())
        }
        ActivityType::DiaperChange { contents: _ } => {
            // contents is required by the enum structure
            Ok(())
        }
        ActivityType::Sleep {
            start_time,
            end_time,
        } => {
            // Both start and end are required by the enum structure
            // Validate that end is after start
            if let Some(end) = end_time {
                if end.0 <= start_time.0 {
                    return Err(ValidationError {
                        field: "end_time".to_string(),
                        message: "Sleep end time must be after start time".to_string(),
                        constraint: Some("end must be > start".to_string()),
                    });
                }
            }
            Ok(())
        }
        ActivityType::Pumping { volume_ml } => {
            // volume_ml is required by the enum structure
            // Validate that it's greater than zero
            if *volume_ml == 0 {
                return Err(ValidationError {
                    field: "volume_ml".to_string(),
                    message: "Pumping volume must be greater than zero".to_string(),
                    constraint: Some("must be > 0".to_string()),
                });
            }
            Ok(())
        }
        ActivityType::ActivityTime {
            start_time,
            end_time,
            description,
        } => {
            if let Some(end) = end_time {
                if end.0 <= start_time.0 {
                    return Err(ValidationError {
                        field: "end_time".to_string(),
                        message: "Activity time end time must be after start time".to_string(),
                        constraint: Some("end must be > start".to_string()),
                    });
                }
            }
            if let Some(desc) = description {
                let sanitized = sanitize_string(desc);
                if sanitized.len() > 500 {
                    return Err(ValidationError {
                        field: "description".to_string(),
                        message: "Description is too long".to_string(),
                        constraint: Some("must be 500 characters or fewer".to_string()),
                    });
                }
            }
            Ok(())
        }
        ActivityType::TummyTime {
            start_time,
            end_time,
            notes,
        } => {
            if let Some(end) = end_time {
                if end.0 <= start_time.0 {
                    return Err(ValidationError {
                        field: "end_time".to_string(),
                        message: "Tummy time end time must be after start time".to_string(),
                        constraint: Some("end must be > start".to_string()),
                    });
                }
            }
            if let Some(n) = notes {
                let sanitized = sanitize_string(n);
                if sanitized.len() > 500 {
                    return Err(ValidationError {
                        field: "notes".to_string(),
                        message: "Notes is too long".to_string(),
                        constraint: Some("must be 500 characters or fewer".to_string()),
                    });
                }
            }
            Ok(())
        }
        ActivityType::WakeWindow {
            start_time,
            end_time,
        } => {
            if let Some(end) = end_time {
                if end.0 <= start_time.0 {
                    return Err(ValidationError {
                        field: "end_time".to_string(),
                        message: "Wake window end time must be after start time".to_string(),
                        constraint: Some("end must be > start".to_string()),
                    });
                }
            }
            Ok(())
        }
    }
}

/// Sanitize string input to prevent injection attacks
/// Requirements: 8.4 - Sanitize all string inputs to prevent injection attacks
pub fn sanitize_string(input: &str) -> String {
    // Remove null bytes
    let mut sanitized = input.replace('\0', "");

    // Remove control characters except newline, carriage return, and tab
    sanitized = sanitized
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect();

    // Trim leading and trailing whitespace
    sanitized.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DiaperContents, FeedingType};
    use chrono::{Duration, NaiveDate, Utc};

    #[test]
    fn test_validate_family_name_valid() {
        assert!(validate_family_name("Smith Family").is_ok());
        assert!(validate_family_name("A").is_ok());
        assert!(validate_family_name(&"a".repeat(100)).is_ok());
    }

    #[test]
    fn test_validate_family_name_empty() {
        let result = validate_family_name("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "name");
        assert!(err.message.contains("empty"));
    }

    #[test]
    fn test_validate_family_name_whitespace_only() {
        let result = validate_family_name("   ");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "name");
        assert!(err.message.contains("empty"));
    }

    #[test]
    fn test_validate_family_name_too_long() {
        let result = validate_family_name(&"a".repeat(101));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "name");
        assert!(err.message.contains("too long"));
    }

    #[test]
    fn test_validate_dependent_name_valid() {
        assert!(validate_dependent_name("Alice").is_ok());
        assert!(validate_dependent_name("B").is_ok());
        assert!(validate_dependent_name(&"a".repeat(100)).is_ok());
    }

    #[test]
    fn test_validate_dependent_name_empty() {
        let result = validate_dependent_name("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "name");
        assert!(err.message.contains("empty"));
    }

    #[test]
    fn test_validate_dependent_name_too_long() {
        let result = validate_dependent_name(&"a".repeat(101));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "name");
        assert!(err.message.contains("too long"));
    }

    #[test]
    fn test_validate_date_of_birth_valid() {
        let past_date = Date::from_naive_date(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert!(validate_date_of_birth(&past_date).is_ok());

        let today = Date::today();
        assert!(validate_date_of_birth(&today).is_ok());
    }

    #[test]
    fn test_validate_date_of_birth_future() {
        let future_date = Date::from_naive_date((Utc::now() + Duration::days(1)).date_naive());
        let result = validate_date_of_birth(&future_date);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "date_of_birth");
        assert!(err.message.contains("future"));
    }

    #[test]
    fn test_validate_activity_timestamp_valid() {
        let past_timestamp = Timestamp::from_datetime(Utc::now() - Duration::hours(1));
        assert!(validate_activity_timestamp(&past_timestamp).is_ok());
    }

    #[test]
    fn test_validate_activity_timestamp_future() {
        let future_timestamp = Timestamp::from_datetime(Utc::now() + Duration::hours(1));
        let result = validate_activity_timestamp(&future_timestamp);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "timestamp");
        assert!(err.message.contains("future"));
    }

    #[test]
    fn test_validate_feeding_activity_valid() {
        let activity = ActivityType::Feeding {
            feeding_type: FeedingType::Bottle,
            volume_ml: Some(120),
        };
        assert!(validate_activity_type(&activity).is_ok());

        let activity_no_volume = ActivityType::Feeding {
            feeding_type: FeedingType::Breast,
            volume_ml: None,
        };
        assert!(validate_activity_type(&activity_no_volume).is_ok());
    }

    #[test]
    fn test_validate_feeding_activity_zero_volume() {
        let activity = ActivityType::Feeding {
            feeding_type: FeedingType::Bottle,
            volume_ml: Some(0),
        };
        let result = validate_activity_type(&activity);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "volume_ml");
        assert!(err.message.contains("greater than zero"));
    }

    #[test]
    fn test_validate_diaper_change_activity_valid() {
        let activity = ActivityType::DiaperChange {
            contents: DiaperContents::Wet,
        };
        assert!(validate_activity_type(&activity).is_ok());
    }

    #[test]
    fn test_validate_sleep_activity_valid() {
        let start_time = Timestamp::from_datetime(Utc::now() - Duration::hours(2));
        let end_time = Some(Timestamp::from_datetime(Utc::now() - Duration::hours(1)));
        let activity = ActivityType::Sleep {
            start_time,
            end_time,
        };
        assert!(validate_activity_type(&activity).is_ok());
    }

    #[test]
    fn test_validate_sleep_activity_end_before_start() {
        let start_time = Timestamp::from_datetime(Utc::now() - Duration::hours(1));
        let end_time = Some(Timestamp::from_datetime(Utc::now() - Duration::hours(2)));
        let activity = ActivityType::Sleep {
            start_time,
            end_time,
        };
        let result = validate_activity_type(&activity);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "end_time");
        assert!(err.message.contains("after start"));
    }

    #[test]
    fn test_validate_sleep_activity_end_equals_start() {
        let timestamp = Timestamp::from_datetime(Utc::now() - Duration::hours(1));
        let activity = ActivityType::Sleep {
            start_time: timestamp,
            end_time: Some(timestamp),
        };
        let result = validate_activity_type(&activity);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_pumping_activity_valid() {
        let activity = ActivityType::Pumping { volume_ml: 150 };
        assert!(validate_activity_type(&activity).is_ok());
    }

    #[test]
    fn test_validate_pumping_activity_zero_volume() {
        let activity = ActivityType::Pumping { volume_ml: 0 };
        let result = validate_activity_type(&activity);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.field, "volume_ml");
        assert!(err.message.contains("greater than zero"));
    }

    #[test]
    fn test_sanitize_string_removes_null_bytes() {
        let input = "Hello\0World";
        let result = sanitize_string(input);
        assert_eq!(result, "HelloWorld");
    }

    #[test]
    fn test_sanitize_string_removes_control_characters() {
        let input = "Hello\x01\x02World";
        let result = sanitize_string(input);
        assert_eq!(result, "HelloWorld");
    }

    #[test]
    fn test_sanitize_string_preserves_newlines_and_tabs() {
        let input = "Hello\nWorld\tTest";
        let result = sanitize_string(input);
        assert_eq!(result, "Hello\nWorld\tTest");
    }

    #[test]
    fn test_sanitize_string_trims_whitespace() {
        let input = "  Hello World  ";
        let result = sanitize_string(input);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_sanitize_string_normal_text() {
        let input = "Normal text with spaces";
        let result = sanitize_string(input);
        assert_eq!(result, "Normal text with spaces");
    }
}

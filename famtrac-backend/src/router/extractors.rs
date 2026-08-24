use crate::errors::{HandlerError, ValidationError};

/// Extract a path parameter from a URL path given a prefix.
///
/// This function strips the given prefix from the path and returns the next path segment.
/// For example, given path `/families/123-456/dependents` and prefix `/families/`,
/// it returns `Some("123-456")`.
///
/// # Arguments
///
/// * `path` - The full URL path
/// * `prefix` - The prefix to strip from the path
///
/// # Returns
///
/// * `Some(String)` - The extracted path segment if found
/// * `None` - If the path doesn't start with the prefix or no segment follows
///
/// # Examples
///
/// ```
/// use famtrac_backend::router::extractors::extract_path_param;
///
/// let result = extract_path_param("/families/abc-123", "/families/");
/// assert_eq!(result, Some("abc-123".to_string()));
///
/// let result = extract_path_param("/families/abc-123/dependents", "/families/");
/// assert_eq!(result, Some("abc-123".to_string()));
///
/// let result = extract_path_param("/families", "/families/");
/// assert_eq!(result, None);
/// ```
pub fn extract_path_param(path: &str, prefix: &str) -> Option<String> {
    path.strip_prefix(prefix)
        .and_then(|s| s.split('/').next())
        .map(|s| s.to_string())
}

/// Extract and parse a UUID path parameter from a URL path.
///
/// This function uses `extract_path_param()` to extract the path segment, then attempts
/// to parse it as a UUID. If parsing fails, it returns a `HandlerError::Validation`
/// with the specified field name.
///
/// # Arguments
///
/// * `path` - The full URL path
/// * `prefix` - The prefix to strip from the path
/// * `field_name` - The name of the field for error reporting (e.g., "family_id", "dependent_id")
///
/// # Returns
///
/// * `Ok(uuid::Uuid)` - The parsed UUID if successful
/// * `Err(HandlerError::Validation)` - If the path segment is missing or not a valid UUID
///
/// # Error Handling
///
/// When UUID parsing fails, this function returns a `HandlerError::Validation` with:
/// - `field`: The provided field_name
/// - `message`: "Invalid {field_name} format"
/// - `constraint`: Some("must be a valid UUID")
///
/// This ensures consistent error messages across all route handlers.
///
/// # Examples
///
/// ```
/// use famtrac_backend::router::extractors::extract_uuid_param;
/// use uuid::Uuid;
///
/// let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
/// let path = format!("/families/{}", uuid_str);
/// let result = extract_uuid_param(&path, "/families/", "family_id");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), Uuid::parse_str(uuid_str).unwrap());
///
/// let result = extract_uuid_param("/families/not-a-uuid", "/families/", "family_id");
/// assert!(result.is_err());
/// ```
pub fn extract_uuid_param(
    path: &str,
    prefix: &str,
    field_name: &str,
) -> Result<uuid::Uuid, HandlerError> {
    let param = extract_path_param(path, prefix).ok_or_else(|| {
        HandlerError::Validation(ValidationError {
            field: field_name.to_string(),
            message: format!("Invalid {} format", field_name),
            constraint: Some("must be a valid UUID".to_string()),
        })
    })?;

    param.parse::<uuid::Uuid>().map_err(|_| {
        HandlerError::Validation(ValidationError {
            field: field_name.to_string(),
            message: format!("Invalid {} format", field_name),
            constraint: Some("must be a valid UUID".to_string()),
        })
    })
}

/// Parse a string as a UUID, returning a validation error on failure.
pub fn parse_uuid(value: &str, field_name: &str) -> Result<uuid::Uuid, HandlerError> {
    value.parse::<uuid::Uuid>().map_err(|_| {
        HandlerError::Validation(ValidationError {
            field: field_name.to_string(),
            message: format!("Invalid {} format", field_name),
            constraint: Some("must be a valid UUID".to_string()),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_param_basic() {
        let result = extract_path_param("/families/abc-123", "/families/");
        assert_eq!(result, Some("abc-123".to_string()));
    }

    #[test]
    fn test_extract_path_param_with_trailing_segment() {
        let result = extract_path_param("/families/abc-123/dependents", "/families/");
        assert_eq!(result, Some("abc-123".to_string()));
    }

    #[test]
    fn test_extract_path_param_no_segment() {
        let result = extract_path_param("/families", "/families/");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_path_param_empty_segment() {
        let result = extract_path_param("/families/", "/families/");
        assert_eq!(result, Some("".to_string()));
    }

    #[test]
    fn test_extract_path_param_wrong_prefix() {
        let result = extract_path_param("/dependents/abc-123", "/families/");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_path_param_uuid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/families/{}", uuid_str);
        let result = extract_path_param(&path, "/families/");
        assert_eq!(result, Some(uuid_str.to_string()));
    }

    #[test]
    fn test_extract_uuid_param_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/families/{}", uuid_str);
        let result = extract_uuid_param(&path, "/families/", "family_id");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uuid::Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_extract_uuid_param_valid_with_trailing_segment() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let path = format!("/families/{}/dependents", uuid_str);
        let result = extract_uuid_param(&path, "/families/", "family_id");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uuid::Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_extract_uuid_param_invalid_uuid() {
        let result = extract_uuid_param("/families/not-a-uuid", "/families/", "family_id");

        assert!(result.is_err());
        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "family_id");
                assert_eq!(err.message, "Invalid family_id format");
                assert_eq!(err.constraint, Some("must be a valid UUID".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_extract_uuid_param_missing_segment() {
        let result = extract_uuid_param("/families/", "/families/", "family_id");

        assert!(result.is_err());
        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "family_id");
                assert_eq!(err.message, "Invalid family_id format");
                assert_eq!(err.constraint, Some("must be a valid UUID".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_extract_uuid_param_different_field_names() {
        let result = extract_uuid_param("/dependents/not-a-uuid", "/dependents/", "dependent_id");
        assert!(result.is_err());
        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "dependent_id");
                assert_eq!(err.message, "Invalid dependent_id format");
            }
            _ => panic!("Expected ValidationError"),
        }

        let result = extract_uuid_param("/activities/not-a-uuid", "/activities/", "activity_id");
        assert!(result.is_err());
        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "activity_id");
                assert_eq!(err.message, "Invalid activity_id format");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_extract_uuid_param_malformed_uuid() {
        let test_cases = vec![
            "/families/123",
            "/families/abc-def-ghi",
            "/families/550e8400-e29b-41d4-a716",
            "/families/550e8400-e29b-41d4-a716-446655440000-extra",
        ];

        for path in test_cases {
            let result = extract_uuid_param(path, "/families/", "family_id");
            assert!(result.is_err(), "Expected error for path: {}", path);
        }
    }

    #[test]
    fn test_parse_uuid_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid(uuid_str, "family_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uuid::Uuid::parse_str(uuid_str).unwrap());
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let result = parse_uuid("not-a-uuid", "family_id");
        assert!(result.is_err());
        match result {
            Err(HandlerError::Validation(err)) => {
                assert_eq!(err.field, "family_id");
                assert_eq!(err.message, "Invalid family_id format");
                assert_eq!(err.constraint, Some("must be a valid UUID".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}

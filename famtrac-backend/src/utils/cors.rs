use std::collections::HashMap;

/// CORS configuration for the API
/// Requirements: 11.1, 11.2, 11.3, 11.4
#[derive(Clone)]
pub struct CorsConfig {
    pub allow_origin: String,
    pub allow_methods: String,
    pub allow_headers: String,
}

impl Default for CorsConfig {
    fn default() -> Self {
        CorsConfig {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            allow_headers: "Content-Type, Authorization".to_string(),
        }
    }
}

/// Add CORS headers to a response
/// Requirements: 11.2, 11.3, 11.4
pub fn add_cors_headers(headers: &mut HashMap<String, String>, config: &CorsConfig) {
    headers.insert(
        "Access-Control-Allow-Origin".to_string(),
        config.allow_origin.clone(),
    );
    headers.insert(
        "Access-Control-Allow-Methods".to_string(),
        config.allow_methods.clone(),
    );
    headers.insert(
        "Access-Control-Allow-Headers".to_string(),
        config.allow_headers.clone(),
    );
}

/// Handle OPTIONS preflight requests
/// Requirements: 11.1
pub fn handle_options(config: &CorsConfig) -> (u16, String, HashMap<String, String>) {
    let mut headers = HashMap::new();
    add_cors_headers(&mut headers, config);
    (204, String::new(), headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cors_config() {
        let config = CorsConfig::default();
        assert_eq!(config.allow_origin, "*");
        assert_eq!(config.allow_methods, "GET, POST, PUT, DELETE, OPTIONS");
        assert_eq!(config.allow_headers, "Content-Type, Authorization");
    }

    #[test]
    fn test_add_cors_headers() {
        let config = CorsConfig::default();
        let mut headers = HashMap::new();
        add_cors_headers(&mut headers, &config);

        assert_eq!(
            headers.get("Access-Control-Allow-Origin"),
            Some(&"*".to_string())
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Methods"),
            Some(&"GET, POST, PUT, DELETE, OPTIONS".to_string())
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Headers"),
            Some(&"Content-Type, Authorization".to_string())
        );
    }

    #[test]
    fn test_handle_options() {
        let config = CorsConfig::default();
        let (status, body, headers) = handle_options(&config);

        assert_eq!(status, 204);
        assert_eq!(body, "");
        assert!(headers.contains_key("Access-Control-Allow-Origin"));
        assert!(headers.contains_key("Access-Control-Allow-Methods"));
        assert!(headers.contains_key("Access-Control-Allow-Headers"));
    }

    #[test]
    fn test_custom_cors_config() {
        let config = CorsConfig {
            allow_origin: "https://example.com".to_string(),
            allow_methods: "GET, POST".to_string(),
            allow_headers: "Content-Type".to_string(),
        };

        let mut headers = HashMap::new();
        add_cors_headers(&mut headers, &config);

        assert_eq!(
            headers.get("Access-Control-Allow-Origin"),
            Some(&"https://example.com".to_string())
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Methods"),
            Some(&"GET, POST".to_string())
        );
        assert_eq!(
            headers.get("Access-Control-Allow-Headers"),
            Some(&"Content-Type".to_string())
        );
    }
}

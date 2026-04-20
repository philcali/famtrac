use aws_sdk_dynamodb::{Client, types::AttributeValue};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

// ── Custom authorizer event (manually defined since aws_lambda_events 0.15
//     doesn't re-export it) ──────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CustomAuthorizerEvent {
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub multi_value_headers: Option<HashMap<String, Vec<String>>>,
    #[serde(default)]
    pub query_string_parameters: Option<HashMap<String, String>>,
    #[serde(default)]
    pub multi_value_query_string_parameters: Option<HashMap<String, Vec<String>>>,
    #[serde(default)]
    pub stage_variables: Option<HashMap<String, String>>,
    #[serde(default)]
    pub request_context: Option<AuthorizerRequestContext>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub is_base64_encoded: Option<bool>,
    pub http_method: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizerRequestContext {
    pub authorizer: Option<HashMap<String, String>>,
}

// ── Response types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PolicyDocument {
    pub version: String,
    pub statement: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Statement {
    pub action: String,
    pub effect: String,
    pub resource: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomAuthorizerResponse {
    pub principal_id: String,
    pub policy_document: PolicyDocument,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, String>>,
}

// ── Config ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuthorizerConfig {
    pub cognito_issuer: String,
    pub cognito_audience: String,
    pub api_tokens_table: String,
    jwks_cache: Arc<tokio::sync::RwLock<Option<JwksCache>>>,
}

#[derive(Debug, Clone)]
struct JwksCache {
    jwks: serde_json::Value,
    expires_at: chrono::DateTime<Utc>,
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn is_jwt(token: &str) -> bool {
    token.split('.').count() == 3
}

fn base64url_decode(input: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(input)
        .map_err(|e| format!("base64url decode: {e}"))
}

// ── JWT path ───────────────────────────────────────────────────────────────

async fn fetch_jwks(issuer: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}/.well-known/jwks.json", issuer.trim_end_matches('/'));
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("fetch JWKS: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("JWKS HTTP {}", resp.status()));
    }
    resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())
}

/// Convert a JWK EC P-256 key to PEM-encoded SubjectPublicKeyInfo.
fn jwk_to_pem(jwk: &serde_json::Value) -> Result<String, String> {
    let kty = jwk
        .get("kty")
        .and_then(|v| v.as_str())
        .ok_or("JWK missing 'kty'")?;
    if kty != "EC" {
        return Err("only EC keys supported".into());
    }

    let crv = jwk
        .get("crv")
        .and_then(|v| v.as_str())
        .ok_or("JWK missing 'crv'")?;
    if crv != "P-256" {
        return Err("only P-256 curve supported".into());
    }

    let x = jwk
        .get("x")
        .and_then(|v| v.as_str())
        .ok_or("JWK missing 'x'")?;
    let y = jwk
        .get("y")
        .and_then(|v| v.as_str())
        .ok_or("JWK missing 'y'")?;

    let x_bytes = base64url_decode(x).map_err(|e| format!("decode x: {e}"))?;
    let y_bytes = base64url_decode(y).map_err(|e| format!("decode y: {e}"))?;

    // Build uncompressed point: 0x04 || x || y
    let point_bytes: Vec<u8> = std::iter::once(0x04)
        .chain(x_bytes.iter().copied())
        .chain(y_bytes.iter().copied())
        .collect();

    // Parse as EC verifying key
    let public_key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&point_bytes)
        .map_err(|e| format!("parse EC point: {e}"))?;

    let der_bytes = public_key.to_sec1_bytes().to_vec();

    // Build SubjectPublicKeyInfo DER manually for EC P-256
    // AlgorithmIdentifier: { OID ecPublicKey(1.2.840.10045.2.1), OID prime256v1(1.2.840.10045.3.1.7) }
    let algo_id: &[u8] = &[
        0x30, 0x10, // SEQUENCE
        0x06, 0x07, // OID ecPublicKey
        0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x02, 0x01,
        0x06, 0x08, // OID prime256v1
        0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x03, 0x01, 0x07,
    ];

    // BIT STRING wrapping the point
    let bit_string: Vec<u8> = {
        let mut bs = vec![0x03, (1 + der_bytes.len()) as u8, 0x00];
        bs.extend_from_slice(&der_bytes);
        bs
    };

    // SEQUENCE { algo_id, bit_string }
    let mut spki = vec![0x30, (algo_id.len() + bit_string.len()) as u8];
    spki.extend_from_slice(algo_id);
    spki.extend_from_slice(&bit_string);

    let pem = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        STANDARD.encode(&spki)
    );

    Ok(pem)
}

/// Verify a JWT and return (sub, username).
async fn verify_jwt(token: &str, config: &AuthorizerConfig) -> Result<(String, Option<String>), String> {
    // Decode header to get kid
    let header_b64 = token.split('.').nth(0).unwrap();
    let header_bytes = base64url_decode(header_b64)?;
    let header: serde_json::Value = serde_json::from_slice(&header_bytes)
        .map_err(|e| format!("parse JWT header: {e}"))?;
    let kid = header
        .get("kid")
        .and_then(|v| v.as_str())
        .ok_or("JWT missing 'kid'")?;

    // Get JWKS (with cache)
    let jwks = {
        let guard = config.jwks_cache.read().await;
        if let Some(cached) = guard.as_ref() {
            if cached.expires_at > Utc::now() {
                cached.jwks.clone()
            } else {
                drop(guard);
                fetch_jwks(&config.cognito_issuer).await?
            }
        } else {
            drop(guard);
            fetch_jwks(&config.cognito_issuer).await?
        }
    };

    // Find the matching key
    let keys = jwks
        .get("keys")
        .and_then(|v| v.as_array())
        .ok_or("JWKS missing 'keys'")?;
    let jwk = keys
        .iter()
        .find(|k| k.get("kid").and_then(|v| v.as_str()) == Some(kid))
        .ok_or(format!("no key for kid: {kid}"))?;

    // Convert JWK to PEM and then to DecodingKey
    let pem = jwk_to_pem(jwk)?;
    let decoding_key = jsonwebtoken::DecodingKey::from_ec_pem(pem.as_bytes())
        .map_err(|e| format!("parse EC PEM: {e}"))?;

    // Verify the token
    let token_data = jsonwebtoken::decode::<serde_json::Value>(
        token,
        &decoding_key,
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::ES256),
    )
    .map_err(|e| format!("JWT verify: {e}"))?;

    let claims: &serde_json::Value = &token_data.claims;
    let claims_obj = claims.as_object().ok_or("JWT claims not an object")?;
    let sub = claims_obj
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or("JWT missing 'sub'")?
        .to_string();

    let username = claims_obj
        .get("username")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok((sub, username))
}

// ── API token path ─────────────────────────────────────────────────────────

async fn lookup_api_token(
    client: &Client,
    table_name: &str,
    token: &str,
) -> Result<(String, Option<String>), String> {
    let result = client
        .get_item()
        .table_name(table_name)
        .key("PK", AttributeValue::S(format!("API_TOKEN#{}", token)))
        .key("SK", AttributeValue::S("#token".to_string()))
        .send()
        .await
        .map_err(|e| format!("lookup API token: {e}"))?;

    let item = result.item.ok_or("API token not found")?;

    let status = item
        .get("status")
        .and_then(|v| v.as_s().ok())
        .ok_or("API token missing status")?;

    if status != "active" {
        return Err(format!("API token is not active: {status}"));
    }

    let user_id: String = item
        .get("user_id")
        .and_then(|v| v.as_s().ok())
        .ok_or("API token missing user_id")?
        .clone();

    let username: Option<String> = item
        .get("username")
        .and_then(|v| v.as_s().ok())
        .map(|s| s.to_string());

    Ok((user_id.clone(), username))
}

// ── Authorizer ─────────────────────────────────────────────────────────────

fn create_response(
    principal_id: &str,
    identity_source: Option<&str>,
) -> CustomAuthorizerResponse {
    let mut context = HashMap::new();
    if let Some(source) = identity_source {
        context.insert("identity_source".to_string(), source.to_string());
    }

    CustomAuthorizerResponse {
        principal_id: principal_id.to_string(),
        policy_document: PolicyDocument {
            version: "2012-10-17".to_string(),
            statement: vec![Statement {
                action: "execute-api:Invoke".to_string(),
                effect: "Allow".to_string(),
                resource: vec!["*".to_string()],
            }],
        },
        context: Some(context),
    }
}

fn extract_bearer_token(event: &CustomAuthorizerEvent) -> Option<String> {
    event
        .headers
        .get("authorization")
        .or_else(|| event.headers.get("Authorization"))
        .and_then(|v| {
            let s = v.as_str();
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
}

async fn handle_authorizer(
    event: CustomAuthorizerEvent,
    client: &Client,
    config: &AuthorizerConfig,
) -> Result<CustomAuthorizerResponse, String> {
    let token = extract_bearer_token(&event).ok_or_else(|| {
        "Missing or invalid Authorization header. Expected 'Bearer <token>'".to_string()
    })?;

    let result = if is_jwt(&token) {
        verify_jwt(&token, config).await?
    } else {
        lookup_api_token(client, &config.api_tokens_table, &token).await?
    };

    let mut context = HashMap::new();
    context.insert("identity_id".to_string(), result.0.clone());
    if let Some(ref username) = result.1 {
        context.insert("identity_source".to_string(), username.clone());
    }

    Ok(CustomAuthorizerResponse {
        principal_id: result.0,
        policy_document: PolicyDocument {
            version: "2012-10-17".to_string(),
            statement: vec![Statement {
                action: "execute-api:Invoke".to_string(),
                effect: "Allow".to_string(),
                resource: vec!["*".to_string()],
            }],
        },
        context: Some(context),
    })
}

// ── Entry point ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let cognito_issuer =
        env::var("COGNITO_ISSUER").expect("COGNITO_ISSUER environment variable is required");
    let cognito_audience =
        env::var("COGNITO_AUDIENCE").expect("COGNITO_AUDIENCE environment variable is required");
    let api_tokens_table =
        env::var("API_TOKENS_TABLE").expect("API_TOKENS_TABLE environment variable is required");

    let config = AuthorizerConfig {
        cognito_issuer,
        cognito_audience,
        api_tokens_table,
        jwks_cache: Arc::new(tokio::sync::RwLock::new(None)),
    };

    let dynamodb_config = aws_config::load_from_env().await;
    let client = Client::new(&dynamodb_config);

    lambda_runtime::run(lambda_runtime::handler_fn(|event, _context| {
        let client = client.clone();
        let config = config.clone();
        async move { handle_authorizer(event, &client, &config).await }
    }))
    .await?;

    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_jwt_valid() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        assert!(is_jwt(jwt));
    }

    #[test]
    fn test_is_jwt_invalid() {
        assert!(!is_jwt("not-a-jwt"));
        assert!(!is_jwt("Bearer eyJhbGciOiJIUzI1NiJ9"));
        assert!(!is_jwt(""));
    }

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer test-token".to_string());
        let event = CustomAuthorizerEvent {
            headers,
            multi_value_headers: None,
            query_string_parameters: None,
            multi_value_query_string_parameters: None,
            stage_variables: None,
            request_context: None,
            body: None,
            is_base64_encoded: None,
            http_method: "GET".to_string(),
            path: "/test".to_string(),
        };
        assert_eq!(extract_bearer_token(&event), Some("test-token".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_case_insensitive() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer case-token".to_string());
        let event = CustomAuthorizerEvent {
            headers,
            multi_value_headers: None,
            query_string_parameters: None,
            multi_value_query_string_parameters: None,
            stage_variables: None,
            request_context: None,
            body: None,
            is_base64_encoded: None,
            http_method: "GET".to_string(),
            path: "/test".to_string(),
        };
        assert_eq!(extract_bearer_token(&event), Some("case-token".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let event = CustomAuthorizerEvent {
            headers: HashMap::new(),
            multi_value_headers: None,
            query_string_parameters: None,
            multi_value_query_string_parameters: None,
            stage_variables: None,
            request_context: None,
            body: None,
            is_base64_encoded: None,
            http_method: "GET".to_string(),
            path: "/test".to_string(),
        };
        assert!(extract_bearer_token(&event).is_none());
    }
}

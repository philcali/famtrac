use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};

use famtrac_backend::context::RequestContext;
use famtrac_backend::domain::{ActivityId, ActivityType, Date, DependentId, FamilyId};
use famtrac_backend::errors::{AuthError, HandlerError};
use famtrac_backend::handlers;
use famtrac_backend::repository::{
    DynamoDbActivityRepository, DynamoDbDependentRepository, DynamoDbFamilyRepository,
};
use famtrac_backend::utils::cors::CorsConfig;
use famtrac_backend::utils::response::HttpResponse;

/// Main Lambda handler function
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize AWS SDK config
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .load()
        .await;
    let dynamodb_client = Client::new(&config);

    // Get table name from environment variable
    let table_name =
        std::env::var("DYNAMODB_TABLE_NAME").unwrap_or_else(|_| "famtrac-backend".to_string());

    // Initialize repositories
    let family_repo = DynamoDbFamilyRepository::new(dynamodb_client.clone(), table_name.clone());
    let dependent_repo =
        DynamoDbDependentRepository::new(dynamodb_client.clone(), table_name.clone());
    let activity_repo = DynamoDbActivityRepository::new(dynamodb_client, table_name);

    // Initialize CORS config
    let cors_config = CorsConfig::default();

    // Create handler closure with repository instances
    let handler = service_fn(move |event: LambdaEvent<ApiGatewayProxyRequest>| {
        let family_repo = family_repo.clone();
        let dependent_repo = dependent_repo.clone();
        let activity_repo = activity_repo.clone();
        let cors_config = cors_config.clone();

        async move {
            handle_request(
                event.payload,
                &family_repo,
                &dependent_repo,
                &activity_repo,
                &cors_config,
            )
            .await
        }
    });

    lambda_runtime::run(handler).await?;
    Ok(())
}

/// Route and handle incoming API Gateway requests
async fn handle_request(
    request: ApiGatewayProxyRequest,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
    cors_config: &CorsConfig,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Log request for debugging (Requirement 9.4)
    eprintln!(
        "Received request: {} {}",
        request.http_method,
        request.path.as_deref().unwrap_or("/")
    );

    // Handle OPTIONS requests for CORS preflight
    if request.http_method == "OPTIONS" {
        return Ok(create_options_response(cors_config));
    }

    // Extract request context and identity (Requirement 12.1)
    let context = match RequestContext::from_api_gateway_context(&request.request_context) {
        Ok(ctx) => ctx,
        Err(AuthError::MissingIdentity) => {
            let response = HttpResponse::from_error(
                HandlerError::Auth(AuthError::MissingIdentity),
                cors_config,
            );
            return Ok(to_api_gateway_response(response));
        }
        Err(e) => {
            let response = HttpResponse::from_error(HandlerError::Auth(e), cors_config);
            return Ok(to_api_gateway_response(response));
        }
    };

    // Route request to appropriate handler
    let http_response = route_request(
        &request,
        &context,
        family_repo,
        dependent_repo,
        activity_repo,
        cors_config,
    );

    Ok(to_api_gateway_response(http_response))
}

/// Route request to appropriate handler based on method and path
fn route_request(
    request: &ApiGatewayProxyRequest,
    context: &RequestContext,
    family_repo: &DynamoDbFamilyRepository,
    dependent_repo: &DynamoDbDependentRepository,
    activity_repo: &DynamoDbActivityRepository,
    cors_config: &CorsConfig,
) -> HttpResponse {
    let method = request.http_method.as_str();
    let path = request.path.as_deref().unwrap_or("/");
    let body = request.body.as_deref().unwrap_or("");

    // Log routing for debugging
    eprintln!("Routing: {} {}", method, path);

    let result = match (method, path) {
        // Family endpoints
        ("POST", "/families") => handlers::create_family(body, context, family_repo),
        ("GET", path) if path.starts_with("/families/") && !path.contains("/dependents") => {
            let family_id = extract_path_param(path, "/families/");
            match family_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => {
                    handlers::get_family(FamilyId(id), context, family_repo, dependent_repo)
                }
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "family_id".to_string(),
                        message: "Invalid family ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }
        ("PUT", path) if path.starts_with("/families/") && !path.contains("/dependents") => {
            let family_id = extract_path_param(path, "/families/");
            match family_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => handlers::update_family(
                    FamilyId(id),
                    body,
                    context,
                    family_repo,
                    dependent_repo,
                ),
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "family_id".to_string(),
                        message: "Invalid family ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }

        // Dependent endpoints
        ("POST", "/dependents") => {
            handlers::create_dependent(body, context, family_repo, dependent_repo)
        }
        ("GET", path) if path.starts_with("/dependents/") && !path.contains("/activities") => {
            let dependent_id = extract_path_param(path, "/dependents/");
            match dependent_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => {
                    handlers::get_dependent(DependentId(id), context, family_repo, dependent_repo)
                }
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "dependent_id".to_string(),
                        message: "Invalid dependent ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }
        ("PUT", path) if path.starts_with("/dependents/") && !path.contains("/activities") => {
            let dependent_id = extract_path_param(path, "/dependents/");
            match dependent_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => handlers::update_dependent(
                    DependentId(id),
                    body,
                    context,
                    family_repo,
                    dependent_repo,
                ),
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "dependent_id".to_string(),
                        message: "Invalid dependent ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }
        ("GET", path) if path.contains("/families/") && path.ends_with("/dependents") => {
            let family_id = extract_path_param(path, "/families/");
            match family_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => {
                    handlers::list_dependents(FamilyId(id), context, family_repo, dependent_repo)
                }
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "family_id".to_string(),
                        message: "Invalid family ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }

        // Activity endpoints
        ("POST", "/activities") => {
            handlers::create_activity(body, context, family_repo, dependent_repo, activity_repo)
        }
        ("GET", path) if path.starts_with("/activities/") => {
            let activity_id = extract_path_param(path, "/activities/");
            match activity_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => handlers::get_activity(
                    ActivityId(id),
                    context,
                    family_repo,
                    dependent_repo,
                    activity_repo,
                ),
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "activity_id".to_string(),
                        message: "Invalid activity ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }
        ("PUT", path) if path.starts_with("/activities/") => {
            let activity_id = extract_path_param(path, "/activities/");
            match activity_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => handlers::update_activity(
                    ActivityId(id),
                    body,
                    context,
                    family_repo,
                    dependent_repo,
                    activity_repo,
                ),
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "activity_id".to_string(),
                        message: "Invalid activity ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }
        ("DELETE", path) if path.starts_with("/activities/") => {
            let activity_id = extract_path_param(path, "/activities/");
            match activity_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => handlers::delete_activity(
                    ActivityId(id),
                    context,
                    family_repo,
                    dependent_repo,
                    activity_repo,
                ),
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "activity_id".to_string(),
                        message: "Invalid activity ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }
        ("GET", path) if path.contains("/dependents/") && path.ends_with("/activities") => {
            let dependent_id = extract_path_param(path, "/dependents/");
            match dependent_id.and_then(|id| id.parse::<uuid::Uuid>().ok()) {
                Some(id) => {
                    // Parse query parameters
                    let query_params = &request.query_string_parameters;
                    let start_date = query_params
                        .first("start_date")
                        .and_then(|s| s.parse::<chrono::NaiveDate>().ok())
                        .map(Date::from_naive_date);
                    let end_date = query_params
                        .first("end_date")
                        .and_then(|s| s.parse::<chrono::NaiveDate>().ok())
                        .map(Date::from_naive_date);
                    let activity_type = query_params.first("activity_type").and_then(|s| {
                        match s.to_string().as_str() {
                            "feeding" => Some(ActivityType::Feeding {
                                feeding_type: famtrac_backend::domain::FeedingType::Breast,
                                volume_ml: None,
                            }),
                            "diaper_change" => Some(ActivityType::DiaperChange {
                                contents: famtrac_backend::domain::DiaperContents::Wet,
                            }),
                            "sleep" => {
                                let now = famtrac_backend::domain::Timestamp::now();
                                Some(ActivityType::Sleep {
                                    start: now,
                                    end: now,
                                })
                            }
                            "pumping" => Some(ActivityType::Pumping { volume_ml: 0 }),
                            _ => None,
                        }
                    });

                    handlers::query_activities(
                        DependentId(id),
                        start_date,
                        end_date,
                        activity_type,
                        context,
                        family_repo,
                        dependent_repo,
                        activity_repo,
                    )
                }
                None => Err(HandlerError::Validation(
                    famtrac_backend::errors::ValidationError {
                        field: "dependent_id".to_string(),
                        message: "Invalid dependent ID format".to_string(),
                        constraint: Some("must be a valid UUID".to_string()),
                    },
                )),
            }
        }

        // Unknown route
        _ => Err(HandlerError::NotFound(format!(
            "Route not found: {} {}",
            method, path
        ))),
    };

    // Convert handler result to HTTP response with CORS headers
    HttpResponse::from_handler_result(result, cors_config)
}

/// Extract path parameter from URL path
fn extract_path_param(path: &str, prefix: &str) -> Option<String> {
    path.strip_prefix(prefix)
        .and_then(|s| s.split('/').next())
        .map(|s| s.to_string())
}

/// Create OPTIONS response for CORS preflight
fn create_options_response(cors_config: &CorsConfig) -> ApiGatewayProxyResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Access-Control-Allow-Origin",
        cors_config.allow_origin.parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        cors_config.allow_methods.parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        cors_config.allow_headers.parse().unwrap(),
    );

    ApiGatewayProxyResponse {
        status_code: 200,
        headers,
        multi_value_headers: HeaderMap::new(),
        body: None,
        is_base64_encoded: false,
    }
}

/// Convert HttpResponse to ApiGatewayProxyResponse
fn to_api_gateway_response(response: HttpResponse) -> ApiGatewayProxyResponse {
    let mut headers = HeaderMap::new();
    for (key, value) in response.headers {
        if let Ok(header_value) = value.parse() {
            // Use a static string for common headers or leak the string for custom headers
            match key.as_str() {
                "Access-Control-Allow-Origin" => {
                    headers.insert("Access-Control-Allow-Origin", header_value);
                }
                "Access-Control-Allow-Methods" => {
                    headers.insert("Access-Control-Allow-Methods", header_value);
                }
                "Access-Control-Allow-Headers" => {
                    headers.insert("Access-Control-Allow-Headers", header_value);
                }
                "Content-Type" => {
                    headers.insert("Content-Type", header_value);
                }
                _ => {
                    // For other headers, we need to use a static lifetime
                    // In a real implementation, you might want to handle this differently
                    eprintln!("Warning: Skipping custom header: {}", key);
                }
            }
        }
    }

    ApiGatewayProxyResponse {
        status_code: response.status as i64,
        headers,
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text(response.body)),
        is_base64_encoded: false,
    }
}

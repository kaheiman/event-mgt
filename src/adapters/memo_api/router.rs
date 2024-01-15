use axum::{
    async_trait,
    http::{StatusCode, HeaderMap, self},
    extract::{Path, State, Request, FromRequestParts},
    response::{IntoResponse, Response, Json},
    routing::{get, post},
    body::{Body, Bytes},
    middleware::{self, Next},
    Json as AxumJson,
    Router, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, to_value};
use std::sync::Arc;
use jsonwebtoken::{decode, DecodingKey, Validation, EncodingKey};
use http_body_util::BodyExt;

use crate::{services::notification::{NotificationService, NotificationServiceInterface}, adapters::memo_events::processors::model::NotificationStatus};
use once_cell::sync::Lazy;


static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    tracing::info!("JWT_SECRET: {:?}", secret);
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    uid: String,
    name: String,
    email: String,
    created_at: i64,
    expired_at: i64
}

#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub struct AppService {
    pub notification_service: NotificationService,
}

// fn validate_jwt(token: &str, secret: &str) -> Result<Claims, JwtError> {
//     // JWT validation logic
//     Ok(())
// }

// Start defining routes
pub fn construct(app_state: Arc<AppService>) -> Router {
    Router::new()
        .route("/notification/message/:user_id", get(get_notification))
        .route("/notification/message/:user_id/:noti_id", post(update_notification_status))
        .with_state(app_state)
        .layer(middleware::from_fn(print_request_response))
}

// GET endpoint logic
async fn get_notification(
    State(app_service): State<Arc<AppService>>,
    claims: Claims,
    Path(user_id): Path<String>,
    req: Request<Body>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("claims: {:?}", claims);
    let secret = "your_secret"; // Use your actual secret key
    let mut json_value: Value = json!({});
    let token = req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    println!("Token: {:?}", token);
    println!("User ID: {:?}", user_id);
    let result = app_service.notification_service.get_notification_by_user_id(user_id).await;
    match result {
        Ok(notification) => {
            println!("Notification: {:?}", notification);
            json_value = to_value(notification).expect("Failed to serialize notification");
            // Ok(AxumJson(serde_json::json!(json_value)))
        },
        Err(e) => {
            println!("get_notification Error: {:?}", e);
        }

    }

    Ok(AxumJson(serde_json::json!({"message": "succeeded", "data": json_value })))
}


// Struct to capture the POST request body
#[derive(Serialize, Deserialize, Debug)]
struct UpdateNotificationBody {
    action: NotificationStatus,
}

async fn update_notification_status(
    State(app_service): State<Arc<AppService>>,
    headers: HeaderMap,
    Path((user_id, noti_id)): Path<(String, String)>,
    Json(payload): Json<UpdateNotificationBody>,
) ->  Result<Json<Value>, StatusCode> {
    let secret = "your_secret"; // Use your actual secret key
    let mut json_value: Value = json!({});

    // if let Some(auth_header) = headers.get("AUTHORIZATION") {
    //     let header_value = auth_header.to_str().unwrap_or("");
    //     println!("Received custom header: {}", header_value);
    // }
    let token = headers.get("AUTHORIZATION")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    println!("usr_id: {:?}", user_id);
    println!("payload: {:?}", payload);
    println!("noti_id: {:?}", noti_id);

    let result = app_service.notification_service.update_notification_message(user_id, noti_id).await;
    if let Err(e) = result {
        println!("update_notification_status Error: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(AxumJson(serde_json::json!({"message": "succeeded", "data": "asd" })))
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (req_parts, body) = req.into_parts();

    let clone_req_parts = req_parts.clone();
    let req = Request::from_parts(req_parts, body);
    let res = next.run(req).await;

    let (res_parts, body) = res.into_parts();
    let clone_res_parts = res_parts.clone();
    let bytes = buffer_and_print(body, clone_req_parts, clone_res_parts).await?;
    let res = Response::from_parts(res_parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(body: B, req_parts: http::request::Parts, res_parts: http::response::Parts) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let method = req_parts.method.clone().to_string();
    let uri = req_parts.uri.clone().to_string();
    let headers = req_parts.headers.clone();

    let status = res_parts.status.clone().to_string();

    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read response body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::info!(method, uri, status, "response:{body:?} {headers:?}");
    }

    Ok(bytes)
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

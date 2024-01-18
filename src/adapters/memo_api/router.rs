use axum::{
    async_trait,
    http::{StatusCode, self},
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
use jsonwebtoken::{decode, DecodingKey, Validation};
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
    iat: i64,
    exp: i64,
}

#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    InvalidToken,
}

#[derive(Debug)]
enum ApiServerError {
    InternalServerError(String),
    UnexpectedError(String),
}

struct Keys {
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub struct AppService {
    pub notification_service: NotificationService,
}

// Start defining routes
pub fn construct(app_state: Arc<AppService>) -> Router {
    Router::new()
        .route("/n/notification/message/:user_id", get(get_notification))
        .route("/n/notification/message/:user_id/:noti_id", post(update_notification_status))
        .with_state(app_state)
        .layer(middleware::from_fn(print_request_response))
}

// GET endpoint logic
async fn get_notification(
    State(app_service): State<Arc<AppService>>,
    claims: Claims,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, ApiServerError> {
    tracing::info!("claims: {:?}", claims);
    if user_id != claims.uid {
        tracing::error!("Invalid user id provided {:?}", user_id);
        return Err(ApiServerError::UnexpectedError("Invalid user id provided".to_string()));
    }
    let result = app_service.notification_service.get_notification_by_user_id(user_id).await;
    match result {
        Ok(notification) => {
            tracing::info!("Notification: {:?}", notification);
            let resp_json = to_value(notification).expect("Failed to serialize notification");
            Ok(AxumJson(serde_json::json!({"message": "succeeded", "data": resp_json })))
        },
        Err(e) => {
            tracing::error!("get_notification Error: {:?}", e);
            return Err(ApiServerError::InternalServerError(format!("get_notification Error: {:?}", e)))
        }
    }
}


// Struct to capture the POST request body
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateNotificationBody {
    pub action: NotificationStatus,
}

async fn update_notification_status(
    State(app_service): State<Arc<AppService>>,
    claims: Claims,
    Path((user_id, noti_id)): Path<(String, String)>,
    Json(payload): Json<UpdateNotificationBody>,
) ->  Result<Json<Value>, ApiServerError> {
    if user_id != claims.uid {
        tracing::error!("Invalid user id provided {:?}", user_id);
        return Err(ApiServerError::UnexpectedError("Invalid user id provided".to_string()));
    }
    let result = app_service.notification_service.update_notification_message(user_id, noti_id, payload).await;
    if let Err(e) = result {
        tracing::error!("update_notification_status Error: {:?}", e);
        return Err(ApiServerError::InternalServerError(format!("update_notification_status Error: {:?}", e)))
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
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl IntoResponse for ApiServerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiServerError::InternalServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            ApiServerError::UnexpectedError(message) => (StatusCode::NOT_ACCEPTABLE, message),
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
            .map_err(|_| AuthError::WrongCredentials)?;
        let mut validation = Validation::default();
        // Todo: update to true
        validation.validate_exp = false;
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &validation);
        match token_data {
            Ok(token_data) => {
                Ok(token_data.claims)
            },
            Err(e) => {
                tracing::info!("token_data error: {:?}", e);
                Err(AuthError::InvalidToken)
            }
        }

    }
}

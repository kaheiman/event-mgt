use axum::{
    http::{StatusCode, Request, HeaderMap},
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    body,
    Json as AxumJson,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, to_value};
use std::sync::Arc;
use jsonwebtoken::{decode, DecodingKey, Validation, errors::Error as JwtError};

use crate::{services::notification::{NotificationService, NotificationServiceInterface}, adapters::memo_events::processors::model::NotificationStatus};

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    // Your claims fields
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
}

// GET endpoint logic
async fn get_notification(
    State(app_service): State<Arc<AppService>>,
    Path(user_id): Path<String>,
    req: Request<body::Body>,
) -> Result<Json<Value>, StatusCode> {
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

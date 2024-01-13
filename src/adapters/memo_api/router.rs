use axum::{
    Extension,
    http::{StatusCode, Request},
    extract::{Path, State},
    response::Json,
    routing::get,
    body,
    Json as AxumJson,
    Router,
};
use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use jsonwebtoken::{decode, DecodingKey, Validation, errors::Error as JwtError};

use crate::services::notification::{NotificationService, NotificationServiceInterface};


// Struct to capture the POST request body
#[derive(Serialize, Deserialize, Debug)]
struct PostBody {
    action: String,
    message_id: String,
}

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
        .with_state(app_state)
}

// GET endpoint logic
async fn get_notification(
    State(app_service): State<Arc<AppService>>,
    Path(user_id): Path<String>,
    req: Request<body::Body>,
) -> Result<Json<Value>, StatusCode> {
    let secret = "your_secret"; // Use your actual secret key
    let token = req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    println!("Token: {:?}", token);
    println!("User ID: {:?}", user_id);
    app_service.notification_service.get_notification_by_user_id(user_id);
    // app_state.db_client.put_item()
    //     .table_name("memo-management")
    //     .item(json!({
    //         "user_id": user_id,
    //         "token": token,
    //     }))
    //     .send()
    //     .await
    //     .map_err(|e| {
    //         println!("Error: {:?}", e);
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?;

    // Your logic here...

    Ok(AxumJson(serde_json::json!({"message": "done"})))
}

// POST endpoint logic
async fn post_notification(
    Path(user_id): Path<String>,
    Json(body): Json<PostBody>,
) -> (StatusCode, Json<serde_json::Value>) {
    println!("User ID: {:?}", user_id);
    println!("Body: {:?}", body);

    (StatusCode::OK, Json(json!({"message": "done"})))
}

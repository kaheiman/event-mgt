mod adapters;
mod client;
mod errors;
mod utils;
mod services;
use std::env;

use aws_sdk_dynamodb::Client as DynamoDbClient;
use serde::{Deserialize, Serialize};
use aws_sdk_sqs::Client as SQSClient;

use dotenv::dotenv;
use services::notification::NotificationService;
use services::store::DatabaseStoreService;
use crate::adapters::memo_events::sqs_poller::{SQSPoller, SQSPollerOption, SQSPollerInterface};
use crate::adapters::memo_api::router;
use crate::client::dynamodb_client;
use std::sync::Arc;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize, Deserialize)]
struct SQSEvent {
    detail: EventDetail,
}
#[derive(Debug, Serialize, Deserialize)]
struct EventDetail {
    version: String,
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    source: String,
    time: String,
    data: EventData,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventData {
    notification_id: String,
    #[serde(rename = "type")]
    event_type: String,
    user_id: String,
    replyer_id: String,
    topic_id: String,
    message_id: String,
    content: String,
    created_time: String,
}

fn build_notification_service(db_client: DynamoDbClient, table_name: String) -> NotificationService {
    let db_service = DatabaseStoreService {
        store: db_client,
        table_name: table_name,
    };
    let notification_service = NotificationService {
        database_store_service: db_service,
    };
    return notification_service;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "memo-events-mgt=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv().ok();
    let config = aws_config::from_env().load().await;
    let memo_module = match env::var("MEMO_MODULE") {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to get MEMO_MODULE from environment: {:?}", e);
            return;
        }
    };
    let memo_failure_queue = match env::var("MEMO_FAILURE_QUEUE") {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to get MEMO_FAILURE_QUEUE from environment: {:?}", e);
            return;
        }
    };
    let memo_sqs_event_queue = match env::var("MEMO_SQS_EVENT_QUEUE") {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to get MEMO_SQS_EVENT_QUEUE from environment: {:?}", e);
            return;
        }
    };
    let dynamo_db_table_name = match env::var("DYNAMODB_TABLE_NAME") {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to get DYNAMODB_TABLE_NAME from environment: {:?}", e);
            return;
        }
    };
    if memo_module.eq(&"READER".to_string()) {
        tracing::info!("Memo reader module is running");
        dynamodb_client::init(&config).await;
        let dynamodb_client = dynamodb_client::get().unwrap();
        let notification_service = build_notification_service(dynamodb_client, dynamo_db_table_name);
        let sqs_client = SQSClient::new(&config);
        let sqs_option =  SQSPollerOption {
            sqs_client: sqs_client,
            sqs_queue: memo_sqs_event_queue,
            failure_queue: memo_failure_queue,
            wait_time_seconds: Some(10),
            max_number_of_messages: Some(10), // max is 10
            max_retry: Some(5),
            notification_service: notification_service,
        };
        SQSPoller::new(sqs_option).await.start_processing().await;
        return;
    } else if memo_module.eq(&"SERVER".to_string()) {
        tracing::info!("Memo server module is running");
        dynamodb_client::init(&config).await;
        let dynamodb_client = dynamodb_client::get().unwrap();
        let app_service = Arc::new(router::AppService {
            notification_service: build_notification_service(dynamodb_client, dynamo_db_table_name),
        });

        let router = router::construct(app_service);
        // Run the server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .unwrap();
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, router).await.unwrap();
    } else {
        panic!("Invalid MEMO_MODULE value: {}", memo_module)
    }
}
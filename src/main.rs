mod adapters;
mod client;
mod errors;
mod utils;
mod services;
use std::env;

use aws_sdk_dynamodb::Client as DynamoDbClient;
use errors::main::SystemError;
use serde::{Deserialize, Serialize};
use aws_sdk_sqs::Client as SQSClient;
use aws_sdk_eventbridge::{Client as EventBridgeClient, Error as EventBridgeError, types::PutEventsRequestEntry};

use dotenv::dotenv;
use services::notification::NotificationService;
use services::store::DatabaseStoreService;
use crate::adapters::memo_events::sqs_poller::{SQSPoller, SQSPollerOption, SQSPollerInterface};
use crate::adapters::memo_api::router;
use crate::client::dynamodb_client;
use crate::errors::main::SerializationError;
use std::sync::Arc;

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

async fn send_notification_event_to_eventbridge(event: &EventData, client: &EventBridgeClient) -> Result<(), SystemError> {

    // Serialize EventData to JSON
    let event_json = serde_json::to_string(event).map_err(|e| SerializationError::new(&e.to_string()))?;

    print!("event response: {:?}", event_json);

    let event_entry = PutEventsRequestEntry::builder()
        .detail(event_json)
        .detail_type("Memo Event Type") // Replace with your detail type
        .source("rust-test") // Replace with your source
        .event_bus_name("eventbridge-memo-events-listener")
        .build();

    let response = client.put_events()
        .entries(event_entry)
        .send()
        .await
        .map_err(|e| EventBridgeError::from(e))?;

    println!("EventBridge response: {:?}", response);

    Ok(())
}

#[tokio::main]
async fn main() {
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
        println!("Memo reader module is running");
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
        println!("Memo server module is running");
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
        println!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, router).await.unwrap();
    } else {
        panic!("Invalid MEMO_MODULE value: {}", memo_module)
    }

    let event_json = r#"{
    "detail": {
        "version": "1.0",
        "id": "event-id",
        "type": "memo-events-1.0.0",
        "source": "memo:monkier",
        "time": "2021-03-10T06:45:01:777614Z",
        "data": {
            "notification_id": "unique-notification-id-1234",
            "type": "memo:message.created-1.0.0",
            "replyer_id": "replyer-id",
            "user_id": "user-id",
            "topic_id": "topic_id",
            "message_id": "message_id",
            "content": "content",
            "created_time": "2023-01-01T12:00:00Z"
            }
        }
    }"#;

    let event_bridge_client = EventBridgeClient::new(&config);

    let event_bridge_data = EventData {
        event_type: "memo:message.created-1.0.0".to_string(),
        notification_id: "notif123".to_string(),
        user_id: "user456".to_string(),
        replyer_id: "replyer789".to_string(),
        topic_id: "topic345".to_string(),
        message_id: "message678".to_string(),
        content: "Example content".to_string(),
        created_time: "2023-01-01T00:00:00Z".to_string(),
    };

    if let Err(e) = send_notification_event_to_eventbridge(&event_bridge_data, &event_bridge_client).await {
        eprintln!("Failed to send event to eventBridge: {:?}", e);
    }
}
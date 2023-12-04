use std::{collections::HashMap, error::Error};

use serde::{Deserialize, Serialize};
use serde_json::{Value, to_string};
use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError, types::AttributeValue};

use aws_sdk_eventbridge::{Client as EventBridgeClient, Error as EventBridgeError, types::PutEventsRequestEntry};

use dotenv::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone)]
enum NotificationType {
  Message,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum NotificationStatus {
  Read,
  Unread,
  Removed,
}

#[derive(Debug, Serialize, Deserialize)]
struct SQSEvent {
  detail: EventDetail,
}

#[derive(Debug, Serialize, Deserialize)]
struct NotificationEvent {
  version: String,
  source: String,
  data: EventData,
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
  user_id: String,
  replyer_id: String,
  memo_id: String,
  topic_id: String,
  message_id: String,
  content: String,
  created_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DBNotifcation {
  #[serde(rename = "PK")]
  notification_id: String,
  #[serde(rename = "SK")]
  user_id: String,
  replyer_id: String,
  notification_type: NotificationType,
  status: NotificationStatus,
  topic_id: String,
  message_id: String,
  content: String,
  created_time: String,
  updated_time: Option<String>,
}

fn struct_to_hashmap<T: Serialize>(t: &T) -> Result<HashMap<String, AttributeValue>, Box<dyn Error>> {
    // Convert the struct to a JSON string
    let json = to_string(t)?;
    let map: HashMap<String, Value> = serde_json::from_str(&json)?;

    let hashmap = map.into_iter().map(|(k, v)| {
        let attr_value = match v {
            Value::String(s) => AttributeValue::S(s),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    AttributeValue::N(i.to_string())
                } else if let Some(f) = n.as_f64() {
                    AttributeValue::N(f.to_string())
                } else {
                    AttributeValue::Null(true)
                }
            },
            Value::Bool(b) => AttributeValue::Bool(b),
            Value::Null => AttributeValue::Null(true),
            // Add other conversions as needed (e.g., lists, maps)
            _ => AttributeValue::Null(true), // Fallback for unsupported types
        };
        (k, attr_value)
    }).collect();

    Ok(hashmap)
}

async fn store_event_in_dynamodb(event: &SQSEvent, client: &DynamoDbClient) -> Result<(), DynamoDbError> {
    let item = DBNotifcation {
      notification_id: event.detail.data.notification_id.clone(),
      user_id: event.detail.data.user_id.clone(),
      replyer_id: event.detail.data.replyer_id.clone(),
      notification_type: NotificationType::Message,
      status: NotificationStatus::Unread,
      topic_id: event.detail.data.topic_id.clone(),
      message_id: event.detail.data.message_id.clone(),
      content: event.detail.data.content.clone(),
      created_time: event.detail.data.created_time.clone(),
      updated_time: None,
    };

    // Call to struct_to_hashmap and handle the result
    let item_hashmap = match struct_to_hashmap(&item) {
        Ok(hashmap) => {
            // If successful, return the hashmap to use it in the next steps
            hashmap
        },
        Err(e) => {
            // If an error occurs, handle it here and possibly exit
            eprintln!("Failed to convert struct to hashmap: {}", e);
            return // or use std::process::exit(1) to exit with an error code
        }
    };

    let condition_expression = "attribute_not_exists(PK) AND attribute_not_exists(SK)";

    client.put_item()
        .table_name("memo-management")
        .set_item(Some(item_hashmap))
        .condition_expression(condition_expression)
        .send()
        .await?;

    Ok(())
}

async fn send_notification_event_to_eventbridge(event: &NotificationEvent, client: &EventBridgeClient) -> Result<(), EventBridgeError> {

    // Serialize EventData to JSON
    let event_json = serde_json::to_string(event);

    let event_entry = PutEventsRequestEntry::builder()
        .detail(event_json.unwrap())
        .detail_type("memo:message.created-1.0.0") // Replace with your detail type
        .source("rust-test") // Replace with your source
        .event_bus_name("eventbridge-memo-events-listener")
        .build();

    let response = client.put_events()
        .entries(event_entry)
        .send()
        .await?;

    println!("Response: {:?}", response);
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let event_json = r#"{
      "detail": {
        "version": "1.0",
        "id": "event-id",
        "type": "memo-events-1.0.0",
        "source": "memo:monkier",
        "time": "2021-03-10T06:45:01:777614Z",
        "data": {
          "notification_id": "unique-notification-id",
          "replyer_id": "replyer-id",
          "user_id": "user-id",
          "memo_id": "456",
          "topic_id": "topic_id",
          "message_id": "message_id",
          "content": "content",
          "created_time": "2023-01-01T12:00:00Z"
        }
      }
    }"#;

    let event: SQSEvent = serde_json::from_str(event_json).expect("Failed to parse event JSON");

    let config = aws_config::from_env().load().await;

    let db_client = DynamoDbClient::new(&config);

    let event_bridge_client = EventBridgeClient::new(&config);

    let event_bridge_data = NotificationEvent {
        version: "1.0".to_string(),
        source: "rust-test".to_string(),
        data: EventData {
          notification_id: "notif123".to_string(),
          user_id: "user456".to_string(),
          replyer_id: "replyer789".to_string(),
          memo_id: "memo012".to_string(),
          topic_id: "topic345".to_string(),
          message_id: "message678".to_string(),
          content: "Example content".to_string(),
          created_time: "2023-01-01T00:00:00Z".to_string(),
        }
      };

    if let Err(e) = send_notification_event_to_eventbridge(&event_bridge_data, &event_bridge_client).await {
        eprintln!("Failed to send event to eventBridge: {:?}", e);
    }

    if let Err(e) = store_event_in_dynamodb(&event, &db_client).await {
        eprintln!("Failed to store event in DynamoDB: {:?}", e);
    }
}
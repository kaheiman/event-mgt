use async_trait::async_trait;
use crate::{
  utils::utils::struct_to_hashmap,
  adapters::memo_events::processors::{model::{CreateMessageBody, DBNotifcation, NotificationType, NotificationStatus},
  event_type_processor::{PermanentError, ApplicationError, RetryableError}}
};
use serde_dynamo::from_items;

use super::store::{DatabaseStoreService, DatabaseStoreInterface};

// Define the trait for database operations
#[async_trait]
pub trait NotificationServiceInterface {
  async fn create_notification_message(&self, body: CreateMessageBody) -> Result<(), ApplicationError>;
  async fn get_notification_by_user_id(&self, user_id: String) -> Result<Vec<DBNotifcation>, ApplicationError>;
  async fn update_notification_message(&self, user_id: String, noti_id: String) -> Result<(), ApplicationError>;
}

// Define the struct implementing the trait
#[derive(Debug)]
pub struct NotificationService {
  pub database_store_service: DatabaseStoreService,
}

#[async_trait]
impl NotificationServiceInterface for NotificationService {
  async fn update_notification_message(&self, user_id: String, noti_id: String) -> Result<(), ApplicationError> {
    let noti_output  = self.database_store_service
      .db_get_notification_item_with_pk_sk(
        format!("USR#{}", user_id),
        format!("NTF#{}", noti_id)
      )
      .await.map_err(|e| RetryableError::new(&e.to_string()))?;

    if let Some(_) = noti_output.item {
        self.database_store_service
          .db_update_notification_message(
            user_id,
            noti_id,
            NotificationStatus::READ
          )
          .await.map_err(|e| RetryableError::new(&e.to_string()))?;
    } else {
        println!("update notification item not found");
    }

    Ok(())
  }

  async fn get_notification_by_user_id(&self, user_id: String) -> Result<Vec<DBNotifcation>, ApplicationError> {
    // Implementation for getting a notification message from DynamoDB
    let mut noti_items: Vec<DBNotifcation> = vec![];
    let result = self.database_store_service
      .db_get_notifications_by_user_id(user_id)
      .await.map_err(|e| RetryableError::new(&e.to_string()))?;

    if let Some(items) = result.items {
      noti_items = from_items(items).map_err(|e| PermanentError::new(&e.to_string()))?;
      println!("Got {} notification", noti_items.len());
    }

    Ok(noti_items)
  }

  async fn create_notification_message(&self, body: CreateMessageBody) -> Result<(), ApplicationError> {
    let now = chrono::Utc::now().to_rfc3339();
    let d = DBNotifcation{
      notification_id: format!("NTF#{}",body.detail.notification_id.clone()),
      user_id: format!("USR#{}", body.detail.user_id.clone()),
      replyer_id: body.detail.replyer_id.clone(),
      notification_type: NotificationType::Message,
      status: NotificationStatus::UNREAD,
      topic_id: body.detail.topic_id.clone(),
      message_id: body.detail.message_id.clone(),
      content: body.detail.content.clone(),
      created_time: now,
      updated_time: None,
    };

    let condition_expression = "attribute_not_exists(PK) AND attribute_not_exists(SK)";

    let item = struct_to_hashmap(&d)
            .map_err(|_| PermanentError::new("Failed to convert struct to hashmap"))?;

    self.database_store_service
      .db_create_notification_message(item, condition_expression.to_string())
      .await.map_err(|e| RetryableError::new(&e.to_string()))?;

    Ok(())
  }
}

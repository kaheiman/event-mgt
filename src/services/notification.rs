use async_trait::async_trait;
use crate::{
  utils::utils::struct_to_hashmap,
  adapters::memo_events::processors::{model::{CreateMessageBody, DBNotifcation, NotificationType, NotificationStatus},
  event_type_processor::{PermanentError, ApplicationError, RetryableError}}
};

use super::store::{DatabaseStoreService, DatabaseStoreInterface};

// Define the trait for database operations
#[async_trait]
pub trait NotificationServiceInterface {
  async fn create_notification_message(&self, body: CreateMessageBody) -> Result<(), ApplicationError>;
  async fn get_notification_by_user_id(&self, user_id: String) -> Result<(), ApplicationError>;
}

// Define the struct implementing the trait
#[derive(Debug)]
pub struct NotificationService {
  pub database_store_service: DatabaseStoreService,
}

#[async_trait]
impl NotificationServiceInterface for NotificationService {
  async fn get_notification_by_user_id(&self, user_id: String) -> Result<(), ApplicationError> {
    // Implementation for getting a notification message from DynamoDB
    self.database_store_service.get_notification_message(user_id).await.map_err(|e| RetryableError::new(&e.to_string()))?;
    Ok(())
  }

  async fn create_notification_message(&self, body: CreateMessageBody) -> Result<(), ApplicationError> {
    let d = DBNotifcation{
      notification_id: body.detail.notification_id.clone(),
      user_id: body.detail.user_id.clone(),
      replyer_id: body.detail.replyer_id.clone(),
      notification_type: NotificationType::Message,
      status: NotificationStatus::Unread,
      topic_id: body.detail.topic_id.clone(),
      message_id: body.detail.message_id.clone(),
      content: body.detail.content.clone(),
      created_time: body.detail.created_time.clone(),
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

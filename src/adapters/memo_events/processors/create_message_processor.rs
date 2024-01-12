use async_trait::async_trait;
use crate::adapters::memo_events::processors::event_type_processor::{PermanentError, RetryableError};
use crate::client::dynamodb_client;

use crate::adapters::memo_events::processors::model::{NotificationType, NotificationStatus};
use crate::utils::utils::struct_to_hashmap;

use super::{
  event_type_processor::{EventTypeProcessorInterface, ProcessorError },
  model::{CreateMessageProcessor, CreateMessageProcessorOption, CreateMessageBody, DBNotifcation},
};

#[async_trait]
impl EventTypeProcessorInterface for CreateMessageProcessor {
  type Input = CreateMessageProcessorOption;

  fn new(input: Self::Input) -> Self {
    let parsed: CreateMessageBody = serde_json::from_str(input.body.as_str()).expect("CreateMessageBody was not well-formatted");
    CreateMessageProcessor {
      event_type: input.event_type,
      body: parsed,
    }
  }

  async fn process(&self) -> Result<(), ProcessorError>{
    eprintln!("CreateMessageProcessor::process {:?} ", self.body);
    let db_client = dynamodb_client::get().unwrap();
    // create notification struct
    let d = DBNotifcation{
      notification_id: self.body.detail.notification_id.clone(),
      user_id: self.body.detail.user_id.clone(),
      replyer_id: self.body.detail.replyer_id.clone(),
      notification_type: NotificationType::Message,
      status: NotificationStatus::Unread,
      topic_id: self.body.detail.topic_id.clone(),
      message_id: self.body.detail.message_id.clone(),
      content: self.body.detail.content.clone(),
      created_time: self.body.detail.created_time.clone(),
      updated_time: None,
    };

    let condition_expression = "attribute_not_exists(PK) AND attribute_not_exists(SK)";

    let item = struct_to_hashmap(&d)
            .map_err(|_| PermanentError::new("Failed to convert struct to hashmap"))?;

    let resp = db_client.put_item()
        .table_name("memo-management")
        .set_item(Some(item))
        .condition_expression(condition_expression)
        .send()
        .await
        .map_err(|e| RetryableError::new(&e.to_string()))?;
    println!("dynamodb resp: {:?}", resp);

    Ok(())
  }
}
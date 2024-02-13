use async_trait::async_trait;

use crate::{
  services::notification::NotificationServiceInterface,
  adapters::memo_events::processors::event_type_processor::{EventTypeProcessorInterface, ApplicationError, PermanentError },
  adapters::memo_events::processors::model::{CreateMessageProcessor, CreateMessageProcessorOption, CreateMessageBody}
};

#[async_trait]
impl EventTypeProcessorInterface for CreateMessageProcessor {
  type Input = CreateMessageProcessorOption;

  fn new(input: Self::Input) -> Self {
    eprintln!("CreateMessageProcessor::new {:?}", input);
    CreateMessageProcessor {
      event_type: input.event_type,
      notification_service: input.notification_service,
    }
  }

  async fn process(&self, body: String) -> Result<(), ApplicationError>{
    // let parsed: CreateMessageBody = serde_json::from_str(input.body.as_str()).expect("CreateMessageBody was not well-formatted");
    // .expect("CreateMessageBody was not well-formatted");
    let parsed: CreateMessageBody = serde_json::from_str(body.as_str()).map_err(|e| ApplicationError::PermanentError(PermanentError{ message: format!("CreateMessageBody was not well-formatted {:?}", e) }))?;
    tracing::info!("CreateMessageProcessor::process {:?}", parsed);
    self.notification_service.create_notification_message(parsed).await?;
    Ok(())
  }
}
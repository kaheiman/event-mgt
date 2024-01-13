use std::time::Duration;

use aws_sdk_sqs::{Client as SQSClient, types::{Message, QueueAttributeName, MessageSystemAttributeName}};

use async_trait::async_trait;
use serde_json::Value;
use tokio::time::sleep;

use crate::services::notification::NotificationService;

use super::processors::{model::{CreateMessageProcessor, CreateMessageProcessorOption}, event_type::MemoEventTypes, event_type_processor::EventTypeProcessorInterface};

#[async_trait]
pub trait SQSPollerInterface {
    async fn new(option: SQSPollerOption) -> Self;
    async fn start_processing(&mut self);
    async fn stop_processing(&self);
    async fn poll_once(&mut self);
    async fn delegate_event_to_processor(&self, message: Message);
}


pub struct SQSPollerOption {
    pub sqs_client: SQSClient,
    pub sqs_queue: String,
    pub failure_queue: String,
    pub wait_time_seconds: Option<i32>,
    pub max_number_of_messages: Option<i32>,
    pub max_retry: Option<i32>,
    pub notification_service: NotificationService,
}

pub struct SQSPoller {
    processing: bool,
    sqs_client: SQSClient,
    failure_queue: String,
    sqs_queue: String,
    wait_time_seconds: i32,
    max_number_of_messages: i32,
    max_retry: i32,
    create_message_processor: CreateMessageProcessor,
}

#[async_trait]
impl SQSPollerInterface for SQSPoller {
    async fn new(option: SQSPollerOption) -> Self {
        let create_message_processor = CreateMessageProcessor::new(CreateMessageProcessorOption {
            event_type: MemoEventTypes::CreateMessage,
            notification_service: option.notification_service,
        });
        SQSPoller {
            processing: false,
            sqs_client: option.sqs_client,
            sqs_queue: option.sqs_queue,
            failure_queue: option.failure_queue,
            wait_time_seconds: option.wait_time_seconds.unwrap_or(10),
            max_retry: option.max_retry.unwrap_or(10),
            max_number_of_messages: option.max_number_of_messages.unwrap_or(10),
            create_message_processor: create_message_processor,
        }
    }

    async fn start_processing(&mut self) {
        self.processing = true;
        let mut i = 0;
        while self.processing {
            self.poll_once().await;
            println!("Sleeping for 5 seconds... {:?}", i);
            sleep(Duration::from_secs(5)).await;
            i += 1;
        }
    }

    async fn poll_once(&mut self) {
        let resp = self.sqs_client.receive_message()
            .queue_url(self.sqs_queue.clone())
            .max_number_of_messages(self.max_number_of_messages)
            .attribute_names(QueueAttributeName::All)
            .wait_time_seconds(self.wait_time_seconds)
            .send()
            .await
            .unwrap();

        match resp.messages {
            Some(messages) => {
                for message in messages {
                    println!("Received Message: {:?}", message);
                    self.delegate_event_to_processor(message).await;
                    // let receipt_handle = message.receipt_handle.unwrap();
                    // let body = message.body.unwrap();
                    // let message_id = message.message_id.unwrap();
                    // let message_attributes = message.message_attributes.unwrap();
                    // let message_type = message_attributes.get("message_type").unwrap().string_value.unwrap();
                    // let message_type = message_type.parse::<MemoEventTypes>().unwrap();
                }
            },
            None => {
                println!("No messages found...");
            }
        }
    }

    async fn delegate_event_to_processor(&self, message: Message) {
        let mut reveived_count: Option<i32> = None;
        if let Some(attribute) = message.attributes {
            if let Some(count) = attribute.get(&MessageSystemAttributeName::ApproximateReceiveCount) {
                reveived_count = count.parse::<i32>().ok();
            }
        }
        if let Some(body) = message.body {
            if let Ok(json) = serde_json::from_str::<Value>(&body) {
                // Extract detail.type
                if let Some(detail_type) = json["detail"]["type"].as_str() {
                    match detail_type {
                        "memo:message.created-1.0.0" => {
                            let result = self.create_message_processor.process(body).await;
                            match result {
                                Ok(_) => {
                                    println!("EventProcessor::delegate_event_to_processor: CreateMessageProcessor::process: Ok");
                                },
                                Err(err) => {
                                    if let Some(reveived_count) = reveived_count {
                                        println!("reveived_count: {:?}", reveived_count);
                                    }
                                    println!("EventProcessor::delegate_event_to_processor: CreateMessageProcessor::process: Err: {:?}", err);
                                },

                            }
                            // response reuslt error
                            // if error check how many time and put back into deadletter queue
                            // if true remove from queue
                        },
                        _ => {
                            println!("EventProcessor::delegate_event_to_processor: event_type is not found");
                        },
                    }
                    // Here you can process the extracted detail.type
                }
            }

        }
    }

    async fn stop_processing(&self) {
        // Stop processing logic
    }
}
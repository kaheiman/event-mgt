use aws_sdk_sqs::{Client as SQSClient, types::{QueueAttributeName, Message, MessageSystemAttributeName}};
use serde_json::Value;
use tokio::task;
use async_trait::async_trait;
use tokio::time::{sleep, Duration};

use super::processors::{event_type::MemoEventTypes, model::{CreateMessageProcessorOption, CreateMessageProcessor}, event_type_processor::EventTypeProcessorInterface};

#[async_trait]
pub trait SQSPollerInterface {
    async fn new(option: SQSPollerOption) -> Self;
    async fn start_processing(&mut self);
    async fn stop_processing(&self);
}

#[async_trait]
trait SQSPollerPrivateInterface {
    async fn poll_once(poller: SQSPoller);
    async fn delegate_event_to_processor(message: Message);
}


pub struct SQSPollerOption {
    pub sqs_client: SQSClient,
    pub sqs_queue: String,
    pub failure_queue: String,
    pub wait_time_seconds: Option<i32>,
    pub max_number_of_messages: Option<i32>,
    pub max_retry: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct SQSPoller {
    processing: bool,
    sqs_client: SQSClient,
    failure_queue: String,
    sqs_queue: String,
    wait_time_seconds: i32,
    max_number_of_messages: i32,
    max_retry: i32,
}


#[async_trait]
impl SQSPollerPrivateInterface for SQSPoller {

    async fn poll_once(poller: SQSPoller) {
        let resp = poller.sqs_client.receive_message()
            .queue_url(poller.sqs_queue)
            .max_number_of_messages(poller.max_number_of_messages)
            .attribute_names(QueueAttributeName::All)
            .wait_time_seconds(poller.wait_time_seconds)
            .send()
            .await
            .unwrap();

        match resp.messages {
            Some(messages) => {
                for message in messages {
                    println!("Received Message: {:?}", message);
                    SQSPoller::delegate_event_to_processor(message).await;
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

    async fn delegate_event_to_processor(message: Message) {
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
                            let result = CreateMessageProcessor::new(CreateMessageProcessorOption {
                                event_type: MemoEventTypes::CreateMessage,
                                body: body,
                            }).process().await;
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
}


#[async_trait]
impl SQSPollerInterface for SQSPoller {
    async fn new(option: SQSPollerOption) -> Self {
        SQSPoller {
            processing: false,
            sqs_client: option.sqs_client,
            sqs_queue: option.sqs_queue,
            failure_queue: option.failure_queue,
            wait_time_seconds: option.wait_time_seconds.unwrap_or(10),
            max_retry: option.max_retry.unwrap_or(10),
            max_number_of_messages: option.max_number_of_messages.unwrap_or(10),
        }
    }

    async fn start_processing(&mut self) {
        self.processing = true;
        let mut i = 0;
        while self.processing {
            let processing_talk = task::spawn(
                SQSPoller::poll_once(self.clone())
            );
            let _ = processing_talk.await;
            println!("Sleeping for 5 seconds... {:?}", i);
            sleep(Duration::from_secs(5)).await;
            i += 1;
        }
    }

    async fn stop_processing(&self) {
        // Stop processing logic
    }
}
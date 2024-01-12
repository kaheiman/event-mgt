use serde::{Serialize, Deserialize};

use super::event_type::MemoEventTypes;

pub struct CreateMessageProcessor {
  pub event_type: MemoEventTypes,
  pub body: CreateMessageBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageBody {
  pub version: String,
  pub id: String,
  #[serde(rename = "detail-type")]
  pub detail_type: String,
  pub source: String,
  pub account: String,
  pub time: String,
  pub region: String,
  pub resources: Vec<String>,
  pub detail: CreateMessageDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageDetail {
  pub notification_id: String,
  #[serde(rename = "type")]
  pub message_filter_type: String,
  pub user_id: String,
  pub replyer_id: String,
  pub topic_id: String,
  pub message_id: String,
  pub content: String,
  pub created_time: String,
}

pub struct CreateMessageProcessorOption {
  pub event_type: MemoEventTypes,
  pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NotificationType {
    Message,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NotificationStatus {
    Read,
    Unread,
    Removed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DBNotifcation {
    #[serde(rename = "PK")]
    pub notification_id: String,
    #[serde(rename = "SK")]
    pub user_id: String,
    pub replyer_id: String,
    pub notification_type: NotificationType,
    pub status: NotificationStatus,
    pub topic_id: String,
    pub message_id: String,
    pub content: String,
    pub created_time: String,
    pub updated_time: Option<String>,
}
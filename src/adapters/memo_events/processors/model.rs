use serde::{Serialize, Deserialize};

use crate::services::notification::NotificationService;

use super::event_type::MemoEventTypes;

pub struct CreateMessageProcessor {
  pub event_type: MemoEventTypes,
  pub notification_service: NotificationService,
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
  pub event_type: String,
  pub user_id: String,
  pub replyer_id: String,
  pub replyer_name: String,
  pub replyer_avatar: String,
  pub topic_id: String,
  pub message_id: String,
  pub content: String,
  pub created_time: String,
}

#[derive(Debug)]
pub struct CreateMessageProcessorOption {
  pub event_type: MemoEventTypes,
  pub notification_service: NotificationService,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NotificationType {
    Message,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NotificationStatus {
    READ,
    UNREAD,
    REMOVED,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DBNotifcation {
    #[serde(rename = "SK")]
    pub notification_id: String,
    #[serde(rename = "PK")]
    pub user_id: String,
    pub replyer_id: String,
    pub replyer_avatar: String,
    pub replyer_name: String,
    pub notification_type: NotificationType,
    pub status: NotificationStatus,
    pub topic_id: String,
    pub message_id: String,
    pub content: String,
    pub created_time: String,
    pub updated_time: Option<String>,
}
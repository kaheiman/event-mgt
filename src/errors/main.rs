use std::fmt;

use aws_sdk_dynamodb::Error as DynamoDbError;
use aws_sdk_eventbridge::Error as EventBridgeError;

#[derive(Debug)]
pub struct SerializationError {
    message: String,
}

impl SerializationError {
    pub fn new(message: &str) -> Self {
        SerializationError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Serialization Error: {}", self.message)
    }
}

impl std::error::Error for SerializationError {}

#[derive(Debug)]
pub enum SystemError {
    DynamoDbError(DynamoDbError),
    SerializationError(SerializationError),
    EventBridgeError(EventBridgeError),
}

impl From<DynamoDbError> for SystemError {
    fn from(error: DynamoDbError) -> Self {
        SystemError::DynamoDbError(error)
    }
}

impl From<EventBridgeError> for SystemError {
    fn from(error: EventBridgeError) -> Self {
        SystemError::EventBridgeError(error)
    }
}

impl From<SerializationError> for SystemError {
    fn from(error: SerializationError) -> Self {
        SystemError::SerializationError(error)
    }
}
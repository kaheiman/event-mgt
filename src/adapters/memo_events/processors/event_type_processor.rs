use async_trait::async_trait;
use std::fmt;

#[derive(Debug)]
pub struct PermanentError {
    message: String,
}

impl PermanentError {
    pub fn new(message: &str) -> Self {
        PermanentError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for PermanentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Permanent Error: {}", self.message)
    }
}

impl std::error::Error for PermanentError {}

#[derive(Debug)]
pub struct RetryableError {
    message: String,
}

impl RetryableError {
    pub fn new(message: &str) -> Self {
        RetryableError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for RetryableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Retryable Error: {}", self.message)
    }
}

impl std::error::Error for RetryableError {}

#[derive(Debug)]
pub enum ProcessorError {
    RetryableError(RetryableError),
    PermanentError(PermanentError),
}

impl From<RetryableError> for ProcessorError {
    fn from(error: RetryableError) -> Self {
        ProcessorError::RetryableError(error)
    }
}

impl From<PermanentError> for ProcessorError {
    fn from(error: PermanentError) -> Self {
        ProcessorError::PermanentError(error)
    }
}

#[async_trait]
pub trait EventTypeProcessorInterface {
    type Input;
    fn new(input: Self::Input) -> Self;
    async fn process(&self) -> Result<(), ProcessorError>;
}
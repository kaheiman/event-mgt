use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError, types::AttributeValue};
use std::collections::HashMap;
use async_trait::async_trait;

// Define the trait for database operations
#[async_trait]
pub trait DatabaseStoreInterface {
    async fn db_create_notification_message(&self, item: HashMap<String, AttributeValue>, condition: String) -> Result<(), DynamoDbError>;
    async fn update_notification_message(&self, /* parameters */) -> Result<(), DynamoDbError>;
    async fn get_notification_message(&self, user_id: String) -> Result<(), DynamoDbError>;
}

// Define the struct implementing the trait
#[derive(Debug, Clone)]
pub struct DatabaseStoreService {
    pub store: DynamoDbClient,
    pub table_name: String,
}

#[async_trait]
impl DatabaseStoreInterface for DatabaseStoreService {
    async fn db_create_notification_message(&self, item: HashMap<String, AttributeValue>, condition: String) -> Result<(), DynamoDbError> {
        let _ = self.store.put_item()
            .table_name(self.table_name.clone()) //memo-management
            .set_item(Some(item))
            .condition_expression(condition)
            .send()
            .await?;

        Ok(())
    }

    async fn update_notification_message(&self, /* parameters */) -> Result<(), DynamoDbError> {
        // Implementation for updating a notification message in DynamoDB
        Ok(())
    }

    async fn get_notification_message(&self, user_id: String) -> Result<(), DynamoDbError> {
        // Implementation for getting a notification message from DynamoDB
        Ok(())
    }
}

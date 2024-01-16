use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError, types::AttributeValue, operation::{query::QueryOutput, get_item::GetItemOutput}};
use std::collections::HashMap;
use async_trait::async_trait;

use crate::adapters::memo_events::processors::model::NotificationStatus;

// Define the trait for database operations
#[async_trait]
pub trait DatabaseStoreInterface {
    async fn db_create_notification_message(&self, item: HashMap<String, AttributeValue>, condition: String) -> Result<(), DynamoDbError>;
    async fn db_update_notification_message(&self, user_id: String, noti_id: String, status: NotificationStatus) -> Result<(), DynamoDbError>;
    async fn db_get_notifications_by_user_id(&self, user_id: String) -> Result<QueryOutput, DynamoDbError>;
    async fn db_get_notification_item_with_pk_sk(&self, pk_value: String, sk_value: String) -> Result<GetItemOutput, DynamoDbError>;
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

    async fn db_get_notification_item_with_pk_sk(&self, pk_value: String, sk_value: String) -> Result<GetItemOutput, DynamoDbError> {
        let mut key = HashMap::new();
        key.insert("PK".to_string(), AttributeValue::S(pk_value.to_string()));
        key.insert("SK".to_string(), AttributeValue::S(sk_value.to_string()));

        let result = self.store.get_item()
            .table_name(self.table_name.clone())
            .set_key(Some(key))
            .send()
            .await?;

        Ok(result)
    }

    async fn db_update_notification_message(&self, user_id: String, noti_id: String, status: NotificationStatus) -> Result<(), DynamoDbError> {
        let user_id_attr = AttributeValue::S(format!("USR#{}", user_id));
        let noti_id_attr = AttributeValue::S(format!("NTF#{}", noti_id));
        let status = AttributeValue::S(format!("{:?}", status));

        self.store.update_item()
            .table_name(self.table_name.clone())
            .key("PK".to_string(), user_id_attr)
            .key("SK".to_string(), noti_id_attr)
            .update_expression("SET #status = :new_status")
            .expression_attribute_names("#status", "status")
            .expression_attribute_values(":new_status", status)
            .send()
            .await?;

        Ok(())
    }

    async fn db_get_notifications_by_user_id(&self, user_id: String) -> Result<QueryOutput, DynamoDbError> {
        // Implementation for getting a notification message from DynamoDB
        let user_id_attr = AttributeValue::S(format!("USR#{}", user_id));
        let unread_status_attr = AttributeValue::S(format!("{:?}", NotificationStatus::UNREAD));

        let result = self.store.query()
            .table_name(self.table_name.clone())
            .key_condition_expression("#pk = :user_id")
            .filter_expression("#status = :unread_status")
            .expression_attribute_names("#pk", "PK")
            .expression_attribute_names("#status", "status")
            .expression_attribute_values(":user_id", user_id_attr)
            .expression_attribute_values(":unread_status", unread_status_attr)
            .scan_index_forward(false)
            .limit(20)
            .send()
            .await?;

        Ok(result)
    }
}

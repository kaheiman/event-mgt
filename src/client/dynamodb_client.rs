use aws_config::SdkConfig;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use lazy_static::lazy_static;
use std::sync::Mutex;

// Define a global client within a Mutex for thread safety
lazy_static! {
    static ref DYNAMO_DB_CLIENT: Mutex<Option<DynamoDbClient>> = Mutex::new(None);
}

// Initialize the DynamoDB client
pub async fn init(config: &SdkConfig) {
    let client = DynamoDbClient::new(config);
    let mut global_client = DYNAMO_DB_CLIENT.lock().unwrap();
    *global_client = Some(client);
}

// Retrieve the DynamoDB client
pub fn get() -> Option<DynamoDbClient> {
    DYNAMO_DB_CLIENT.lock().unwrap().clone()
}

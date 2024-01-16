// tracing
export RUST_LOG=info

// EventBridge Example
let event_bridge_client = EventBridgeClient::new(&config);

let event_bridge_data = EventData {
    event_type: "memo:message.created-1.0.0".to_string(),
    notification_id: "notif123".to_string(),
    user_id: "user456".to_string(),
    replyer_id: "replyer789".to_string(),
    topic_id: "topic345".to_string(),
    message_id: "message678".to_string(),
    content: "Example content".to_string(),
    created_time: "2023-01-01T00:00:00Z".to_string(),
};

if let Err(e) = send_notification_event_to_eventbridge(&event_bridge_data, &event_bridge_client).await {
    eprintln!("Failed to send event to eventBridge: {:?}", e);
}

async fn send_notification_event_to_eventbridge(event: &EventData, client: &EventBridgeClient) -> Result<(), SystemError> {

    // Serialize EventData to JSON
    let event_json = serde_json::to_string(event).map_err(|e| SerializationError::new(&e.to_string()))?;

    print!("event response: {:?}", event_json);

    let event_entry = PutEventsRequestEntry::builder()
        .detail(event_json)
        .detail_type("Memo Event Type") // Replace with your detail type
        .source("rust-test") // Replace with your source
        .event_bus_name("eventbridge-memo-events-listener")
        .build();

    let response = client.put_events()
        .entries(event_entry)
        .send()
        .await
        .map_err(|e| EventBridgeError::from(e))?;

    println!("EventBridge response: {:?}", response);

    Ok(())
}

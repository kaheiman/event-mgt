use crate::event_processor::GenericEvent;
use crate::event_processor::EventHandler;

pub struct CreateMessageEventHandler;

impl EventHandler for CreateMessageEventHandler {
    fn handle_event(&self, event: &GenericEvent) -> Result<(), String> {
        // Process apple event
        Ok(())
    }
}
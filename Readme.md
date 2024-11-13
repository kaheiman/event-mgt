# RUST API Server and Event Processor

This repository combines a **RUST API Server** with an **Event Processor**.

## Overview

- **API Server**:
  - Retrieves data from DynamoDB.
  - Sends events to EventBridge.

- **Event Processor**:
  - Listens for events from EventBridge.
  - Processes incoming events.

## Getting Started

Clone the repository and follow the setup instructions to configure the API Server and Event Processor.

### Requirements

- Rust
- DynamoDB
- EventBridge

### Usage

1. **Run the API Server** to start retrieving data and dispatching events.
2. **Start the Event Processor** to listen for events and handle processing.

## License

This project is licensed under the MIT License.

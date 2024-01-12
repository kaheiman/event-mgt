use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use serde::Serialize;
use serde_json::{to_string, Value};


use crate::errors::main::SerializationError;

pub fn struct_to_hashmap<T: Serialize>(t: &T) -> Result<HashMap<String, AttributeValue>, SerializationError> {
    // Convert the struct to a JSON string
    let json = to_string(t).map_err(|e| SerializationError::new(&e.to_string()))?;
    let map: HashMap<String, Value> = serde_json::from_str(&json).map_err(|e| SerializationError::new(&e.to_string()))?;

    let hashmap = map.into_iter().map(|(k, v)| {
        let attr_value = match v {
            Value::String(s) => AttributeValue::S(s),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    AttributeValue::N(i.to_string())
                } else if let Some(f) = n.as_f64() {
                    AttributeValue::N(f.to_string())
                } else {
                    AttributeValue::Null(true)
                }
            },
            Value::Bool(b) => AttributeValue::Bool(b),
            Value::Null => AttributeValue::Null(true),
            // Add other conversions as needed (e.g., lists, maps)
            _ => AttributeValue::Null(true), // Fallback for unsupported types
        };
        (k, attr_value)
    }).collect();

    Ok(hashmap)
}
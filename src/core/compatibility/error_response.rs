use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FireflyErrorResponse {
    pub message: String,
    pub errors: HashMap<String, Vec<String>>,
}

impl FireflyErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            errors: HashMap::new(),
        }
    }
}

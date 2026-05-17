use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct SecurityEvent {
    pub actor: Option<Uuid>,
    pub event_type: String,
    pub outcome: String,
    pub reason_code: Option<String>,
    pub request_id: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default)]
pub struct AuditLogger;

impl AuditLogger {
    pub fn emit(&self, mut event: SecurityEvent) {
        if event.timestamp.is_empty() {
            event.timestamp = Utc::now().to_rfc3339();
        }

        tracing::info!(target: "audit", event = ?event, "security event");
    }

    pub fn new_event(
        event_type: &str,
        outcome: &str,
        actor: Option<Uuid>,
        reason_code: Option<&str>,
        request_id: Option<String>,
    ) -> SecurityEvent {
        SecurityEvent {
            actor,
            event_type: event_type.to_string(),
            outcome: outcome.to_string(),
            reason_code: reason_code.map(|value| value.to_string()),
            request_id,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

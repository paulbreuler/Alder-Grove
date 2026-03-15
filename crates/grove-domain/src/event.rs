use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Lifecycle,
    Action,
    Gate,
    Content,
    Error,
    Metric,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventEmitter {
    Agent,
    System,
    Human,
}

/// Append-only event — no updated_at field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: Uuid,
    pub session_id: Uuid,
    pub event_type: String,
    pub category: EventCategory,
    pub summary: String,
    pub data: serde_json::Value,
    pub emitted_by: EventEmitter,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn event_category_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&EventCategory::Lifecycle).unwrap(),
            "\"lifecycle\""
        );
    }

    #[test]
    fn event_emitter_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&EventEmitter::Agent).unwrap(),
            "\"agent\""
        );
    }

    #[test]
    fn event_serde_roundtrip() {
        let e = Event {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            event_type: "session_started".into(),
            category: EventCategory::Lifecycle,
            summary: "Session started".into(),
            data: serde_json::json!({}),
            emitted_by: EventEmitter::System,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(e.id, back.id);
        assert_eq!(e.event_type, back.event_type);
        assert_eq!(e.category, back.category);
        assert_eq!(e.emitted_by, back.emitted_by);
    }

    #[test]
    fn event_has_no_updated_at() {
        let json = serde_json::json!({
            "id": Uuid::now_v7(),
            "session_id": Uuid::now_v7(),
            "event_type": "file_modify",
            "category": "action",
            "summary": "Modified file",
            "data": {"path": "src/main.rs"},
            "emitted_by": "agent",
            "created_at": "2026-03-14T12:00:00Z"
        });
        let event: Event = serde_json::from_value(json).unwrap();
        // Event struct has no updated_at field — if it compiled, it works
        assert_eq!(event.event_type, "file_modify");
    }
}

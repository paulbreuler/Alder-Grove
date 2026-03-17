use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum EventEmitter {
    Agent,
    System,
    Human,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Event {
    pub id: Uuid,
    pub session_id: Uuid,
    pub workspace_id: Uuid,
    pub event_type: String,
    pub category: EventCategory,
    pub summary: String,
    pub data: serde_json::Value,
    pub emitted_by: EventEmitter,
    pub created_at: DateTime<Utc>,
}

impl Event {
    /// Create a lifecycle event (session started, completed, etc.)
    pub fn lifecycle(
        session_id: Uuid,
        workspace_id: Uuid,
        event_type: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            session_id,
            workspace_id,
            event_type: event_type.into(),
            category: EventCategory::Lifecycle,
            summary: summary.into(),
            data: serde_json::json!({}),
            emitted_by: EventEmitter::System,
            created_at: Utc::now(),
        }
    }

    /// Create an action event (file modified, command run, etc.)
    pub fn action(
        session_id: Uuid,
        workspace_id: Uuid,
        event_type: impl Into<String>,
        summary: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            session_id,
            workspace_id,
            event_type: event_type.into(),
            category: EventCategory::Action,
            summary: summary.into(),
            data,
            emitted_by: EventEmitter::System,
            created_at: Utc::now(),
        }
    }

    /// Create a gate event (gate triggered, approved, denied, etc.)
    pub fn gate_event(
        session_id: Uuid,
        workspace_id: Uuid,
        event_type: impl Into<String>,
        summary: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            session_id,
            workspace_id,
            event_type: event_type.into(),
            category: EventCategory::Gate,
            summary: summary.into(),
            data,
            emitted_by: EventEmitter::System,
            created_at: Utc::now(),
        }
    }

    /// Create an error event
    pub fn error(
        session_id: Uuid,
        workspace_id: Uuid,
        event_type: impl Into<String>,
        summary: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            session_id,
            workspace_id,
            event_type: event_type.into(),
            category: EventCategory::Error,
            summary: summary.into(),
            data,
            emitted_by: EventEmitter::System,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn event_lifecycle_factory() {
        let session_id = Uuid::now_v7();
        let workspace_id = Uuid::now_v7();
        let e = Event::lifecycle(
            session_id,
            workspace_id,
            "session_started",
            "Session started",
        );
        assert_eq!(e.session_id, session_id);
        assert_eq!(e.workspace_id, workspace_id);
        assert_eq!(e.event_type, "session_started");
        assert_eq!(e.category, EventCategory::Lifecycle);
        assert_eq!(e.emitted_by, EventEmitter::System);
        assert_eq!(e.data, serde_json::json!({}));
    }

    #[test]
    fn event_action_factory() {
        let session_id = Uuid::now_v7();
        let workspace_id = Uuid::now_v7();
        let data = serde_json::json!({"path": "src/main.rs", "lines_changed": 42});
        let e = Event::action(
            session_id,
            workspace_id,
            "file_modify",
            "Modified main.rs",
            data.clone(),
        );
        assert_eq!(e.category, EventCategory::Action);
        assert_eq!(e.workspace_id, workspace_id);
        assert_eq!(e.data, data);
    }

    #[test]
    fn event_gate_factory() {
        let session_id = Uuid::now_v7();
        let workspace_id = Uuid::now_v7();
        let gate_id = Uuid::now_v7();
        let data = serde_json::json!({"gate_id": gate_id.to_string()});
        let e = Event::gate_event(
            session_id,
            workspace_id,
            "gate_triggered",
            "Gate triggered for review",
            data,
        );
        assert_eq!(e.category, EventCategory::Gate);
        assert_eq!(e.workspace_id, workspace_id);
        assert_eq!(e.event_type, "gate_triggered");
    }

    #[test]
    fn event_error_factory() {
        let session_id = Uuid::now_v7();
        let workspace_id = Uuid::now_v7();
        let data = serde_json::json!({"error": "connection refused", "retries": 3});
        let e = Event::error(
            session_id,
            workspace_id,
            "api_error",
            "API call failed",
            data.clone(),
        );
        assert_eq!(e.category, EventCategory::Error);
        assert_eq!(e.workspace_id, workspace_id);
        assert_eq!(e.data, data);
    }

    #[test]
    fn event_factories_generate_unique_ids() {
        let sid = Uuid::now_v7();
        let wid = Uuid::now_v7();
        let e1 = Event::lifecycle(sid, wid, "a", "A");
        let e2 = Event::lifecycle(sid, wid, "b", "B");
        assert_ne!(e1.id, e2.id);
    }

    #[test]
    fn event_category_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&EventCategory::Lifecycle).unwrap(),
            r#""lifecycle""#
        );
        assert_eq!(
            serde_json::to_string(&EventCategory::Action).unwrap(),
            r#""action""#
        );
        assert_eq!(
            serde_json::to_string(&EventCategory::Gate).unwrap(),
            r#""gate""#
        );
        assert_eq!(
            serde_json::to_string(&EventCategory::Content).unwrap(),
            r#""content""#
        );
        assert_eq!(
            serde_json::to_string(&EventCategory::Error).unwrap(),
            r#""error""#
        );
        assert_eq!(
            serde_json::to_string(&EventCategory::Metric).unwrap(),
            r#""metric""#
        );
    }

    #[test]
    fn event_emitter_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&EventEmitter::Agent).unwrap(),
            r#""agent""#
        );
        assert_eq!(
            serde_json::to_string(&EventEmitter::System).unwrap(),
            r#""system""#
        );
        assert_eq!(
            serde_json::to_string(&EventEmitter::Human).unwrap(),
            r#""human""#
        );
    }

    #[test]
    fn event_serde_roundtrip() {
        let now = Utc::now();
        let event = Event {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            event_type: "file_modify".into(),
            category: EventCategory::Action,
            summary: "Modified main.rs".into(),
            data: serde_json::json!({"path": "src/main.rs"}),
            emitted_by: EventEmitter::Agent,
            created_at: now,
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(event.id, back.id);
        assert_eq!(event.session_id, back.session_id);
        assert_eq!(event.workspace_id, back.workspace_id);
        assert_eq!(event.event_type, back.event_type);
        assert_eq!(event.category, back.category);
        assert_eq!(event.emitted_by, back.emitted_by);
    }
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Summary of a gate for ACP messaging — lighter than the full Gate entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateSummary {
    pub id: Uuid,
    pub reason: String,
    pub context: serde_json::Value,
}

/// ACP protocol message — the envelope for all agent-human communication.
///
/// Internally tagged by `"type"` with associated `"payload"` data.
/// Variants cover gate decisions, user messages, agent events,
/// gate requests, session state changes, and errors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum AcpMessage {
    #[serde(rename = "gate_decision")]
    GateDecision {
        gate_id: Uuid,
        approved: bool,
        reason: Option<String>,
    },
    #[serde(rename = "user_message")]
    UserMessage { content: String },
    #[serde(rename = "agent_event")]
    AgentEvent { event: crate::event::Event },
    #[serde(rename = "gate_request")]
    GateRequest { gate: GateSummary },
    #[serde(rename = "session_state_change")]
    SessionStateChange {
        session_id: Uuid,
        status: crate::session::SessionStatus,
    },
    #[serde(rename = "error")]
    Error { code: String, message: String },
}

/// WebSocket frame — multiplexes ACP messages and CRDT sync on a single connection.
///
/// Tagged by `"channel"` to distinguish frame types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "channel")]
pub enum WsFrame {
    #[serde(rename = "acp")]
    Acp { message: AcpMessage },
    #[serde(rename = "sync")]
    Sync {
        document_id: String,
        update: Vec<u8>,
    },
    #[serde(rename = "awareness")]
    Awareness { states: Vec<u8> },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventCategory, EventEmitter};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn acp_message_gate_decision_serde() {
        let gate_id = Uuid::now_v7();
        let msg = AcpMessage::GateDecision {
            gate_id,
            approved: true,
            reason: Some("Looks good".into()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["type"], "gate_decision");
        assert_eq!(val["payload"]["approved"], true);
        assert_eq!(val["payload"]["reason"], "Looks good");

        let back: AcpMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn acp_message_session_state_change_serde() {
        let session_id = Uuid::now_v7();
        let msg = AcpMessage::SessionStateChange {
            session_id,
            status: crate::session::SessionStatus::Active,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["type"], "session_state_change");
        assert_eq!(val["payload"]["status"], "active");

        let back: AcpMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn ws_frame_acp_channel_serde() {
        let msg = AcpMessage::UserMessage {
            content: "Hello agent".into(),
        };
        let frame = WsFrame::Acp {
            message: msg.clone(),
        };
        let json = serde_json::to_string(&frame).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["channel"], "acp");

        let back: WsFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(frame, back);
    }

    #[test]
    fn ws_frame_sync_channel_serde() {
        let frame = WsFrame::Sync {
            document_id: "doc-123".into(),
            update: vec![0x01, 0x02, 0x03],
        };
        let json = serde_json::to_string(&frame).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["channel"], "sync");
        assert_eq!(val["document_id"], "doc-123");

        let back: WsFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(frame, back);
    }

    #[test]
    fn acp_message_agent_event_serde() {
        let event = Event {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            event_type: "file_modify".into(),
            category: EventCategory::Action,
            summary: "Modified main.rs".into(),
            data: serde_json::json!({"path": "src/main.rs"}),
            emitted_by: EventEmitter::Agent,
            created_at: Utc::now(),
        };
        let msg = AcpMessage::AgentEvent {
            event: event.clone(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["type"], "agent_event");

        let back: AcpMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn acp_message_error_serde() {
        let msg = AcpMessage::Error {
            code: "RATE_LIMIT".into(),
            message: "Too many requests".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["type"], "error");
        assert_eq!(val["payload"]["code"], "RATE_LIMIT");

        let back: AcpMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn acp_message_gate_request_serde() {
        let gate = GateSummary {
            id: Uuid::now_v7(),
            reason: "Deleting auth module".into(),
            context: serde_json::json!({"file": "src/auth.rs"}),
        };
        let msg = AcpMessage::GateRequest { gate };
        let json = serde_json::to_string(&msg).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["type"], "gate_request");
        assert_eq!(val["payload"]["gate"]["reason"], "Deleting auth module");

        let back: AcpMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn ws_frame_awareness_channel_serde() {
        let frame = WsFrame::Awareness {
            states: vec![0xAA, 0xBB],
        };
        let json = serde_json::to_string(&frame).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["channel"], "awareness");

        let back: WsFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(frame, back);
    }
}

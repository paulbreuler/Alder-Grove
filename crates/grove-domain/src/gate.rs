use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    Automatic,
    Manual,
    Threshold,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    Single,
    AnyOf,
    AllOf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutAction {
    Cancel,
    Approve,
    Escalate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Pending,
    Approved,
    Denied,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateDefinition {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub trigger_config: serde_json::Value,
    pub approval_type: ApprovalType,
    pub timeout_minutes: Option<i32>,
    pub timeout_action: TimeoutAction,
    pub enabled: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Gate {
    pub id: Uuid,
    pub session_id: Uuid,
    pub gate_definition_id: Option<Uuid>,
    pub status: GateStatus,
    pub reason: String,
    pub context: serde_json::Value,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_rationale: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn trigger_type_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&TriggerType::Automatic).unwrap(),
            "\"automatic\""
        );
        assert_eq!(
            serde_json::to_string(&TriggerType::Threshold).unwrap(),
            "\"threshold\""
        );
    }

    #[test]
    fn gate_definition_serde_roundtrip() {
        let gd = GateDefinition {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Code Deletion Review".into(),
            description: Some("Requires approval before deleting code".into()),
            trigger_type: TriggerType::Automatic,
            trigger_config: serde_json::json!({
                "patterns": [
                    {"event_type": "file_delete", "match": "**/*.rs"}
                ]
            }),
            approval_type: ApprovalType::Single,
            timeout_minutes: Some(60),
            timeout_action: TimeoutAction::Cancel,
            enabled: true,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&gd).unwrap();
        let back: GateDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(gd.id, back.id);
        assert_eq!(gd.trigger_type, back.trigger_type);
        assert_eq!(gd.timeout_action, back.timeout_action);
    }

    #[test]
    fn gate_serde_roundtrip() {
        let g = Gate {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            gate_definition_id: Some(Uuid::now_v7()),
            status: GateStatus::Pending,
            reason: "Agent attempted to delete auth types".into(),
            context: serde_json::json!({"triggering_event": {"type": "file_delete"}}),
            decided_by: None,
            decided_at: None,
            decision_rationale: None,
            expires_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&g).unwrap();
        let back: Gate = serde_json::from_str(&json).unwrap();
        assert_eq!(g.id, back.id);
        assert_eq!(g.status, back.status);
        assert!(back.decided_by.is_none());
    }
}

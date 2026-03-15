use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

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

impl Gate {
    /// Require the gate to be Pending before any decision.
    fn require_pending(&self) -> Result<(), DomainError> {
        if self.status != GateStatus::Pending {
            return Err(DomainError::Conflict(format!(
                "gate is {:?}, expected Pending",
                self.status
            )));
        }
        Ok(())
    }

    /// Record a human decision on a pending gate.
    fn decide(
        &mut self,
        status: GateStatus,
        decided_by: String,
        rationale: String,
    ) -> Result<(), DomainError> {
        self.require_pending()?;
        self.status = status;
        self.decided_by = Some(decided_by);
        self.decided_at = Some(Utc::now());
        self.decision_rationale = Some(rationale);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Approve a pending gate, recording who decided and why.
    pub fn approve(&mut self, decided_by: String, rationale: String) -> Result<(), DomainError> {
        self.decide(GateStatus::Approved, decided_by, rationale)
    }

    /// Deny a pending gate, recording who decided and why.
    pub fn deny(&mut self, decided_by: String, rationale: String) -> Result<(), DomainError> {
        self.decide(GateStatus::Denied, decided_by, rationale)
    }

    /// Mark a pending gate as timed out (no human decision).
    pub fn time_out(&mut self) -> Result<(), DomainError> {
        self.require_pending()?;
        self.status = GateStatus::TimedOut;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Returns `true` if the gate is still awaiting a decision.
    pub fn is_pending(&self) -> bool {
        self.status == GateStatus::Pending
    }

    /// Returns `true` if the gate has been resolved (Approved, Denied, or TimedOut).
    pub fn is_resolved(&self) -> bool {
        matches!(
            self.status,
            GateStatus::Approved | GateStatus::Denied | GateStatus::TimedOut
        )
    }
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

    // ── New behavioral tests ──

    #[test]
    fn gate_approve_sets_fields() {
        let now = Utc::now();
        let mut gate = Gate {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            gate_definition_id: None,
            status: GateStatus::Pending,
            reason: "Needs review".into(),
            context: serde_json::json!({}),
            decided_by: None,
            decided_at: None,
            decision_rationale: None,
            expires_at: None,
            created_at: now,
            updated_at: now,
        };
        gate.approve("user_123".into(), "Looks good".into())
            .unwrap();
        assert_eq!(gate.status, GateStatus::Approved);
        assert_eq!(gate.decided_by, Some("user_123".into()));
        assert!(gate.decided_at.is_some());
        assert_eq!(gate.decision_rationale, Some("Looks good".into()));
        assert!(gate.updated_at >= now);
    }

    #[test]
    fn gate_deny_sets_fields() {
        let mut gate = Gate {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            gate_definition_id: None,
            status: GateStatus::Pending,
            reason: "Needs review".into(),
            context: serde_json::json!({}),
            decided_by: None,
            decided_at: None,
            decision_rationale: None,
            expires_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        gate.deny("user_456".into(), "Too risky".into()).unwrap();
        assert_eq!(gate.status, GateStatus::Denied);
        assert_eq!(gate.decided_by, Some("user_456".into()));
        assert_eq!(gate.decision_rationale, Some("Too risky".into()));
    }

    #[test]
    fn gate_time_out_from_pending() {
        let mut gate = Gate {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            gate_definition_id: None,
            status: GateStatus::Pending,
            reason: "Needs review".into(),
            context: serde_json::json!({}),
            decided_by: None,
            decided_at: None,
            decision_rationale: None,
            expires_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        gate.time_out().unwrap();
        assert_eq!(gate.status, GateStatus::TimedOut);
        assert!(gate.decided_by.is_none()); // No human decided
    }

    #[test]
    fn gate_approve_rejects_non_pending() {
        let mut gate = Gate {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            gate_definition_id: None,
            status: GateStatus::Denied,
            reason: "Was denied".into(),
            context: serde_json::json!({}),
            decided_by: Some("user".into()),
            decided_at: Some(Utc::now()),
            decision_rationale: Some("No".into()),
            expires_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let err = gate
            .approve("user_2".into(), "Override".into())
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn gate_is_pending_and_is_resolved() {
        let pending = Gate {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            gate_definition_id: None,
            status: GateStatus::Pending,
            reason: "Review".into(),
            context: serde_json::json!({}),
            decided_by: None,
            decided_at: None,
            decision_rationale: None,
            expires_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(pending.is_pending());
        assert!(!pending.is_resolved());

        let mut approved = pending.clone();
        approved.approve("user".into(), "ok".into()).unwrap();
        assert!(!approved.is_pending());
        assert!(approved.is_resolved());
    }
}

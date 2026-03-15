use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Cancelled,
    Gated,
    TimedOut,
}

impl SessionStatus {
    pub fn can_transition_to(self, target: SessionStatus) -> bool {
        matches!(
            (self, target),
            (SessionStatus::Pending, SessionStatus::Active)
                | (SessionStatus::Active, SessionStatus::Completed)
                | (SessionStatus::Active, SessionStatus::Failed)
                | (SessionStatus::Active, SessionStatus::Cancelled)
                | (SessionStatus::Active, SessionStatus::Gated)
                | (SessionStatus::Gated, SessionStatus::Active)
                | (SessionStatus::Gated, SessionStatus::Cancelled)
                | (SessionStatus::Gated, SessionStatus::TimedOut)
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionIntent {
    Implement,
    Review,
    Assess,
    Analyze,
    Author,
    Execute,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionTargetType {
    Specification,
    Task,
    Journey,
    Step,
    Snapshot,
    Repository,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Session {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub agent_id: Uuid,
    pub status: SessionStatus,
    pub intent: SessionIntent,
    pub target_type: Option<SessionTargetType>,
    pub target_id: Option<Uuid>,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct SessionDef {
    id: Uuid,
    workspace_id: Uuid,
    agent_id: Uuid,
    status: SessionStatus,
    intent: SessionIntent,
    target_type: Option<SessionTargetType>,
    target_id: Option<Uuid>,
    config: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Session {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        agent_id: Uuid,
        status: SessionStatus,
        intent: SessionIntent,
        target_type: Option<SessionTargetType>,
        target_id: Option<Uuid>,
        config: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if target_type.is_some() != target_id.is_some() {
            return Err(DomainError::Validation(
                "target_type and target_id must both be set or both be None".into(),
            ));
        }

        Ok(Self {
            id,
            workspace_id,
            agent_id,
            status,
            intent,
            target_type,
            target_id,
            config,
            created_at,
            updated_at,
        })
    }

    /// Validate and apply a status transition, updating `updated_at`.
    pub fn transition_to(&mut self, target: SessionStatus) -> Result<(), DomainError> {
        if !self.status.can_transition_to(target) {
            return Err(DomainError::Conflict(format!(
                "cannot transition session from {:?} to {:?}",
                self.status, target
            )));
        }
        self.status = target;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Pending -> Active.
    pub fn start(&mut self) -> Result<(), DomainError> {
        self.transition_to(SessionStatus::Active)
    }

    /// Active -> Completed.
    pub fn complete(&mut self) -> Result<(), DomainError> {
        self.transition_to(SessionStatus::Completed)
    }

    /// Active -> Failed.
    pub fn fail(&mut self) -> Result<(), DomainError> {
        self.transition_to(SessionStatus::Failed)
    }

    /// Active | Gated -> Cancelled.
    pub fn cancel(&mut self) -> Result<(), DomainError> {
        self.transition_to(SessionStatus::Cancelled)
    }

    /// Active -> Gated.
    pub fn gate(&mut self) -> Result<(), DomainError> {
        self.transition_to(SessionStatus::Gated)
    }

    /// Gated -> Active.
    pub fn resume(&mut self) -> Result<(), DomainError> {
        self.transition_to(SessionStatus::Active)
    }
}

impl<'de> Deserialize<'de> for Session {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = SessionDef::deserialize(deserializer)?;
        Session::new(
            raw.id,
            raw.workspace_id,
            raw.agent_id,
            raw.status,
            raw.intent,
            raw.target_type,
            raw.target_id,
            raw.config,
            raw.created_at,
            raw.updated_at,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn session_status_serializes_to_snake_case() {
        let json = serde_json::to_string(&SessionStatus::TimedOut).unwrap();
        assert_eq!(json, r#""timed_out""#);
    }

    #[test]
    fn session_intent_serializes_to_snake_case() {
        let json = serde_json::to_string(&SessionIntent::Implement).unwrap();
        assert_eq!(json, r#""implement""#);
    }

    #[test]
    fn valid_transitions() {
        // Pending -> Active
        assert!(SessionStatus::Pending.can_transition_to(SessionStatus::Active));
        // Active -> Completed, Failed, Cancelled, Gated
        assert!(SessionStatus::Active.can_transition_to(SessionStatus::Completed));
        assert!(SessionStatus::Active.can_transition_to(SessionStatus::Failed));
        assert!(SessionStatus::Active.can_transition_to(SessionStatus::Cancelled));
        assert!(SessionStatus::Active.can_transition_to(SessionStatus::Gated));
        // Gated -> Active, Cancelled, TimedOut
        assert!(SessionStatus::Gated.can_transition_to(SessionStatus::Active));
        assert!(SessionStatus::Gated.can_transition_to(SessionStatus::Cancelled));
        assert!(SessionStatus::Gated.can_transition_to(SessionStatus::TimedOut));
    }

    #[test]
    fn invalid_transitions() {
        assert!(!SessionStatus::Pending.can_transition_to(SessionStatus::Completed));
        assert!(!SessionStatus::Pending.can_transition_to(SessionStatus::Gated));
        assert!(!SessionStatus::Active.can_transition_to(SessionStatus::Pending));
        assert!(!SessionStatus::Active.can_transition_to(SessionStatus::TimedOut));
        assert!(!SessionStatus::Gated.can_transition_to(SessionStatus::Completed));
        assert!(!SessionStatus::Gated.can_transition_to(SessionStatus::Failed));
    }

    #[test]
    fn terminal_states_have_no_transitions() {
        let terminal = [
            SessionStatus::Completed,
            SessionStatus::Failed,
            SessionStatus::Cancelled,
            SessionStatus::TimedOut,
        ];
        let all = [
            SessionStatus::Pending,
            SessionStatus::Active,
            SessionStatus::Completed,
            SessionStatus::Failed,
            SessionStatus::Cancelled,
            SessionStatus::Gated,
            SessionStatus::TimedOut,
        ];
        for t in &terminal {
            for target in &all {
                assert!(
                    !t.can_transition_to(*target),
                    "{t:?} should not transition to {target:?}"
                );
            }
        }
    }

    #[test]
    fn session_serde_roundtrip() {
        let now = Utc::now();
        let session = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Pending,
            SessionIntent::Implement,
            Some(SessionTargetType::Specification),
            Some(Uuid::now_v7()),
            None,
            now,
            now,
        )
        .unwrap();
        let json = serde_json::to_string(&session).unwrap();
        let back: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(session.id, back.id);
        assert_eq!(session.status, back.status);
        assert_eq!(session.intent, back.intent);
        assert_eq!(session.target_type, back.target_type);
        assert_eq!(session.target_id, back.target_id);
    }

    #[test]
    fn session_requires_target_type_and_target_id_to_appear_together() {
        let now = Utc::now();
        // Both None — valid
        let result = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Pending,
            SessionIntent::Analyze,
            None,
            None,
            None,
            now,
            now,
        );
        assert!(result.is_ok());

        // Both Some — valid
        let result = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Pending,
            SessionIntent::Analyze,
            Some(SessionTargetType::Task),
            Some(Uuid::now_v7()),
            None,
            now,
            now,
        );
        assert!(result.is_ok());

        // target_type without target_id — invalid
        let err = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Pending,
            SessionIntent::Analyze,
            Some(SessionTargetType::Task),
            None,
            None,
            now,
            now,
        )
        .unwrap_err();
        assert_eq!(
            err,
            DomainError::Validation(
                "target_type and target_id must both be set or both be None".into()
            )
        );
    }

    #[test]
    fn session_deserialize_rejects_target_id_without_target_type() {
        let json = format!(
            concat!(
                r#"{{"id":"{}","workspace_id":"{}","agent_id":"{}","#,
                r#""status":"pending","intent":"implement","#,
                r#""target_type":null,"target_id":"{}","#,
                r#""config":null,"#,
                r#""created_at":"{}","updated_at":"{}"}}"#
            ),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339()
        );
        let err = serde_json::from_str::<Session>(&json).unwrap_err();
        assert!(
            err.to_string()
                .contains("target_type and target_id must both be set or both be None")
        );
    }

    // ── New behavioral tests ──

    #[test]
    fn session_start_transitions_pending_to_active() {
        let now = Utc::now();
        let mut session = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Pending,
            SessionIntent::Implement,
            None,
            None,
            None,
            now,
            now,
        )
        .unwrap();
        assert!(session.start().is_ok());
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.updated_at >= now);
    }

    #[test]
    fn session_start_rejects_non_pending() {
        let now = Utc::now();
        let mut session = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Active,
            SessionIntent::Implement,
            None,
            None,
            None,
            now,
            now,
        )
        .unwrap();
        let err = session.start().unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn session_complete_from_active() {
        let now = Utc::now();
        let mut session = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Active,
            SessionIntent::Review,
            None,
            None,
            None,
            now,
            now,
        )
        .unwrap();
        assert!(session.complete().is_ok());
        assert_eq!(session.status, SessionStatus::Completed);
    }

    #[test]
    fn session_fail_from_active() {
        let now = Utc::now();
        let mut session = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Active,
            SessionIntent::Implement,
            None,
            None,
            None,
            now,
            now,
        )
        .unwrap();
        assert!(session.fail().is_ok());
        assert_eq!(session.status, SessionStatus::Failed);
    }

    #[test]
    fn session_gate_and_resume_lifecycle() {
        let now = Utc::now();
        let mut session = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Active,
            SessionIntent::Implement,
            None,
            None,
            None,
            now,
            now,
        )
        .unwrap();
        // Active -> Gated
        assert!(session.gate().is_ok());
        assert_eq!(session.status, SessionStatus::Gated);
        // Gated -> Active (resume)
        assert!(session.resume().is_ok());
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn session_cancel_from_active_or_gated() {
        let now = Utc::now();
        let mut s1 = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Active,
            SessionIntent::Implement,
            None,
            None,
            None,
            now,
            now,
        )
        .unwrap();
        assert!(s1.cancel().is_ok());
        assert_eq!(s1.status, SessionStatus::Cancelled);

        let mut s2 = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Gated,
            SessionIntent::Implement,
            None,
            None,
            None,
            now,
            now,
        )
        .unwrap();
        assert!(s2.cancel().is_ok());
        assert_eq!(s2.status, SessionStatus::Cancelled);
    }

    #[test]
    fn session_transition_to_rejects_invalid() {
        let now = Utc::now();
        let mut session = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            SessionStatus::Completed,
            SessionIntent::Implement,
            None,
            None,
            None,
            now,
            now,
        )
        .unwrap();
        let err = session.transition_to(SessionStatus::Active).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
        // Status should not have changed
        assert_eq!(session.status, SessionStatus::Completed);
    }
}

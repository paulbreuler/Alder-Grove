use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SessionTargetType {
    Specification,
    Task,
    Journey,
    Step,
    Snapshot,
    Repository,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Session {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub agent_id: Uuid,
    pub title: String,
    pub status: SessionStatus,
    pub intent: SessionIntent,
    pub target_type: Option<SessionTargetType>,
    pub target_id: Option<Uuid>,
    pub context: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub initiated_by: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct SessionDef {
    id: Uuid,
    workspace_id: Uuid,
    agent_id: Uuid,
    title: String,
    status: SessionStatus,
    intent: SessionIntent,
    target_type: Option<SessionTargetType>,
    target_id: Option<Uuid>,
    context: serde_json::Value,
    result: Option<serde_json::Value>,
    initiated_by: String,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Session {
    // Domain entity constructor — mirrors DB row fields directly.
    // A builder would add indirection without value at the domain layer.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        agent_id: Uuid,
        title: String,
        status: SessionStatus,
        intent: SessionIntent,
        target_type: Option<SessionTargetType>,
        target_id: Option<Uuid>,
        context: serde_json::Value,
        result: Option<serde_json::Value>,
        initiated_by: String,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
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
            title,
            status,
            intent,
            target_type,
            target_id,
            context,
            result,
            initiated_by,
            started_at,
            completed_at,
            created_at,
            updated_at,
        })
    }

    /// Validate and apply a status transition, updating `updated_at`.
    /// Sets `started_at` when transitioning to Active (if not already set).
    /// Sets `completed_at` when transitioning to a terminal state.
    pub fn transition_to(&mut self, target: SessionStatus) -> Result<(), DomainError> {
        if !self.status.can_transition_to(target) {
            return Err(DomainError::Conflict(format!(
                "cannot transition session from {:?} to {:?}",
                self.status, target
            )));
        }

        let now = Utc::now();

        if target == SessionStatus::Active && self.started_at.is_none() {
            self.started_at = Some(now);
        }

        if matches!(
            target,
            SessionStatus::Completed
                | SessionStatus::Failed
                | SessionStatus::Cancelled
                | SessionStatus::TimedOut
        ) {
            self.completed_at = Some(now);
        }

        self.status = target;
        self.updated_at = now;
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
            raw.title,
            raw.status,
            raw.intent,
            raw.target_type,
            raw.target_id,
            raw.context,
            raw.result,
            raw.initiated_by,
            raw.started_at,
            raw.completed_at,
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

    /// Helper to build a Session with sensible defaults, reducing boilerplate.
    fn make_session(status: SessionStatus, intent: SessionIntent) -> Session {
        let now = Utc::now();
        Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            "Test session".into(),
            status,
            intent,
            None,
            None,
            serde_json::json!({}),
            None,
            "user_test".into(),
            None,
            None,
            now,
            now,
        )
        .unwrap()
    }

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
            "Implement feature X".into(),
            SessionStatus::Pending,
            SessionIntent::Implement,
            Some(SessionTargetType::Specification),
            Some(Uuid::now_v7()),
            serde_json::json!({"key": "value"}),
            None,
            "user_abc".into(),
            None,
            None,
            now,
            now,
        )
        .unwrap();
        let json = serde_json::to_string(&session).unwrap();
        let back: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(session.id, back.id);
        assert_eq!(session.title, back.title);
        assert_eq!(session.status, back.status);
        assert_eq!(session.intent, back.intent);
        assert_eq!(session.target_type, back.target_type);
        assert_eq!(session.target_id, back.target_id);
        assert_eq!(session.context, back.context);
        assert_eq!(session.result, back.result);
        assert_eq!(session.initiated_by, back.initiated_by);
        assert_eq!(session.started_at, back.started_at);
        assert_eq!(session.completed_at, back.completed_at);
    }

    #[test]
    fn session_requires_target_type_and_target_id_to_appear_together() {
        let now = Utc::now();
        // Both None — valid
        let result = Session::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            "Test".into(),
            SessionStatus::Pending,
            SessionIntent::Analyze,
            None,
            None,
            serde_json::json!({}),
            None,
            "user_test".into(),
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
            "Test".into(),
            SessionStatus::Pending,
            SessionIntent::Analyze,
            Some(SessionTargetType::Task),
            Some(Uuid::now_v7()),
            serde_json::json!({}),
            None,
            "user_test".into(),
            None,
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
            "Test".into(),
            SessionStatus::Pending,
            SessionIntent::Analyze,
            Some(SessionTargetType::Task),
            None,
            serde_json::json!({}),
            None,
            "user_test".into(),
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
                r#""title":"Test","#,
                r#""status":"pending","intent":"implement","#,
                r#""target_type":null,"target_id":"{}","#,
                r#""context":{{}},"result":null,"#,
                r#""initiated_by":"user_test","#,
                r#""started_at":null,"completed_at":null,"#,
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

    // ── Behavioral tests ──

    #[test]
    fn session_start_transitions_pending_to_active() {
        let mut session = make_session(SessionStatus::Pending, SessionIntent::Implement);
        let before = session.updated_at;
        assert!(session.start().is_ok());
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.started_at.is_some());
        assert!(session.updated_at >= before);
    }

    #[test]
    fn session_start_rejects_non_pending() {
        let mut session = make_session(SessionStatus::Active, SessionIntent::Implement);
        let err = session.start().unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn session_complete_from_active() {
        let mut session = make_session(SessionStatus::Active, SessionIntent::Review);
        assert!(session.complete().is_ok());
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn session_fail_from_active() {
        let mut session = make_session(SessionStatus::Active, SessionIntent::Implement);
        assert!(session.fail().is_ok());
        assert_eq!(session.status, SessionStatus::Failed);
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn session_gate_and_resume_lifecycle() {
        let mut session = make_session(SessionStatus::Active, SessionIntent::Implement);
        // Active -> Gated
        assert!(session.gate().is_ok());
        assert_eq!(session.status, SessionStatus::Gated);
        // Gated -> Active (resume) — started_at should not be overwritten
        assert!(session.resume().is_ok());
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn session_cancel_from_active_or_gated() {
        let mut s1 = make_session(SessionStatus::Active, SessionIntent::Implement);
        assert!(s1.cancel().is_ok());
        assert_eq!(s1.status, SessionStatus::Cancelled);
        assert!(s1.completed_at.is_some());

        let mut s2 = make_session(SessionStatus::Gated, SessionIntent::Implement);
        assert!(s2.cancel().is_ok());
        assert_eq!(s2.status, SessionStatus::Cancelled);
        assert!(s2.completed_at.is_some());
    }

    #[test]
    fn session_transition_to_rejects_invalid() {
        let mut session = make_session(SessionStatus::Completed, SessionIntent::Implement);
        let err = session.transition_to(SessionStatus::Active).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
        // Status should not have changed
        assert_eq!(session.status, SessionStatus::Completed);
    }

    // ── New timestamp behavior tests ──

    #[test]
    fn transition_to_active_sets_started_at() {
        let mut session = make_session(SessionStatus::Pending, SessionIntent::Execute);
        assert!(session.started_at.is_none());
        session.start().unwrap();
        assert!(session.started_at.is_some());
    }

    #[test]
    fn resume_from_gated_does_not_overwrite_started_at() {
        let mut session = make_session(SessionStatus::Pending, SessionIntent::Execute);
        session.start().unwrap();
        let original_started = session.started_at;
        session.gate().unwrap();
        session.resume().unwrap();
        assert_eq!(session.started_at, original_started);
    }

    #[test]
    fn terminal_transitions_set_completed_at() {
        // Completed
        let mut s1 = make_session(SessionStatus::Active, SessionIntent::Implement);
        assert!(s1.completed_at.is_none());
        s1.complete().unwrap();
        assert!(s1.completed_at.is_some());

        // Failed
        let mut s2 = make_session(SessionStatus::Active, SessionIntent::Implement);
        s2.fail().unwrap();
        assert!(s2.completed_at.is_some());

        // Cancelled
        let mut s3 = make_session(SessionStatus::Active, SessionIntent::Implement);
        s3.cancel().unwrap();
        assert!(s3.completed_at.is_some());

        // TimedOut (from Gated)
        let mut s4 = make_session(SessionStatus::Gated, SessionIntent::Implement);
        s4.transition_to(SessionStatus::TimedOut).unwrap();
        assert!(s4.completed_at.is_some());
    }
}

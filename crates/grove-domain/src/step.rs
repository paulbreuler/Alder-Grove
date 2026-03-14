use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::AiProvenance;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Step {
    pub id: Uuid,
    pub journey_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: StepStatus,
    pub persona_id: Option<Uuid>,
    pub percent_complete: f32,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::AiProvenance;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn step_status_in_progress_serializes_to_snake_case() {
        let json = serde_json::to_string(&StepStatus::InProgress).unwrap();
        assert_eq!(json, r#""in_progress""#);
    }

    #[test]
    fn step_serde_roundtrip() {
        let now = Utc::now();
        let step = Step {
            id: Uuid::now_v7(),
            journey_id: Uuid::now_v7(),
            name: "Sign up".into(),
            description: Some("User creates an account".into()),
            sort_order: 1,
            status: StepStatus::Pending,
            persona_id: Some(Uuid::now_v7()),
            percent_complete: 0.0,
            ai: AiProvenance::default(),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&step).unwrap();
        let back: Step = serde_json::from_str(&json).unwrap();
        assert_eq!(step.id, back.id);
        assert_eq!(step.sort_order, back.sort_order);
        assert_eq!(step.status, back.status);
        assert_eq!(step.percent_complete, back.percent_complete);
    }
}

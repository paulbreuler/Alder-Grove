use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::AiProvenance;
use crate::error::DomainError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Step {
    pub id: Uuid,
    pub journey_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: StepStatus,
    pub persona_id: Option<Uuid>,
    percent_complete: f32,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct StepDef {
    id: Uuid,
    journey_id: Uuid,
    name: String,
    description: Option<String>,
    sort_order: i32,
    status: StepStatus,
    persona_id: Option<Uuid>,
    percent_complete: f32,
    #[serde(flatten)]
    ai: AiProvenance,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Step {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        journey_id: Uuid,
        name: String,
        description: Option<String>,
        sort_order: i32,
        status: StepStatus,
        persona_id: Option<Uuid>,
        percent_complete: f32,
        ai: AiProvenance,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if !(0.0..=1.0).contains(&percent_complete) {
            return Err(DomainError::Validation(
                "percent_complete must be between 0.0 and 1.0".into(),
            ));
        }

        Ok(Self {
            id,
            journey_id,
            name,
            description,
            sort_order,
            status,
            persona_id,
            percent_complete,
            ai,
            created_at,
            updated_at,
        })
    }

    pub fn percent_complete(&self) -> f32 {
        self.percent_complete
    }
}

impl<'de> Deserialize<'de> for Step {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = StepDef::deserialize(deserializer)?;
        Step::new(
            raw.id,
            raw.journey_id,
            raw.name,
            raw.description,
            raw.sort_order,
            raw.status,
            raw.persona_id,
            raw.percent_complete,
            raw.ai,
            raw.created_at,
            raw.updated_at,
        )
        .map_err(serde::de::Error::custom)
    }
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
        let step = Step::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            "Sign up".into(),
            Some("User creates an account".into()),
            1,
            StepStatus::Pending,
            Some(Uuid::now_v7()),
            0.0,
            AiProvenance::default(),
            now,
            now,
        )
        .unwrap();
        let json = serde_json::to_string(&step).unwrap();
        let back: Step = serde_json::from_str(&json).unwrap();
        assert_eq!(step.id, back.id);
        assert_eq!(step.sort_order, back.sort_order);
        assert_eq!(step.status, back.status);
        assert_eq!(step.percent_complete(), back.percent_complete());
    }

    #[test]
    fn step_rejects_invalid_percent_complete() {
        let err = Step::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            "Sign up".into(),
            None,
            1,
            StepStatus::Pending,
            None,
            1.2,
            AiProvenance::default(),
            Utc::now(),
            Utc::now(),
        )
        .unwrap_err();
        assert_eq!(
            err,
            DomainError::Validation("percent_complete must be between 0.0 and 1.0".into())
        );
    }

    #[test]
    fn step_deserialize_rejects_invalid_percent_complete() {
        let json = format!(
            concat!(
                r#"{{"id":"{}","journey_id":"{}","name":"Sign up","description":null,"#,
                r#""sort_order":1,"status":"pending","persona_id":null,"percent_complete":1.1,"#,
                r#""created_at":"{}","updated_at":"{}"}}"#
            ),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339()
        );
        let err = serde_json::from_str::<Step>(&json).unwrap_err();
        assert!(
            err.to_string()
                .contains("percent_complete must be between 0.0 and 1.0")
        );
    }
}

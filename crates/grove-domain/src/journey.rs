use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::AiProvenance;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum JourneyStatus {
    Draft,
    Active,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Journey {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: JourneyStatus,
    pub persona_id: Option<Uuid>,
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
    fn journey_status_serializes_to_snake_case() {
        let json = serde_json::to_string(&JourneyStatus::Active).unwrap();
        assert_eq!(json, r#""active""#);

        let json = serde_json::to_string(&JourneyStatus::Draft).unwrap();
        assert_eq!(json, r#""draft""#);

        let json = serde_json::to_string(&JourneyStatus::Completed).unwrap();
        assert_eq!(json, r#""completed""#);

        let json = serde_json::to_string(&JourneyStatus::Archived).unwrap();
        assert_eq!(json, r#""archived""#);
    }

    #[test]
    fn journey_serde_roundtrip() {
        let now = Utc::now();
        let journey = Journey {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Onboarding Flow".into(),
            description: Some("New user onboarding".into()),
            status: JourneyStatus::Active,
            persona_id: Some(Uuid::now_v7()),
            ai: AiProvenance::default(),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&journey).unwrap();
        let back: Journey = serde_json::from_str(&json).unwrap();
        assert_eq!(journey.id, back.id);
        assert_eq!(journey.status, back.status);
        assert_eq!(journey.persona_id, back.persona_id);
    }
}

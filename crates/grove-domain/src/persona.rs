use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::AiProvenance;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Persona {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub goals: Option<String>,
    pub pain_points: Option<String>,
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
    fn persona_serde_roundtrip() {
        let now = Utc::now();
        let persona = Persona {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Developer".into(),
            description: Some("A software developer".into()),
            goals: Some("Ship features fast".into()),
            pain_points: Some("Too many meetings".into()),
            ai: AiProvenance {
                ai_authored: true,
                ai_confidence: Some(0.85),
                ai_rationale: Some("Inferred from interviews".into()),
            },
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&persona).unwrap();
        let back: Persona = serde_json::from_str(&json).unwrap();
        assert_eq!(persona.id, back.id);
        assert_eq!(persona.name, back.name);
        assert_eq!(persona.ai.ai_authored, back.ai.ai_authored);
    }

    #[test]
    fn persona_ai_fields_flattened_at_top_level() {
        let now = Utc::now();
        let persona = Persona {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Designer".into(),
            description: None,
            goals: None,
            pain_points: None,
            ai: AiProvenance {
                ai_authored: true,
                ai_confidence: Some(0.9),
                ai_rationale: None,
            },
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&persona).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        // ai_authored should be at top level, not nested under "ai"
        assert_eq!(val["ai_authored"], serde_json::Value::Bool(true));
        assert!(val.get("ai").is_none());
    }
}

use serde::{Deserialize, Serialize};

/// AI authorship metadata embedded in content entities.
/// Flattened into parent structs via `#[serde(flatten)]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AiProvenance {
    pub ai_authored: bool,
    pub ai_confidence: Option<f32>,
    pub ai_rationale: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_provenance_defaults_to_not_authored() {
        let prov = AiProvenance::default();
        assert!(!prov.ai_authored);
        assert!(prov.ai_confidence.is_none());
        assert!(prov.ai_rationale.is_none());
    }

    #[test]
    fn ai_provenance_defaults_when_fields_missing() {
        // Verifies #[serde(default)] — unknown fields ignored, missing AI fields get defaults
        let json = r#"{}"#;
        let prov: AiProvenance = serde_json::from_str(json).unwrap();
        assert!(!prov.ai_authored);
        assert!(prov.ai_confidence.is_none());
        assert!(prov.ai_rationale.is_none());
    }

    #[test]
    fn ai_provenance_flatten_ignores_sibling_fields() {
        // Simulates #[serde(flatten)] context where parent fields leak into AiProvenance
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Parent {
            title: String,
            #[serde(flatten)]
            ai: AiProvenance,
        }

        let json = r#"{"title":"hello","ai_authored":true,"ai_confidence":0.9}"#;
        let parent: Parent = serde_json::from_str(json).unwrap();
        assert_eq!(parent.title, "hello");
        assert!(parent.ai.ai_authored);
        assert_eq!(parent.ai.ai_confidence, Some(0.9));
    }

    #[test]
    fn ai_provenance_serde_roundtrip() {
        let prov = AiProvenance {
            ai_authored: true,
            ai_confidence: Some(0.95),
            ai_rationale: Some("Generated from user story".into()),
        };
        let json = serde_json::to_string(&prov).unwrap();
        let back: AiProvenance = serde_json::from_str(&json).unwrap();
        assert_eq!(prov.ai_authored, back.ai_authored);
        assert_eq!(prov.ai_confidence, back.ai_confidence);
        assert_eq!(prov.ai_rationale, back.ai_rationale);
    }
}

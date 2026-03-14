use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// AI authorship metadata embedded in content entities.
/// Flattened into parent structs via `#[serde(flatten)]`.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct AiProvenance {
    ai_authored: bool,
    ai_confidence: Option<f32>,
    ai_rationale: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct AiProvenanceDef {
    ai_authored: bool,
    ai_confidence: Option<f32>,
    ai_rationale: Option<String>,
}

impl AiProvenance {
    pub fn new(
        ai_authored: bool,
        ai_confidence: Option<f32>,
        ai_rationale: Option<String>,
    ) -> Result<Self, DomainError> {
        if let Some(confidence) = ai_confidence {
            if !(0.0..=1.0).contains(&confidence) {
                return Err(DomainError::Validation(
                    "ai_confidence must be between 0.0 and 1.0".into(),
                ));
            }
        }

        Ok(Self {
            ai_authored,
            ai_confidence,
            ai_rationale,
        })
    }

    pub fn ai_authored(&self) -> bool {
        self.ai_authored
    }

    pub fn ai_confidence(&self) -> Option<f32> {
        self.ai_confidence
    }

    pub fn ai_rationale(&self) -> Option<&str> {
        self.ai_rationale.as_deref()
    }
}

impl<'de> Deserialize<'de> for AiProvenance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = AiProvenanceDef::deserialize(deserializer)?;
        Self::new(raw.ai_authored, raw.ai_confidence, raw.ai_rationale)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_provenance_defaults_to_not_authored() {
        let prov = AiProvenance::default();
        assert!(!prov.ai_authored());
        assert!(prov.ai_confidence().is_none());
        assert!(prov.ai_rationale().is_none());
    }

    #[test]
    fn ai_provenance_defaults_when_fields_missing() {
        // Verifies #[serde(default)] — unknown fields ignored, missing AI fields get defaults
        let json = r#"{}"#;
        let prov: AiProvenance = serde_json::from_str(json).unwrap();
        assert!(!prov.ai_authored());
        assert!(prov.ai_confidence().is_none());
        assert!(prov.ai_rationale().is_none());
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
        assert!(parent.ai.ai_authored());
        assert_eq!(parent.ai.ai_confidence(), Some(0.9));
    }

    #[test]
    fn ai_provenance_serde_roundtrip() {
        let prov =
            AiProvenance::new(true, Some(0.95), Some("Generated from user story".into())).unwrap();
        let json = serde_json::to_string(&prov).unwrap();
        let back: AiProvenance = serde_json::from_str(&json).unwrap();
        assert_eq!(prov.ai_authored(), back.ai_authored());
        assert_eq!(prov.ai_confidence(), back.ai_confidence());
        assert_eq!(prov.ai_rationale(), back.ai_rationale());
    }

    #[test]
    fn ai_provenance_rejects_invalid_confidence() {
        let err = AiProvenance::new(true, Some(1.5), None).unwrap_err();
        assert_eq!(
            err,
            DomainError::Validation("ai_confidence must be between 0.0 and 1.0".into())
        );
    }

    #[test]
    fn ai_provenance_deserialize_rejects_invalid_confidence() {
        let err = serde_json::from_str::<AiProvenance>(r#"{"ai_confidence":-0.1}"#).unwrap_err();
        assert!(
            err.to_string()
                .contains("ai_confidence must be between 0.0 and 1.0")
        );
    }
}

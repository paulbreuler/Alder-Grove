use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::AiProvenance;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SpecificationStatus {
    Draft,
    Ready,
    InProgress,
    Done,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct RequirementItem {
    pub description: String,
    pub met: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SpecificationRequirements {
    pub functional: Vec<RequirementItem>,
    pub non_functional: Vec<RequirementItem>,
    pub acceptance: Vec<RequirementItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Dependency {
    pub specification_id: Uuid,
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ErrorHandlingStrategy {
    pub scenario: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct TestingStrategy {
    pub unit: String,
    pub integration: String,
    pub e2e: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ComponentSpec {
    pub path: String,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Specification {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub status: SpecificationStatus,
    pub requirements: SpecificationRequirements,
    pub dependencies: Vec<Dependency>,
    pub error_handling: Vec<ErrorHandlingStrategy>,
    pub testing_strategy: Option<TestingStrategy>,
    pub components: Vec<ComponentSpec>,
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
    fn specification_status_in_progress_serializes_to_snake_case() {
        let json = serde_json::to_string(&SpecificationStatus::InProgress).unwrap();
        assert_eq!(json, r#""in_progress""#);
    }

    #[test]
    fn requirement_item_serde_roundtrip() {
        let item = RequirementItem {
            description: "Must support SSO".into(),
            met: false,
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: RequirementItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item.description, back.description);
        assert_eq!(item.met, back.met);
    }

    #[test]
    fn specification_requirements_default_has_empty_vecs() {
        let reqs = SpecificationRequirements::default();
        assert!(reqs.functional.is_empty());
        assert!(reqs.non_functional.is_empty());
        assert!(reqs.acceptance.is_empty());
    }

    #[test]
    fn specification_full_roundtrip() {
        let now = Utc::now();
        let spec = Specification {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            title: "User Auth".into(),
            description: Some("Authentication feature".into()),
            scope: Some("Backend API".into()),
            status: SpecificationStatus::Ready,
            requirements: SpecificationRequirements {
                functional: vec![RequirementItem {
                    description: "Login with email".into(),
                    met: true,
                }],
                non_functional: vec![RequirementItem {
                    description: "Response under 200ms".into(),
                    met: false,
                }],
                acceptance: vec![RequirementItem {
                    description: "User can reset password".into(),
                    met: false,
                }],
            },
            dependencies: vec![Dependency {
                specification_id: Uuid::now_v7(),
                relationship: "blocks".into(),
            }],
            error_handling: vec![ErrorHandlingStrategy {
                scenario: "Invalid credentials".into(),
                response: "Return 401".into(),
            }],
            testing_strategy: Some(TestingStrategy {
                unit: "Test auth service".into(),
                integration: "Test API endpoints".into(),
                e2e: "Test login flow".into(),
            }),
            components: vec![ComponentSpec {
                path: "src/auth/mod.rs".into(),
                action: "create".into(),
                description: "Auth module entry".into(),
            }],
            ai: AiProvenance::new(true, Some(0.92), Some("Generated from user story".into()))
                .unwrap(),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let back: Specification = serde_json::from_str(&json).unwrap();
        assert_eq!(spec.id, back.id);
        assert_eq!(spec.title, back.title);
        assert_eq!(spec.status, back.status);
        assert_eq!(back.requirements.functional.len(), 1);
        assert_eq!(back.dependencies.len(), 1);
        assert_eq!(back.error_handling.len(), 1);
        assert!(back.testing_strategy.is_some());
        assert_eq!(back.components.len(), 1);
        assert_eq!(back.ai.ai_authored(), spec.ai.ai_authored());
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailCategory {
    Prohibition,
    Requirement,
    Boundary,
    Preference,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailScope {
    Workspace,
    Session,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailEnforcement {
    Enforced,
    Advisory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuardrailRule {
    Prohibition {
        description: String,
        patterns: Vec<String>,
        actions: Vec<String>,
    },
    Requirement {
        description: String,
        check: String,
        params: serde_json::Value,
    },
    Preference {
        description: String,
        context: String,
        guidance: String,
    },
    Boundary {
        description: String,
        allowed_paths: Vec<String>,
        denied_paths: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuardrailVerdict {
    /// Action is allowed — no rule matched
    Allowed,
    /// Action is blocked — enforced guardrail matched
    Denied {
        guardrail_name: String,
        reason: String,
    },
    /// Action is flagged — advisory guardrail matched
    Advisory {
        guardrail_name: String,
        guidance: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guardrail {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: GuardrailCategory,
    pub scope: GuardrailScope,
    pub enforcement: GuardrailEnforcement,
    pub rule: GuardrailRule,
    pub version: i32,
    pub sort_order: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Guardrail {
    /// Check if an action (identified by type and path) violates this guardrail.
    /// Returns `Allowed` if disabled, not applicable, or no rule matches.
    /// Returns `Denied` or `Advisory` based on enforcement level.
    pub fn evaluate_action(&self, action_type: &str, path: &str) -> GuardrailVerdict {
        if !self.enabled {
            return GuardrailVerdict::Allowed;
        }

        let violation_description = match &self.rule {
            GuardrailRule::Prohibition {
                description,
                patterns,
                actions,
            } => {
                let action_matches = actions.iter().any(|a| a == action_type);
                let path_matches = patterns.iter().any(|p| path.contains(p));
                if action_matches && path_matches {
                    Some(description.clone())
                } else {
                    None
                }
            }
            GuardrailRule::Boundary {
                description,
                allowed_paths,
                denied_paths,
            } => {
                let in_denied = denied_paths.iter().any(|d| path.starts_with(d));
                if in_denied {
                    return self.verdict(description);
                }
                let outside_allowed =
                    !allowed_paths.is_empty() && !allowed_paths.iter().any(|a| path.starts_with(a));
                if outside_allowed {
                    Some(description.clone())
                } else {
                    None
                }
            }
            // Requirement and Preference cannot be evaluated against a single action
            GuardrailRule::Requirement { .. } | GuardrailRule::Preference { .. } => None,
        };

        match violation_description {
            Some(desc) => self.verdict(&desc),
            None => GuardrailVerdict::Allowed,
        }
    }

    /// Whether this guardrail is currently active.
    pub fn is_active(&self) -> bool {
        self.enabled
    }

    /// Build the appropriate verdict variant based on enforcement level.
    fn verdict(&self, description: &str) -> GuardrailVerdict {
        match self.enforcement {
            GuardrailEnforcement::Enforced => GuardrailVerdict::Denied {
                guardrail_name: self.name.clone(),
                reason: description.to_owned(),
            },
            GuardrailEnforcement::Advisory => GuardrailVerdict::Advisory {
                guardrail_name: self.name.clone(),
                guidance: description.to_owned(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn evaluate_prohibition_blocks_matching_action() {
        let g = Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "No migration edits".into(),
            description: None,
            category: GuardrailCategory::Prohibition,
            scope: GuardrailScope::Workspace,
            enforcement: GuardrailEnforcement::Enforced,
            rule: GuardrailRule::Prohibition {
                description: "Do not modify migration files".into(),
                patterns: vec!["migrations/".into()],
                actions: vec!["file_modify".into(), "file_delete".into()],
            },
            version: 1,
            sort_order: 0,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let result = g.evaluate_action("file_modify", "crates/api/migrations/001.sql");
        assert!(matches!(result, GuardrailVerdict::Denied { .. }));
    }

    #[test]
    fn evaluate_prohibition_allows_non_matching_action() {
        let g = Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "No migration edits".into(),
            description: None,
            category: GuardrailCategory::Prohibition,
            scope: GuardrailScope::Workspace,
            enforcement: GuardrailEnforcement::Enforced,
            rule: GuardrailRule::Prohibition {
                description: "Do not modify migration files".into(),
                patterns: vec!["migrations/".into()],
                actions: vec!["file_modify".into()],
            },
            version: 1,
            sort_order: 0,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        // Different action type
        let result = g.evaluate_action("file_read", "crates/api/migrations/001.sql");
        assert_eq!(result, GuardrailVerdict::Allowed);
        // Different path
        let result = g.evaluate_action("file_modify", "src/main.rs");
        assert_eq!(result, GuardrailVerdict::Allowed);
    }

    #[test]
    fn evaluate_advisory_prohibition_returns_advisory() {
        let g = Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Prefer tests".into(),
            description: None,
            category: GuardrailCategory::Prohibition,
            scope: GuardrailScope::Workspace,
            enforcement: GuardrailEnforcement::Advisory,
            rule: GuardrailRule::Prohibition {
                description: "Avoid modifying code without tests".into(),
                patterns: vec!["src/".into()],
                actions: vec!["file_modify".into()],
            },
            version: 1,
            sort_order: 0,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let result = g.evaluate_action("file_modify", "src/main.rs");
        assert!(matches!(result, GuardrailVerdict::Advisory { .. }));
    }

    #[test]
    fn evaluate_boundary_denies_denied_path() {
        let g = Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Auth boundary".into(),
            description: None,
            category: GuardrailCategory::Boundary,
            scope: GuardrailScope::Session,
            enforcement: GuardrailEnforcement::Enforced,
            rule: GuardrailRule::Boundary {
                description: "Stay within auth feature".into(),
                allowed_paths: vec!["src/features/auth/".into()],
                denied_paths: vec![".env".into(), "secrets/".into()],
            },
            version: 1,
            sort_order: 0,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        // Denied path
        let result = g.evaluate_action("file_modify", ".env");
        assert!(matches!(result, GuardrailVerdict::Denied { .. }));
        // Outside allowed paths
        let result = g.evaluate_action("file_modify", "src/features/billing/main.rs");
        assert!(matches!(result, GuardrailVerdict::Denied { .. }));
        // Inside allowed path
        let result = g.evaluate_action("file_modify", "src/features/auth/login.rs");
        assert_eq!(result, GuardrailVerdict::Allowed);
    }

    #[test]
    fn evaluate_disabled_guardrail_always_allows() {
        let g = Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Disabled".into(),
            description: None,
            category: GuardrailCategory::Prohibition,
            scope: GuardrailScope::Workspace,
            enforcement: GuardrailEnforcement::Enforced,
            rule: GuardrailRule::Prohibition {
                description: "Block everything".into(),
                patterns: vec!["".into()], // matches all
                actions: vec!["file_modify".into()],
            },
            version: 1,
            sort_order: 0,
            enabled: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let result = g.evaluate_action("file_modify", "anything");
        assert_eq!(result, GuardrailVerdict::Allowed);
    }

    #[test]
    fn evaluate_requirement_always_allows() {
        let g = Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "TDD".into(),
            description: None,
            category: GuardrailCategory::Requirement,
            scope: GuardrailScope::Workspace,
            enforcement: GuardrailEnforcement::Enforced,
            rule: GuardrailRule::Requirement {
                description: "Tests first".into(),
                check: "tdd_order".into(),
                params: serde_json::json!({}),
            },
            version: 1,
            sort_order: 0,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        // Requirements can't be evaluated against a single action
        let result = g.evaluate_action("file_modify", "src/main.rs");
        assert_eq!(result, GuardrailVerdict::Allowed);
    }

    #[test]
    fn is_active_returns_enabled_status() {
        let make = |enabled: bool| Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "test".into(),
            description: None,
            category: GuardrailCategory::Prohibition,
            scope: GuardrailScope::Workspace,
            enforcement: GuardrailEnforcement::Enforced,
            rule: GuardrailRule::Prohibition {
                description: "d".into(),
                patterns: vec![],
                actions: vec![],
            },
            version: 1,
            sort_order: 0,
            enabled,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(make(true).is_active());
        assert!(!make(false).is_active());
    }

    #[test]
    fn guardrail_category_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&GuardrailCategory::Prohibition).unwrap(),
            r#""prohibition""#
        );
        assert_eq!(
            serde_json::to_string(&GuardrailCategory::Requirement).unwrap(),
            r#""requirement""#
        );
        assert_eq!(
            serde_json::to_string(&GuardrailCategory::Boundary).unwrap(),
            r#""boundary""#
        );
        assert_eq!(
            serde_json::to_string(&GuardrailCategory::Preference).unwrap(),
            r#""preference""#
        );
    }

    #[test]
    fn guardrail_enforcement_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&GuardrailEnforcement::Enforced).unwrap(),
            r#""enforced""#
        );
        assert_eq!(
            serde_json::to_string(&GuardrailEnforcement::Advisory).unwrap(),
            r#""advisory""#
        );
    }

    #[test]
    fn guardrail_rule_prohibition_serde_roundtrip() {
        let rule = GuardrailRule::Prohibition {
            description: "No secrets".into(),
            patterns: vec![".env".into(), "secrets/".into()],
            actions: vec!["file_modify".into()],
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: GuardrailRule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule, back);
    }

    #[test]
    fn guardrail_serde_roundtrip() {
        let now = Utc::now();
        let g = Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "File boundary".into(),
            description: Some("Restrict file access".into()),
            category: GuardrailCategory::Boundary,
            scope: GuardrailScope::Session,
            enforcement: GuardrailEnforcement::Enforced,
            rule: GuardrailRule::Boundary {
                description: "Stay in src".into(),
                allowed_paths: vec!["src/".into()],
                denied_paths: vec![".env".into()],
            },
            version: 2,
            sort_order: 1,
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&g).unwrap();
        let back: Guardrail = serde_json::from_str(&json).unwrap();
        assert_eq!(g.id, back.id);
        assert_eq!(g.name, back.name);
        assert_eq!(g.category, back.category);
        assert_eq!(g.enforcement, back.enforcement);
        assert_eq!(g.rule, back.rule);
    }
}

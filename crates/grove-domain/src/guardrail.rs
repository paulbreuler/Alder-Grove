use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailCategory {
    Prohibition,
    Requirement,
    Preference,
    Boundary,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn guardrail_category_serializes_correctly() {
        let json = serde_json::to_string(&GuardrailCategory::Prohibition).unwrap();
        assert_eq!(json, r#""prohibition""#);
        let json = serde_json::to_string(&GuardrailCategory::Requirement).unwrap();
        assert_eq!(json, r#""requirement""#);
        let json = serde_json::to_string(&GuardrailCategory::Preference).unwrap();
        assert_eq!(json, r#""preference""#);
        let json = serde_json::to_string(&GuardrailCategory::Boundary).unwrap();
        assert_eq!(json, r#""boundary""#);
    }

    #[test]
    fn guardrail_rule_prohibition_serde() {
        let rule = GuardrailRule::Prohibition {
            description: "No direct DB access".into(),
            patterns: vec!["sql!".into(), "raw_query".into()],
            actions: vec!["block".into(), "alert".into()],
        };
        let json = serde_json::to_string(&rule).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["type"], "prohibition");
        assert_eq!(val["description"], "No direct DB access");
        let back: GuardrailRule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule, back);
    }

    #[test]
    fn guardrail_rule_boundary_serde() {
        let rule = GuardrailRule::Boundary {
            description: "Only modify src/".into(),
            allowed_paths: vec!["src/".into(), "tests/".into()],
            denied_paths: vec![".env".into(), "secrets/".into()],
        };
        let json = serde_json::to_string(&rule).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["type"], "boundary");
        let back: GuardrailRule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule, back);
    }

    #[test]
    fn guardrail_serde_roundtrip() {
        let now = Utc::now();
        let guardrail = Guardrail {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "No raw SQL".into(),
            description: Some("Prevent direct database queries".into()),
            category: GuardrailCategory::Prohibition,
            scope: GuardrailScope::Workspace,
            enforcement: GuardrailEnforcement::Enforced,
            rule: GuardrailRule::Prohibition {
                description: "No raw SQL queries".into(),
                patterns: vec!["raw_sql".into()],
                actions: vec!["block".into()],
            },
            version: 1,
            sort_order: 0,
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&guardrail).unwrap();
        let back: Guardrail = serde_json::from_str(&json).unwrap();
        assert_eq!(guardrail.id, back.id);
        assert_eq!(guardrail.name, back.name);
        assert_eq!(guardrail.category, back.category);
        assert_eq!(guardrail.scope, back.scope);
        assert_eq!(guardrail.enforcement, back.enforcement);
        assert_eq!(guardrail.rule, back.rule);
        assert_eq!(guardrail.version, back.version);
        assert_eq!(guardrail.enabled, back.enabled);
    }
}

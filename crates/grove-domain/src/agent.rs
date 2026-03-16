use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum AgentStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Agent {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub provider: String,
    pub model: Option<String>,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
    pub config: serde_json::Value,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn agent_status_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&AgentStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::to_string(&AgentStatus::Disabled).unwrap(),
            "\"disabled\""
        );
    }

    #[test]
    fn agent_serde_roundtrip() {
        let a = Agent {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "Claude Code".into(),
            provider: "anthropic".into(),
            model: Some("claude-opus-4-20250514".into()),
            description: Some("Code generation agent".into()),
            capabilities: vec!["code_generation".into(), "code_review".into()],
            config: serde_json::json!({
                "max_tokens": 8192,
                "temperature": 0.7
            }),
            status: AgentStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Agent = serde_json::from_str(&json).unwrap();
        assert_eq!(a.id, back.id);
        assert_eq!(a.provider, back.provider);
        assert_eq!(a.capabilities.len(), 2);
        assert_eq!(a.status, back.status);
    }
}

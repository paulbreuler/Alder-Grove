use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub id: Uuid,
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn workspace_serde_roundtrip() {
        let now = Utc::now();
        let ws = Workspace {
            id: Uuid::now_v7(),
            org_id: "org_clerk_123".into(),
            name: "My Workspace".into(),
            description: Some("A test workspace".into()),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&ws).unwrap();
        let back: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(ws.id, back.id);
        assert_eq!(ws.org_id, back.org_id);
        assert_eq!(ws.name, back.name);
        assert_eq!(ws.description, back.description);
    }
}

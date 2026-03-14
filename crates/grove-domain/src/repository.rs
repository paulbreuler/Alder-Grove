use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub url: String,
    pub default_branch: String,
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
    fn repository_serde_roundtrip() {
        let now = Utc::now();
        let repo = Repository {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            name: "grove-api".into(),
            url: "https://github.com/org/grove-api".into(),
            default_branch: "main".into(),
            description: Some("The API repo".into()),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&repo).unwrap();
        let back: Repository = serde_json::from_str(&json).unwrap();
        assert_eq!(repo.id, back.id);
        assert_eq!(repo.url, back.url);
        assert_eq!(repo.default_branch, back.default_branch);
    }
}

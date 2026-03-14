use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::AiProvenance;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    Pending,
    Analyzing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub repository_id: Uuid,
    pub status: SnapshotStatus,
    pub summary: Option<String>,
    pub analysis: serde_json::Value,
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
    fn snapshot_status_analyzing_serializes_correctly() {
        let json = serde_json::to_string(&SnapshotStatus::Analyzing).unwrap();
        assert_eq!(json, r#""analyzing""#);
    }

    #[test]
    fn snapshot_serde_roundtrip() {
        let now = Utc::now();
        let snapshot = Snapshot {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            repository_id: Uuid::now_v7(),
            status: SnapshotStatus::Completed,
            summary: Some("3 modules, 42 files".into()),
            analysis: serde_json::json!({
                "modules": 3,
                "files": 42,
                "languages": ["rust", "typescript"]
            }),
            ai: AiProvenance::new(true, Some(0.97), Some("Automated analysis".into())).unwrap(),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let back: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot.id, back.id);
        assert_eq!(snapshot.status, back.status);
        assert_eq!(snapshot.summary, back.summary);
        assert_eq!(back.analysis["modules"], 3);
        assert_eq!(back.analysis["files"], 42);
        assert_eq!(back.ai.ai_authored(), snapshot.ai.ai_authored());
    }
}

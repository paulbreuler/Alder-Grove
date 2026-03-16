use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::AiProvenance;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Task {
    pub id: Uuid,
    pub specification_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub status: TaskStatus,
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
    fn task_status_in_progress_serializes_to_snake_case() {
        let json = serde_json::to_string(&TaskStatus::InProgress).unwrap();
        assert_eq!(json, r#""in_progress""#);
    }

    #[test]
    fn task_serde_roundtrip() {
        let now = Utc::now();
        let task = Task {
            id: Uuid::now_v7(),
            specification_id: Uuid::now_v7(),
            title: "Implement login endpoint".into(),
            description: Some("POST /api/auth/login".into()),
            sort_order: 1,
            status: TaskStatus::Pending,
            ai: AiProvenance::new(true, Some(0.88), None).unwrap(),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&task).unwrap();
        let back: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task.id, back.id);
        assert_eq!(task.title, back.title);
        assert_eq!(task.status, back.status);
        assert_eq!(task.sort_order, back.sort_order);
    }
}

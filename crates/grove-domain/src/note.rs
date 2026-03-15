use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::AiProvenance;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoteCategory {
    Decision,
    Learning,
    Gotcha,
    General,
}

/// All entity types that can be linked to a note.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkableEntityType {
    Journey,
    Step,
    Specification,
    Task,
    Persona,
    Repository,
    Session,
    Agent,
    Gate,
    Guardrail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub content: String,
    pub category: NoteCategory,
    #[serde(flatten)]
    pub ai: AiProvenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoteLink {
    pub id: Uuid,
    pub note_id: Uuid,
    pub entity_type: LinkableEntityType,
    pub entity_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::AiProvenance;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn note_category_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&NoteCategory::Gotcha).unwrap(),
            r#""gotcha""#
        );
        assert_eq!(
            serde_json::to_string(&NoteCategory::Decision).unwrap(),
            r#""decision""#
        );
        assert_eq!(
            serde_json::to_string(&NoteCategory::Learning).unwrap(),
            r#""learning""#
        );
        assert_eq!(
            serde_json::to_string(&NoteCategory::General).unwrap(),
            r#""general""#
        );
    }

    #[test]
    fn linkable_entity_type_all_variants_roundtrip() {
        let variants = vec![
            LinkableEntityType::Journey,
            LinkableEntityType::Step,
            LinkableEntityType::Specification,
            LinkableEntityType::Task,
            LinkableEntityType::Persona,
            LinkableEntityType::Repository,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let back: LinkableEntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn linkable_entity_type_includes_acp_variants() {
        let variants = vec![
            LinkableEntityType::Session,
            LinkableEntityType::Agent,
            LinkableEntityType::Gate,
            LinkableEntityType::Guardrail,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let back: LinkableEntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn note_serde_roundtrip() {
        let now = Utc::now();
        let note = Note {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            title: "Use UUIDv7 for PKs".into(),
            content: "UUIDv7 is time-ordered, better for DB indexing".into(),
            category: NoteCategory::Decision,
            ai: AiProvenance::default(),
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&note).unwrap();
        let back: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(note.id, back.id);
        assert_eq!(note.title, back.title);
        assert_eq!(note.category, back.category);
    }

    #[test]
    fn note_link_serde_roundtrip() {
        let now = Utc::now();
        let link = NoteLink {
            id: Uuid::now_v7(),
            note_id: Uuid::now_v7(),
            entity_type: LinkableEntityType::Specification,
            entity_id: Uuid::now_v7(),
            created_at: now,
        };
        let json = serde_json::to_string(&link).unwrap();
        let back: NoteLink = serde_json::from_str(&json).unwrap();
        assert_eq!(link.id, back.id);
        assert_eq!(link.note_id, back.note_id);
        assert_eq!(link.entity_type, back.entity_type);
        assert_eq!(link.entity_id, back.entity_id);
    }
}

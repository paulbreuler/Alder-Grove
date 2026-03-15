use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Entity types that support CRDT collaborative editing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CollaborativeEntityType {
    Specification,
    Task,
    Note,
    Journey,
    Step,
    Persona,
}

/// CRDT binary state for one field of one entity.
/// Implementation detail of the sync layer — stores Yrs document state
/// for reconnect/resume during real-time co-editing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollaborativeDocument {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub entity_type: CollaborativeEntityType,
    pub entity_id: Uuid,
    pub field_name: String,
    pub crdt_state: Vec<u8>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn collaborative_entity_type_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&CollaborativeEntityType::Specification).unwrap(),
            "\"specification\""
        );
    }

    #[test]
    fn collaborative_entity_type_covers_all_crdt_entities() {
        let types = vec![
            CollaborativeEntityType::Specification,
            CollaborativeEntityType::Task,
            CollaborativeEntityType::Note,
            CollaborativeEntityType::Journey,
            CollaborativeEntityType::Step,
            CollaborativeEntityType::Persona,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let back: CollaborativeEntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn collaborative_document_roundtrip() {
        let doc = CollaborativeDocument {
            id: Uuid::now_v7(),
            workspace_id: Uuid::now_v7(),
            entity_type: CollaborativeEntityType::Specification,
            entity_id: Uuid::now_v7(),
            field_name: "description".into(),
            crdt_state: vec![0x01, 0x02, 0x03],
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let back: CollaborativeDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc.id, back.id);
        assert_eq!(doc.field_name, back.field_name);
        assert_eq!(doc.crdt_state, back.crdt_state);
    }
}

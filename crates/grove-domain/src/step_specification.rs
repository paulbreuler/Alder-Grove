use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepSpecification {
    pub step_id: Uuid,
    pub specification_id: Uuid,
    pub sort_order: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_specification_serde_roundtrip() {
        let link = StepSpecification {
            step_id: Uuid::now_v7(),
            specification_id: Uuid::now_v7(),
            sort_order: Some(10),
        };
        let json = serde_json::to_string(&link).unwrap();
        let back: StepSpecification = serde_json::from_str(&json).unwrap();
        assert_eq!(link, back);
    }

    #[test]
    fn step_specification_allows_missing_sort_order() {
        let json = format!(
            r#"{{"step_id":"{}","specification_id":"{}","sort_order":null}}"#,
            Uuid::now_v7(),
            Uuid::now_v7()
        );
        let back: StepSpecification = serde_json::from_str(&json).unwrap();
        assert!(back.sort_order.is_none());
    }
}

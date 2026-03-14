use thiserror::Error;

/// Domain-level errors. Pure — no HTTP, no framework types.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("{entity} not found: {id}")]
    NotFound { entity: String, id: String },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_displays_entity_and_id() {
        let err = DomainError::NotFound {
            entity: "persona".into(),
            id: "abc-123".into(),
        };
        assert!(err.to_string().contains("persona"));
        assert!(err.to_string().contains("abc-123"));
    }

    #[test]
    fn validation_displays_message() {
        let err = DomainError::Validation("name is required".into());
        assert!(err.to_string().contains("name is required"));
    }
}

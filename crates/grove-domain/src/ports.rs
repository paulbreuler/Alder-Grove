//! Port traits (repository interfaces) for hexagonal architecture.
//!
//! These traits define the boundary between domain and adapters.
//! Implementations live in grove-api (PostgreSQL) or grove-tauri (API proxy).
//! All traits are `Send + Sync` to support async runtimes.

use uuid::Uuid;

use crate::collaborative_document::{CollaborativeDocument, CollaborativeEntityType};
use crate::error::DomainError;
use crate::event::Event;
use crate::gate::{Gate, GateDefinition};
use crate::guardrail::{Guardrail, GuardrailScope};
use crate::note::{LinkableEntityType, Note, NoteLink};
use crate::session::{Session, SessionStatus};
use crate::workspace::Workspace;

/// Generic CRUD operations for a scoped entity repository.
///
/// `T` is the entity type. `ScopeId` is the scoping parameter
/// (typically `Uuid` for workspace-scoped or parent-scoped entities).
#[async_trait::async_trait]
pub trait CrudRepository<T, ScopeId = Uuid>: Send + Sync
where
    T: Send + Sync + 'static,
    ScopeId: Send + Sync + 'static,
{
    async fn find_all(&self, scope_id: ScopeId) -> Result<Vec<T>, DomainError>;
    async fn find_by_id(&self, scope_id: ScopeId, id: Uuid) -> Result<Option<T>, DomainError>;
    async fn create(&self, entity: &T) -> Result<T, DomainError>;
    async fn update(&self, entity: &T) -> Result<T, DomainError>;
    async fn delete(&self, scope_id: ScopeId, id: Uuid) -> Result<(), DomainError>;
}

/// Workspace repository — scoped by org_id (Clerk-managed).
#[async_trait::async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn find_all(&self, org_id: &str) -> Result<Vec<Workspace>, DomainError>;
    async fn find_by_id(&self, org_id: &str, id: Uuid) -> Result<Option<Workspace>, DomainError>;
    async fn create(&self, workspace: &Workspace) -> Result<Workspace, DomainError>;
    async fn update(&self, workspace: &Workspace) -> Result<Workspace, DomainError>;
    async fn delete(&self, org_id: &str, id: Uuid) -> Result<(), DomainError>;
}

/// Note repository — workspace-scoped CRUD + link operations.
#[async_trait::async_trait]
pub trait NoteRepository: CrudRepository<Note> {
    async fn find_links(&self, note_id: Uuid) -> Result<Vec<NoteLink>, DomainError>;
    async fn add_link(
        &self,
        note_id: Uuid,
        entity_type: LinkableEntityType,
        entity_id: Uuid,
    ) -> Result<NoteLink, DomainError>;
    async fn remove_link(&self, link_id: Uuid) -> Result<(), DomainError>;
}

/// Session repository — workspace-scoped CRUD + status filtering.
#[async_trait::async_trait]
pub trait SessionRepository: CrudRepository<Session> {
    async fn find_by_status(
        &self,
        workspace_id: Uuid,
        status: SessionStatus,
    ) -> Result<Vec<Session>, DomainError>;
}

/// Gate definition repository — workspace-scoped CRUD + enabled filtering.
#[async_trait::async_trait]
pub trait GateDefinitionRepository: CrudRepository<GateDefinition> {
    async fn find_enabled(&self, workspace_id: Uuid) -> Result<Vec<GateDefinition>, DomainError>;
    async fn find_disabled(&self, workspace_id: Uuid) -> Result<Vec<GateDefinition>, DomainError>;
}

/// Gate repository — session-scoped CRUD + pending filtering.
#[async_trait::async_trait]
pub trait GateRepository: CrudRepository<Gate> {
    async fn find_pending(&self, session_id: Uuid) -> Result<Vec<Gate>, DomainError>;
}

/// Event repository — scoped by workspace_id + session_id, append-only (create only).
///
/// `workspace_id` is required for RLS-scoped TenantTx isolation.
#[async_trait::async_trait]
pub trait EventRepository: Send + Sync {
    async fn find_all(
        &self,
        workspace_id: Uuid,
        session_id: Uuid,
    ) -> Result<Vec<Event>, DomainError>;
    async fn create(&self, event: &Event) -> Result<Event, DomainError>;
}

/// Guardrail repository — workspace-scoped CRUD + filtered queries.
#[async_trait::async_trait]
pub trait GuardrailRepository: CrudRepository<Guardrail> {
    async fn find_enabled_by_scope(
        &self,
        workspace_id: Uuid,
        scope: GuardrailScope,
    ) -> Result<Vec<Guardrail>, DomainError>;

    /// Find guardrails with optional scope and enabled filters pushed into SQL.
    async fn find_filtered(
        &self,
        workspace_id: Uuid,
        scope: Option<GuardrailScope>,
        enabled: Option<bool>,
    ) -> Result<Vec<Guardrail>, DomainError>;
}

/// Collaborative document repository — find by entity + upsert.
#[async_trait::async_trait]
pub trait CollaborativeDocumentRepository: Send + Sync {
    async fn find_by_entity(
        &self,
        entity_type: CollaborativeEntityType,
        entity_id: Uuid,
        field_name: &str,
    ) -> Result<Option<CollaborativeDocument>, DomainError>;
    async fn upsert(
        &self,
        document: &CollaborativeDocument,
    ) -> Result<CollaborativeDocument, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persona::Persona;
    use std::sync::Arc;

    struct MockWorkspaceRepo;

    #[async_trait::async_trait]
    impl WorkspaceRepository for MockWorkspaceRepo {
        async fn find_all(&self, _org_id: &str) -> Result<Vec<Workspace>, DomainError> {
            Ok(vec![])
        }

        async fn find_by_id(
            &self,
            _org_id: &str,
            _id: Uuid,
        ) -> Result<Option<Workspace>, DomainError> {
            Ok(None)
        }

        async fn create(&self, _workspace: &Workspace) -> Result<Workspace, DomainError> {
            Err(DomainError::Internal("not implemented".into()))
        }

        async fn update(&self, _workspace: &Workspace) -> Result<Workspace, DomainError> {
            Err(DomainError::Internal("not implemented".into()))
        }

        async fn delete(&self, _org_id: &str, _id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn mock_workspace_repo_compiles_and_works() {
        let repo = MockWorkspaceRepo;
        let result = repo.find_all("org_test_123").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // --- Step 1: RED — CrudRepository generic mock tests ---

    struct MockPersonaCrud;

    #[async_trait::async_trait]
    impl CrudRepository<Persona> for MockPersonaCrud {
        async fn find_all(&self, _scope_id: Uuid) -> Result<Vec<Persona>, DomainError> {
            Ok(vec![])
        }
        async fn find_by_id(
            &self,
            _scope_id: Uuid,
            _id: Uuid,
        ) -> Result<Option<Persona>, DomainError> {
            Ok(None)
        }
        async fn create(&self, _entity: &Persona) -> Result<Persona, DomainError> {
            Err(DomainError::Internal("not impl".into()))
        }
        async fn update(&self, _entity: &Persona) -> Result<Persona, DomainError> {
            Err(DomainError::Internal("not impl".into()))
        }
        async fn delete(&self, _scope_id: Uuid, _id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn crud_repository_generic_mock_compiles_and_works() {
        let repo = MockPersonaCrud;
        let result = repo.find_all(Uuid::now_v7()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn crud_repository_is_object_safe() {
        let repo: Arc<dyn CrudRepository<Persona>> = Arc::new(MockPersonaCrud);
        let result = repo.find_all(Uuid::now_v7()).await;
        assert!(result.is_ok());
    }

    // --- Step 3: RED — SessionRepository extends CrudRepository ---

    struct MockSessionRepo;

    #[async_trait::async_trait]
    impl CrudRepository<Session> for MockSessionRepo {
        async fn find_all(&self, _scope_id: Uuid) -> Result<Vec<Session>, DomainError> {
            Ok(vec![])
        }
        async fn find_by_id(
            &self,
            _scope_id: Uuid,
            _id: Uuid,
        ) -> Result<Option<Session>, DomainError> {
            Ok(None)
        }
        async fn create(&self, _entity: &Session) -> Result<Session, DomainError> {
            Err(DomainError::Internal("not impl".into()))
        }
        async fn update(&self, _entity: &Session) -> Result<Session, DomainError> {
            Err(DomainError::Internal("not impl".into()))
        }
        async fn delete(&self, _scope_id: Uuid, _id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl SessionRepository for MockSessionRepo {
        async fn find_by_status(
            &self,
            _workspace_id: Uuid,
            _status: SessionStatus,
        ) -> Result<Vec<Session>, DomainError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn session_repository_extends_crud() {
        let repo: Arc<dyn SessionRepository> = Arc::new(MockSessionRepo);
        // Base CRUD works
        let result = repo.find_all(Uuid::now_v7()).await;
        assert!(result.is_ok());
        // Extended method works
        let result = repo
            .find_by_status(Uuid::now_v7(), SessionStatus::Active)
            .await;
        assert!(result.is_ok());
    }
}

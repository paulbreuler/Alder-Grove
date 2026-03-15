//! Port traits (repository interfaces) for hexagonal architecture.
//!
//! These traits define the boundary between domain and adapters.
//! Implementations live in grove-api (PostgreSQL) or grove-tauri (API proxy).
//! All traits are `Send + Sync` to support async runtimes.

use uuid::Uuid;

use crate::agent::Agent;
use crate::collaborative_document::{CollaborativeDocument, CollaborativeEntityType};
use crate::error::DomainError;
use crate::event::Event;
use crate::gate::{Gate, GateDefinition};
use crate::guardrail::{Guardrail, GuardrailScope};
use crate::journey::Journey;
use crate::note::{LinkableEntityType, Note, NoteLink};
use crate::persona::Persona;
use crate::repository::Repository;
use crate::session::{Session, SessionStatus};
use crate::snapshot::Snapshot;
use crate::specification::Specification;
use crate::step::Step;
use crate::task::Task;
use crate::workspace::Workspace;

/// Workspace repository — scoped by org_id (Clerk-managed).
#[async_trait::async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn find_all(&self, org_id: &str) -> Result<Vec<Workspace>, DomainError>;
    async fn find_by_id(&self, org_id: &str, id: Uuid) -> Result<Option<Workspace>, DomainError>;
    async fn create(&self, workspace: &Workspace) -> Result<Workspace, DomainError>;
    async fn update(&self, workspace: &Workspace) -> Result<Workspace, DomainError>;
    async fn delete(&self, org_id: &str, id: Uuid) -> Result<(), DomainError>;
}

/// Persona repository — scoped by workspace_id.
#[async_trait::async_trait]
pub trait PersonaRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Persona>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Persona>, DomainError>;
    async fn create(&self, persona: &Persona) -> Result<Persona, DomainError>;
    async fn update(&self, persona: &Persona) -> Result<Persona, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Repository (linked codebase) repository — scoped by workspace_id.
#[async_trait::async_trait]
pub trait RepositoryRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Repository>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Repository>, DomainError>;
    async fn create(&self, repository: &Repository) -> Result<Repository, DomainError>;
    async fn update(&self, repository: &Repository) -> Result<Repository, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Journey repository — scoped by workspace_id.
#[async_trait::async_trait]
pub trait JourneyRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Journey>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Journey>, DomainError>;
    async fn create(&self, journey: &Journey) -> Result<Journey, DomainError>;
    async fn update(&self, journey: &Journey) -> Result<Journey, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Step repository — scoped by journey_id.
#[async_trait::async_trait]
pub trait StepRepository: Send + Sync {
    async fn find_all_by_journey(&self, journey_id: Uuid) -> Result<Vec<Step>, DomainError>;
    async fn find_by_id(&self, journey_id: Uuid, id: Uuid)
        -> Result<Option<Step>, DomainError>;
    async fn create(&self, step: &Step) -> Result<Step, DomainError>;
    async fn update(&self, step: &Step) -> Result<Step, DomainError>;
    async fn delete(&self, journey_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Specification repository — scoped by workspace_id.
#[async_trait::async_trait]
pub trait SpecificationRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Specification>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Specification>, DomainError>;
    async fn create(&self, specification: &Specification) -> Result<Specification, DomainError>;
    async fn update(&self, specification: &Specification) -> Result<Specification, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Task repository — scoped by specification_id.
#[async_trait::async_trait]
pub trait TaskRepository: Send + Sync {
    async fn find_all_by_spec(&self, specification_id: Uuid) -> Result<Vec<Task>, DomainError>;
    async fn find_by_id(
        &self,
        specification_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Task>, DomainError>;
    async fn create(&self, task: &Task) -> Result<Task, DomainError>;
    async fn update(&self, task: &Task) -> Result<Task, DomainError>;
    async fn delete(&self, specification_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Note repository — scoped by workspace_id, with link operations.
#[async_trait::async_trait]
pub trait NoteRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Note>, DomainError>;
    async fn find_by_id(&self, workspace_id: Uuid, id: Uuid)
        -> Result<Option<Note>, DomainError>;
    async fn create(&self, note: &Note) -> Result<Note, DomainError>;
    async fn update(&self, note: &Note) -> Result<Note, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;

    // Link operations
    async fn find_links(&self, note_id: Uuid) -> Result<Vec<NoteLink>, DomainError>;
    async fn add_link(
        &self,
        note_id: Uuid,
        entity_type: LinkableEntityType,
        entity_id: Uuid,
    ) -> Result<NoteLink, DomainError>;
    async fn remove_link(&self, link_id: Uuid) -> Result<(), DomainError>;
}

/// Snapshot repository — scoped by workspace_id.
#[async_trait::async_trait]
pub trait SnapshotRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Snapshot>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Snapshot>, DomainError>;
    async fn create(&self, snapshot: &Snapshot) -> Result<Snapshot, DomainError>;
    async fn update(&self, snapshot: &Snapshot) -> Result<Snapshot, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Agent repository — scoped by workspace_id.
#[async_trait::async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Agent>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Agent>, DomainError>;
    async fn create(&self, agent: &Agent) -> Result<Agent, DomainError>;
    async fn update(&self, agent: &Agent) -> Result<Agent, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Session repository — scoped by workspace_id, with status filtering.
#[async_trait::async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Session>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Session>, DomainError>;
    async fn find_by_status(
        &self,
        workspace_id: Uuid,
        status: SessionStatus,
    ) -> Result<Vec<Session>, DomainError>;
    async fn create(&self, session: &Session) -> Result<Session, DomainError>;
    async fn update(&self, session: &Session) -> Result<Session, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Gate definition repository — scoped by workspace_id, with enabled filtering.
#[async_trait::async_trait]
pub trait GateDefinitionRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<GateDefinition>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<GateDefinition>, DomainError>;
    async fn find_enabled(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<GateDefinition>, DomainError>;
    async fn create(
        &self,
        gate_definition: &GateDefinition,
    ) -> Result<GateDefinition, DomainError>;
    async fn update(
        &self,
        gate_definition: &GateDefinition,
    ) -> Result<GateDefinition, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Gate repository — scoped by session_id, with pending filtering.
#[async_trait::async_trait]
pub trait GateRepository: Send + Sync {
    async fn find_all(&self, session_id: Uuid) -> Result<Vec<Gate>, DomainError>;
    async fn find_by_id(&self, session_id: Uuid, id: Uuid)
        -> Result<Option<Gate>, DomainError>;
    async fn find_pending(&self, session_id: Uuid) -> Result<Vec<Gate>, DomainError>;
    async fn create(&self, gate: &Gate) -> Result<Gate, DomainError>;
    async fn update(&self, gate: &Gate) -> Result<Gate, DomainError>;
    async fn delete(&self, session_id: Uuid, id: Uuid) -> Result<(), DomainError>;
}

/// Event repository — scoped by session_id, append-only (create only).
#[async_trait::async_trait]
pub trait EventRepository: Send + Sync {
    async fn find_all(&self, session_id: Uuid) -> Result<Vec<Event>, DomainError>;
    async fn create(&self, event: &Event) -> Result<Event, DomainError>;
}

/// Guardrail repository — scoped by workspace_id, with enabled-by-scope filtering.
#[async_trait::async_trait]
pub trait GuardrailRepository: Send + Sync {
    async fn find_all(&self, workspace_id: Uuid) -> Result<Vec<Guardrail>, DomainError>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Guardrail>, DomainError>;
    async fn find_enabled_by_scope(
        &self,
        workspace_id: Uuid,
        scope: GuardrailScope,
    ) -> Result<Vec<Guardrail>, DomainError>;
    async fn create(&self, guardrail: &Guardrail) -> Result<Guardrail, DomainError>;
    async fn update(&self, guardrail: &Guardrail) -> Result<Guardrail, DomainError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), DomainError>;
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
}

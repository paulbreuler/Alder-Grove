# Alder Grove — Architecture Flows

## Request Flow (Desktop → API)

```mermaid
graph LR
    A[Tauri Shell<br/>Desktop] --> B[React 19.2<br/>Webview]
    B --> C[Axum 0.8 API<br/>Cloud]
    C --> D[PostgreSQL 18]
    B -.- E[ClerkProvider<br/>auth context]
    C -.- F[Clerk JWT validation]
    C -.- G[org_id / workspace_id scoping]
```

## Hexagonal Dependency Flow

```mermaid
graph BT
    UI[UI<br/>Components] --> App[Application<br/>Hooks, Stores, Use Cases]
    Adapters[Adapters<br/>API, Tauri, WS] --> App
    App --> Domain[Domain<br/>Types, Entities, Rules<br/>zero dependencies]
```

Dependencies flow inward. Domain has zero external dependencies.

## ACP Session Lifecycle

```mermaid
sequenceDiagram
    participant C as Client
    participant A as API
    participant Ag as Agent

    C->>A: Create Session
    A->>Ag: WebSocket connect
    Ag-->>A: Events (streaming)
    A-->>C: Stream Events
    Ag->>A: Gate Request
    A->>C: Gate Approval needed
    C->>A: Approve / Deny
    A->>Ag: Gate Decision
    Ag-->>A: Session Complete
    A-->>C: Final State
```

## Session State Machine

```mermaid
stateDiagram-v2
    [*] --> pending
    pending --> active : agent starts

    active --> completed : success
    active --> failed : error
    active --> cancelled : user cancels
    active --> gated : gate triggered

    gated --> active : gate approved
    gated --> cancelled : gate denied
    gated --> timed_out : gate expired

    completed --> [*]
    failed --> [*]
    cancelled --> [*]
    timed_out --> [*]
```

## Gate & Guardrail Enforcement Flow

```mermaid
flowchart TD
    A[Agent Action] --> B{Check Guardrails<br/>workspace + session scope}
    B -->|No violation| D{Check Gate Definitions<br/>trigger_type patterns}
    B -->|Violation| C{Enforced?}
    C -->|Yes| E[Trigger Gate<br/>Session → gated]
    C -->|No - advisory| F[Emit warning event]
    F --> D
    D -->|Pattern matched| E
    D -->|No match| G[Continue execution]
    E --> H[Notify human<br/>Wait for decision]
    H -->|Approved| G
    H -->|Denied| I[Session → cancelled]
    H -->|Timeout| J{timeout_action}
    J -->|cancel| I
    J -->|approve| G
    J -->|escalate| K[Escalate to admin]
```

## CRDT Sync Flow

```mermaid
sequenceDiagram
    participant H as Human<br/>React + Yjs
    participant S as API<br/>grove-sync + Yrs
    participant A as Agent<br/>Yrs client

    Note over H,A: State vector exchange on connect
    S-->>H: Current CRDT state
    S-->>A: Current CRDT state

    H->>S: Yjs update
    S->>S: Merge into Yrs doc
    S-->>A: Broadcast merged update

    A->>S: Yrs update
    S->>S: Merge into Yrs doc
    S-->>H: Broadcast merged update

    Note over S: Debounced persistence
    S->>S: Persist text → PostgreSQL column
    S->>S: Snapshot CRDT binary → collaborative_documents
```

## Awareness / Presence Protocol

```mermaid
sequenceDiagram
    participant H as Human
    participant S as API (hub)
    participant A as Agent

    H->>S: Awareness: cursor at spec:uuid:description pos 42
    S-->>A: Broadcast awareness state
    A->>S: Awareness: editing spec:uuid:description pos 108
    S-->>H: Broadcast awareness state

    Note over H,A: Ephemeral — not persisted
```

Awareness state carries: user identity (id, name, color, type), cursor position (entity, field, offset, selection), and activity label.

## WebSocket Multiplexing

Single WebSocket connection per client, carrying three tagged channels:

```mermaid
graph TD
    subgraph WebSocket Frame
        ACP["channel: acp<br/>GateDecision, UserMessage,<br/>AgentEvent, GateRequest,<br/>SessionStateChange, Error"]
        SYNC["channel: sync<br/>document_id + binary Yrs update"]
        AWARE["channel: awareness<br/>binary awareness payload<br/>user, cursor, activity"]
    end

    Client[Desktop Client] -->|multiplexed WS| API[grove-api]
    API -->|multiplexed WS| Agent[AI Agent]
```

## Crate Dependency Graph

```mermaid
graph BT
    domain[grove-domain<br/>serde, uuid, chrono, thiserror]
    sync[grove-sync<br/>yrs, sqlx] --> domain
    api[grove-api<br/>axum, sqlx, tower,<br/>tokio-tungstenite, utoipa] --> domain
    api --> sync
    tauri[grove-tauri<br/>tauri, reqwest,<br/>tokio-tungstenite, git2] --> domain
    tsgen[grove-ts-gen<br/>ts-rs] --> domain
```

**Rules:**
- `grove-domain` has ZERO framework dependencies
- Dependencies flow toward `grove-domain` (inward)
- `grove-api` is the only crate that depends on `grove-sync`
- `grove-tauri` and `grove-api` never depend on each other

## Tauri IPC Proxy Flow

```mermaid
sequenceDiagram
    participant R as React Component
    participant T as Tauri IPC<br/>grove-tauri
    participant A as Axum API<br/>grove-api
    participant D as PostgreSQL 18

    R->>T: invoke("persona_list", { workspace_id })
    Note over T: Validate input<br/>Attach auth token
    T->>A: HTTP request
    Note over A: JWT validation<br/>Workspace scoping<br/>Port trait dispatch
    A->>D: SELECT ... WHERE workspace_id = $1
    D-->>A: Result rows
    A-->>T: JSON response
    T-->>R: Deserialized entities
```

Frontend never hits the API directly. Auth token is managed in Rust, not exposed to the webview.

## Multi-Tenant Data Flow

```mermaid
flowchart TD
    A[Request] --> B[Clerk JWT Middleware<br/>Extract org_id from claims]
    B --> C[Workspace Middleware<br/>Verify membership<br/>Extract workspace_id]
    C --> D[Route Handler<br/>/orgs/org_id/workspaces/ws_id/...]
    D --> E[Database Query<br/>WHERE workspace_id = $1<br/>always scoped]
```

## Shell Extension Registration

```mermaid
graph TD
    B[bootstrapShell] --> H[HomeExtension]
    B --> W[WorkspaceExtension]
    B --> P[PersonasExtension]
    B --> J[JourneysExtension]
    B --> S[SpecsExtension]
    B --> A[ACPExtension]
    B --> Set[SettingsExtension]
    B --> Sn[SnapshotsExtension<br/>v2]
```

Each extension provides:
- Navigation entry
- Route definitions
- Feature-scoped Zustand store
- Self-contained UI components

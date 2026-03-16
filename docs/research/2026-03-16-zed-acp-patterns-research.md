# Research: Zed ACP Architecture Patterns for Alder Grove ACP Design

**Date:** 2026-03-15
**Status:** Complete
**Scope:** Zed codebase analysis across agent communication, WebSocket patterns, state machines, event streaming, gate/approval patterns, protocol framing, and error handling

---

## Question

What architectural patterns does Zed use for ACP (Agent Client Protocol), WebSocket real-time communication, state machines, event streaming, and human-in-the-loop approval flows that could improve Alder Grove's ACP design?

---

## Findings

### 1. Agent/AI Assistant Communication Architecture

Zed uses the **`agent-client-protocol` crate** (v0.10.2 from crates.io, `zed-industries/agent-client-protocol`) as a published protocol library — not a home-grown WebSocket implementation. The protocol is **stdio-based** (stdin/stdout between processes), not WebSocket-based. This is a fundamental architectural difference from Alder Grove's hub-and-spoke WebSocket design.

**Key protocol lifecycle** (`crates/agent_servers/src/acp.rs`):

```rust
// 1. Initialize handshake with capability negotiation
connection.initialize(
    acp::InitializeRequest::new(acp::ProtocolVersion::V1)
        .client_capabilities(
            acp::ClientCapabilities::new()
                .fs(acp::FileSystemCapabilities::new()
                    .read_text_file(true)
                    .write_text_file(true))
                .terminal(true)
        )
        .client_info(acp::Implementation::new("zed", version))
).await?;

// 2. Create or load a session
connection.new_session(acp::NewSessionRequest::new(cwd)).await?;

// 3. Send a prompt; response streams SessionUpdate events until StopReason
connection.prompt(params).await?;  // returns acp::PromptResponse
```

**`AgentConnection` trait** (`crates/acp_thread/src/connection.rs`) is the abstraction layer. It is a Rust trait (not a protocol) with optional capabilities:

```rust
pub trait AgentConnection {
    fn new_session(self: Rc<Self>, project: ..., cwd: &Path, cx: &mut App) -> Task<Result<Entity<AcpThread>>>;
    fn load_session(self: Rc<Self>, session_id: acp::SessionId, ...) -> Task<Result<Entity<AcpThread>>>;
    fn resume_session(self: Rc<Self>, session_id: acp::SessionId, ...) -> Task<Result<Entity<AcpThread>>>;
    fn close_session(self: Rc<Self>, session_id: &acp::SessionId, ...) -> Task<Result<()>>;
    fn prompt(&self, user_message_id: Option<UserMessageId>, params: acp::PromptRequest, cx: &mut App) -> Task<Result<acp::PromptResponse>>;
    fn cancel(&self, session_id: &acp::SessionId, cx: &mut App);
    // Optional capability methods — return None if unsupported:
    fn supports_load_session(&self) -> bool { false }
    fn supports_resume_session(&self) -> bool { false }
    fn model_selector(&self, session_id: &acp::SessionId) -> Option<Rc<dyn AgentModelSelector>> { None }
    fn session_modes(&self, ...) -> Option<Rc<dyn AgentSessionModes>> { None }
    fn session_list(&self, ...) -> Option<Rc<dyn AgentSessionList>> { None }
}
```

**Capability negotiation is done at two levels:**
- Protocol level: `InitializeRequest` / `InitializeResponse` exchange `AgentCapabilities` and `ClientCapabilities`
- Runtime level: Optional traits (`AgentModelSelector`, `AgentSessionModes`, `AgentSessionConfigOptions`) returned as `Option<Rc<dyn Trait>>`

**Session update streaming** uses a push model via `acp::SessionUpdate` enum:

```rust
// SessionUpdate variants pushed during prompt execution:
acp::SessionUpdate::UserMessageChunk(acp::ContentChunk { content, .. })
acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. })
acp::SessionUpdate::AgentThoughtChunk(acp::ContentChunk { content, .. })
acp::SessionUpdate::ToolCall(tool_call)
acp::SessionUpdate::ToolCallUpdate(tool_call_update)
acp::SessionUpdate::Plan(plan)
acp::SessionUpdate::AvailableCommandsUpdate(..)
acp::SessionUpdate::CurrentModeUpdate(..)
acp::SessionUpdate::ConfigOptionUpdate(..)
```

These are dispatched in `AcpThread::handle_session_update()` (`crates/acp_thread/src/acp_thread.rs:1304`).

---

### 2. WebSocket / Transport Patterns

Zed's **collab** real-time layer uses a different transport from ACP:

**`rpc/src/peer.rs`** implements a bidirectional protobuf-framed multiplexed connection (`Peer` struct). Key design:
- **Unbounded channel for outgoing** messages (never blocks application code)
- **Bounded channel (256) for incoming** (provides backpressure)
- Two channel types: `response_channels` (oneshot, for request/response) and `stream_response_channels` (unbounded mpsc, for streaming responses)
- Envelope-wrapped protobuf messages with `message_id` for correlation
- `ConnectionId { owner_id, id }` — epoch + monotonic counter for crash-safe reconnects

**`rpc/src/conn.rs`** wraps `async-tungstenite` WebSocket behind a `Sink/Stream` trait pair. The `Connection::in_memory()` test helper uses simulated delays and `AtomicBool` kill switch — excellent for integration tests without real network.

**ACP transport** (`crates/agent_servers/src/acp.rs`): Uses process stdio (stdin/stdout/stderr) with line-delimited JSON, not WebSocket. Three concurrent async tasks:
- `_io_task`: Main JSON-RPC I/O loop (background)
- `_wait_task`: Process exit monitor — on exit, notifies all sessions with `LoadError::Exited { status }`
- `_stderr_task`: Logs agent stderr as warnings

**JSON-RPC client** (`context_server/src/client.rs`) uses `smol::channel::unbounded()` for outbound and `parking_lot::Mutex<HashMap<RequestId, ResponseHandler>>` for pending request correlation. Request IDs are atomically incrementing `i32` values. Timeout is configurable per-request with a 60s default.

---

### 3. State Machine Patterns

**`ToolCallStatus`** (`crates/acp_thread/src/acp_thread.rs:492`) is the richest state machine example:

```rust
pub enum ToolCallStatus {
    Pending,                              // Shown to user, not yet running
    WaitingForConfirmation {              // GATE STATE — blocks execution
        options: PermissionOptions,
        respond_tx: oneshot::Sender<acp::PermissionOptionId>,
    },
    InProgress,
    Completed,
    Failed,
    Rejected,                             // User denied
    Canceled,                             // User canceled generation
}
```

**Key design insight:** The `WaitingForConfirmation` variant embeds a `oneshot::Sender` directly in the state enum. This makes the state machine self-contained — the channel IS the gate. Resolving the gate means sending on this channel, which unblocks the awaiting `oneshot::Receiver` in the agent's async execution path. No separate "pending gates" map is needed.

**`ThreadStatus`** is deliberately minimal:
```rust
pub enum ThreadStatus { Idle, Generating }
```
State is kept in two enums — one for the thread lifecycle, one for the tool call lifecycle — rather than one god-state.

**Session lifecycle from `connection.rs`** (the `AcpThread` constructor):
- `running_turn: Option<Task<...>>` holds the active prompt task
- Cancellation via `cancel()` sends `acp::CancelNotification` to the agent, which produces `StopReason::Cancelled`
- `suppress_abort_err` flag handles agent-specific error codes that should be treated as cancellations (not errors)

---

### 4. Event Streaming / Activity Logs

**`AcpThreadEvent`** (`crates/acp_thread/src/acp_thread.rs:1015`) is the event emitter enum for the GPUI entity system:

```rust
pub enum AcpThreadEvent {
    NewEntry,
    TitleUpdated,
    TokenUsageUpdated,
    EntryUpdated(usize),               // index into entries vec
    EntriesRemoved(Range<usize>),
    ToolAuthorizationRequested(acp::ToolCallId),
    ToolAuthorizationReceived(acp::ToolCallId),
    Retry(RetryStatus),
    SubagentSpawned(acp::SessionId),
    Stopped(acp::StopReason),
    Error,
    LoadError(LoadError),
    PromptCapabilitiesUpdated,
    Refusal,
    AvailableCommandsUpdated(Vec<acp::AvailableCommand>),
    ModeUpdated(acp::SessionModeId),
    ConfigOptionsUpdated(Vec<acp::SessionConfigOption>),
}
```

**`ActionLog`** (`crates/action_log/src/action_log.rs`) tracks file-level changes made by tools — a separate concern from the session event log. It uses:
- `BTreeMap<Entity<Buffer>, TrackedBuffer>` for change tracking
- `HashMap<PathBuf, MTime>` for read-time tracking (detects external modifications)
- A `linked_action_log` for parent/subagent rollup

**Streaming text buffer** (`streaming_text_buffer: Option<...>` field on `AcpThread`) is a debounce mechanism — incoming text chunks are buffered and flushed as discrete GPUI entity updates to prevent per-character rerender. This is the key performance pattern for streaming LLM output.

**Append-only invariant**: Zed's entry array only grows (or has ranges removed via `EntriesRemoved`). It never mutates existing entries in-place, only updates by index (`EntryUpdated(usize)`). This maps directly to Alder Grove's append-only `events` table design.

---

### 5. Gate / Approval Patterns

Zed's approval pattern (`crates/acp_thread/src/acp_thread.rs:1909`) is more fine-grained than Alder Grove's current design. Gates are **per-tool-call**, not per-session:

```rust
pub fn request_tool_call_authorization(
    &mut self,
    tool_call: acp::ToolCallUpdate,
    options: PermissionOptions,          // What approval options to show
    cx: &mut Context<Self>,
) -> Result<Task<acp::RequestPermissionOutcome>> {
    let (tx, rx) = oneshot::channel();

    // Gate state embeds the channel
    let status = ToolCallStatus::WaitingForConfirmation {
        options,
        respond_tx: tx,
    };

    self.upsert_tool_call_inner(tool_call, status, cx)?;
    cx.emit(AcpThreadEvent::ToolAuthorizationRequested(tool_call_id.clone()));

    // Returns a future that resolves when the user decides
    Ok(cx.spawn(async move |this, cx| {
        let outcome = match rx.await {
            Ok(option) => acp::RequestPermissionOutcome::Selected(
                acp::SelectedPermissionOutcome::new(option),
            ),
            Err(oneshot::Canceled) => acp::RequestPermissionOutcome::Cancelled,
        };
        this.update(cx, |_this, cx| {
            cx.emit(AcpThreadEvent::ToolAuthorizationReceived(tool_call_id))
        }).ok();
        outcome
    }))
}
```

**`PermissionOptions`** has two display modes:
```rust
pub enum PermissionOptions {
    Flat(Vec<acp::PermissionOption>),             // Button list
    Dropdown(Vec<PermissionOptionChoice>),          // Allow/deny pairs
}
```

**`PermissionOptionKind`** variants: `AllowOnce`, `AllowAlways`, `RejectOnce`, `RejectAlways` — sticky permissions (Always variants) are supported by the protocol but must be enforced by the agent.

**Key design difference from Alder Grove**: Zed gates are inline with tool execution and resolved via `oneshot` channels embedded in state. Alder Grove gates are database rows with polling semantics. The oneshot approach is lower latency and requires no database round-trip for the decision, but Alder Grove's persistence approach is more durable and auditable.

---

### 6. Protocol Message Framing

**ACP protocol** (public crate, `agent-client-protocol` v0.10.2):

Message types are JSON-RPC 2.0 over stdin/stdout. Key request/response pairs:
- `session/initialize` → `InitializeResponse { protocol_version, agent_capabilities, agent_info, auth_methods }`
- `session/new` → `NewSessionResponse { session_id, modes, models, config_options }`
- `session/load` → `LoadSessionResponse`
- `session/resume` → `ResumeSessionResponse`
- `session/close` → `CloseSessionResponse`
- `session/prompt` → streaming `SessionUpdate` notifications + `PromptResponse { stop_reason }`

**`StopReason` enum**: `EndTurn`, `Cancelled`, `MaxTokens`, `Refusal`

**`Meta`** type is `HashMap<String, serde_json::Value>` — an escape hatch for out-of-band data. Zed uses it to pass tool names (`TOOL_NAME_META_KEY`), subagent session info (`SUBAGENT_SESSION_INFO_META_KEY`), terminal auth info, and Gemini-specific auth overrides. This is explicitly a workaround for protocol limitations.

**`AcpStreamMessage`** (`crates/acp_tools/src/acp_tools.rs`) shows the debug protocol observer sees three message types:
```rust
acp::StreamMessageContent::Request { id, method, params }
acp::StreamMessageContent::Response { id, result }
acp::StreamMessageContent::Notification { method, params }
```

**Direction tracking**: `acp::StreamMessageDirection::Incoming` / `Outgoing` — every message is tagged with direction for debugging.

**collab (Peer) framing** uses protobuf envelopes with:
```
Envelope { message_id, payload: proto::Any }
```
Response correlation via `message_id`. Stream responses use a separate `stream_response_channels` map with `mpsc::UnboundedSender` per stream.

**JSON-RPC client** (`context_server/src/client.rs`) uses atomically incrementing `i32` IDs, with the interesting pattern that responses must be deserialized in two passes: first as `AnyResponse` to check for errors, then deserialize `result` field specifically.

---

### 7. Error Handling in Protocol Layers

**Three-tier error handling** in Zed's ACP:

1. **`acp::Error`** (protocol layer): Has `code: acp::ErrorCode` and `data: Option<serde_json::Value>`. Special codes include `ErrorCode::AuthRequired`. The `data` field carries structured error details.

2. **`LoadError`** (`crates/acp_thread/src/acp_thread.rs:1083`) (session layer):
```rust
pub enum LoadError {
    Unsupported { command, current_version, minimum_version },
    FailedToInstall(SharedString),
    Exited { status: ExitStatus },
    Other(SharedString),
}
```
Emitted via `AcpThreadEvent::LoadError` when the agent process exits or version is incompatible.

3. **`map_acp_error`** (`agent_servers/src/acp.rs:928`): Protocol errors are mapped to `anyhow::Error` at the connection boundary. `AuthRequired` errors are converted to the local `AuthRequired` struct (which implements `std::error::Error`) so UI can offer re-auth flows.

**`suppress_abort_err` flag**: A session-scoped flag set before `cancel()` is called. When the agent returns `InternalError` with "This operation was aborted" message, it is silently converted to `StopReason::Cancelled`. This pattern handles the common case where cancellation causes the agent to return an error instead of a clean stop reason — i.e., protocol-level error recovery.

**Context server (JSON-RPC) error handling**: Timeout, cancel, and error paths all use `select!` macro with `pin!()` for futures. Cancellation sends a `Cancelled` notification to the server before bailing with `RequestCanceled` error type. The error type implements `std::error::Error` + `Display` to integrate with `anyhow`.

---

## Decision Matrix

| Dimension | Zed ACP | Alder Grove ACP (current design) | Recommendation |
|-----------|---------|----------------------------------|----------------|
| Transport | stdio JSON-RPC (local process) | WebSocket (network, hub-and-spoke) | Keep WebSocket — Alder Grove targets cloud agents, not local processes |
| Session state | In-memory (`AcpThread` entity) | PostgreSQL rows + in-memory broker | Keep PostgreSQL for durability — adds audit trail Zed lacks |
| Gate mechanism | `oneshot::Sender` embedded in `ToolCallStatus` | Database row + polling | Adopt oneshot pattern for in-memory coordination; keep DB row for audit |
| Event stream | GPUI entity events (`EventEmitter`) | Append-only DB table + WS broadcast | Keep DB events for audit; add in-memory broadcast for low latency |
| Permission granularity | Per-tool-call (fine-grained) | Per-session (coarse) | Add tool-call-level granularity to gate_definitions |
| Permission options | `AllowOnce`/`AllowAlways`/`RejectOnce`/`RejectAlways` | `approved`/`denied` | Adopt the four-variant model for sticky permissions |
| Capability negotiation | At initialize + optional trait returns | Not yet specified | Adopt capability negotiation at session creation |
| Error recovery | `suppress_abort_err` flag pattern | Not yet specified | Adopt: flag + ErrorCode translation at WS boundary |
| Protocol messages | Tagged JSON-RPC methods | Tagged Serde enum (`#[serde(tag="type")]`) | Current design is correct |
| Multiplexing | N/A (single purpose) | Three channels (ACP, CRDT, awareness) | Current design is correct |

---

## Options

### Option A: Adopt Zed's Gate Pattern Directly (oneshot in state)

Embed `oneshot::Sender<GateDecision>` directly in the `SessionStatus::Gated` variant. Session execution `await`s the channel; the API endpoint resolves it.

**Pros:** Zero latency for gate decisions, no DB polling, idiomatic Rust async
**Cons:** Gate state is lost on server restart; requires careful state reconstruction on reconnect

### Option B: Keep DB-primary + Add In-Memory Coordination (hybrid)

Store gate in DB (durable), also hold a `oneshot::Sender` in the broker that resolves the DB row update. Both the DB write and channel send happen atomically in the approve/deny handler.

**Pros:** Durable audit trail, survives server restart, low latency decisions
**Cons:** More complex state synchronization

### Option C: Granular Tool-Call Gates (Zed-inspired permission model)

Add a `tool_call_gates` concept — lightweight gates that don't require a full `gate_definitions` template. Any tool action can request permission ad-hoc with a `reason` and `PermissionOptions`.

**Pros:** More observable, finer control
**Cons:** Schema complexity, more UI surface area

---

## Recommendation

### Primary: Adopt Option B (Hybrid DB + In-Memory Oneshot)

The current Alder Grove gate design (DB-primary) is sound. Augment it with:

1. **Embed a `oneshot::Sender<GateDecision>` in `AcpBroker`'s session state** alongside the DB gate row. When a gate is created: write DB row AND store channel sender in broker. When approved/denied: send on channel AND update DB row. The agent `await`s the channel, not the DB.

2. **Add `PermissionOptionKind` to the gate model**: Extend `gates.status` to support `approved_once`, `approved_always`, `denied_once`, `denied_always`. Map these to guardrail rule updates for `AllowAlways`/`DenyAlways` cases.

3. **Tool-call-level events**: The current event taxonomy includes `gate_triggered/approved/denied` but should also include the `tool_call_id` in gate context JSONB so the full drill-down from session → gate → specific tool action is available.

### Secondary: Adopt Zed's `ToolCallStatus` State Machine Pattern

For the in-memory session state in `grove-api/src/acp/session.rs`, model the session state after Zed's `ToolCallStatus`:

```rust
pub enum SessionExecutionState {
    Pending,
    Running { cancel_tx: oneshot::Sender<()> },
    WaitingForGate {
        gate_id: Uuid,
        decision_tx: oneshot::Sender<GateDecision>,
    },
    Completed,
    Failed,
    Cancelled,
}
```

This makes the state machine transitions explicit and prevents invalid transitions at the type level.

### Tertiary: Adopt `suppress_abort_err` Pattern

Add a `suppress_next_error: bool` flag to the in-memory session state that is set before sending `CancelNotification` to the agent. Map protocol-level "aborted" errors to `SessionStatus::Cancelled` rather than `SessionStatus::Failed`.

### Quaternary: Adopt Capability Negotiation

When a client connects to the ACP WebSocket, send an `InitCapabilities` message (analogous to Zed's `InitializeRequest`) that returns what gate/guardrail capabilities are enabled for this session. This lets future agent clients declare what kinds of permission requests they support.

---

## Differences from Current Alder Grove ACP Design That Could Improve It

1. **Gate resolution latency**: Current design requires DB polling or WS push delay. Oneshot channels eliminate this — gate decision is resolved in microseconds, not milliseconds.

2. **Permission granularity**: Zed supports `AllowAlways`/`DenyAlways` which creates persistent permission rules. Alder Grove could map these to automatically creating/updating guardrail entries.

3. **`suppress_abort_err` pattern is missing**: Without it, agent cancellations may be logged as errors, polluting the event stream and misleading operators.

4. **Optional capability traits**: Zed's `AgentConnection` trait has many `fn ..() -> Option<Rc<dyn Trait>>` methods. This is a cleaner capability pattern than the current Alder Grove design which has no formal capability negotiation at connection time.

5. **Session state machine should carry the cancel channel**: Current design has `AcpBroker` holding a `DashMap` of senders. Moving the sender into a `SessionExecutionState::Running { cancel_tx }` variant is more correct — you can't cancel a session that isn't running, and the type system enforces it.

6. **Tool-call-level observability**: Zed tracks each tool call as a discrete entity with its own status lifecycle. Alder Grove's event taxonomy covers this in `events.event_type` but the in-memory `AcpBroker` has no equivalent — adds richness to the live view of an active session.

---

## Anti-Patterns / Lessons from Zed

1. **`Meta` as escape hatch accumulates technical debt**: Zed uses `Meta: HashMap<String, serde_json::Value>` extensively as a workaround for protocol limitations (tool names, subagent IDs, auth info). Every `meta.get(SOME_KEY)` is an implicit protocol extension that bypasses the type system. Alder Grove should avoid this pattern — use proper typed fields in ACP message structs.

2. **`suppress_abort_err` is a vendor-specific workaround**: The flag exists to paper over Google Gemini CLI returning `InternalError` instead of a clean cancellation. Alder Grove's ACP layer should define strict error code semantics and require agents to conform, rather than adding vendor-specific workarounds in the broker.

3. **`Rc<dyn AgentConnection>` (not `Arc`)**: Zed uses `Rc` (not thread-safe) because GPUI is single-threaded. Alder Grove's API server is multi-threaded (tokio), so all equivalents should use `Arc` + `Send + Sync` bounds.

4. **Protocol version check with hard minimum**: Zed checks `response.protocol_version < MINIMUM_SUPPORTED_VERSION` and returns `UnsupportedVersion` error immediately. Alder Grove should do the same — define `ACP_MIN_PROTOCOL_VERSION` as a constant and reject connections below it during handshake.

5. **JSON-RPC `i32` request IDs overflow risk**: The `context_server` client uses `AtomicI32::fetch_add(1, SeqCst)` — this wraps around after ~2 billion requests. For a long-lived server, use `u64` or UUID request IDs.

---

## Sources

- `crates/acp_tools/src/acp_tools.rs` — ACP debug tooling, connection registry, stream message types
- `crates/acp_thread/src/connection.rs` — `AgentConnection` trait, session lifecycle ops, `ToolCallStatus`, `PermissionOptions`
- `crates/acp_thread/src/acp_thread.rs` — `AcpThread` entity, `AcpThreadEvent`, `handle_session_update`, `request_tool_call_authorization`, streaming patterns
- `crates/agent_servers/src/acp.rs` — `AcpConnection` impl, stdio transport, initialize handshake, session management, `map_acp_error`, `suppress_abort_err`
- `crates/agent/src/native_agent_server.rs` — Native agent server, `AgentServer` trait
- `crates/agent/src/thread.rs` — `Message`, `UserMessage`, retry strategies, `PromptId`
- `crates/action_log/src/action_log.rs` — File change tracking, undo/reject log
- `crates/context_server/src/client.rs` — JSON-RPC client, request/response correlation, timeout, cancellation
- `crates/context_server/src/protocol.rs` — MCP protocol, capability negotiation, `InitializedContextServerProtocol`
- `crates/rpc/src/peer.rs` — Multiplexed WebSocket peer, connection state, bounded/unbounded channels
- `crates/rpc/src/conn.rs` — WebSocket `Connection` abstraction, `in_memory` test helper
- `crates/session/src/session.rs` — App session persistence (window stack, session ID)
- `crates/rules_library/src/rules_library.rs` — Rules/guardrails UI, `PromptStore` integration
- [Zed Industries agent-client-protocol GitHub](https://github.com/zed-industries/agent-client-protocol)
- [Agent Client Protocol specification](https://agentclientprotocol.com)
- [Intro to ACP: The Standard for AI Agent-Editor Integration](https://block.github.io/goose/blog/2025/10/24/intro-to-agent-client-protocol-acp/)
- Existing Alder Grove ACP design spec: `.docs/superpowers/specs/2026-03-14-acp-rust-architecture-design.md`

---

## Next Steps

This research unblocks the following implementation work:

1. **`crates/grove-api/src/acp/session.rs`** — Adopt `SessionExecutionState` enum with embedded oneshot channels for gate resolution
2. **`crates/grove-api/src/acp/broker.rs`** — Move cancel/gate senders into session state rather than a separate DashMap
3. **`crates/grove-domain/src/gate.rs`** — Add `PermissionOptionKind` variants (`approved_once`, `approved_always`, `denied_once`, `denied_always`) to gate status
4. **`crates/grove-domain/src/acp.rs`** — Add `InitCapabilities` / `CapabilitiesResponse` message types to the `AcpMessage` enum for connection-time capability negotiation
5. **`crates/grove-api/src/acp/protocol.rs`** — Add `suppress_abort_err` flag to in-memory session state; map protocol cancellation errors at the boundary
6. **Gate context JSONB** — Ensure `tool_call_id` is included in gate context so tool-action-level drilldown is possible

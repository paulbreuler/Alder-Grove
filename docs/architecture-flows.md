# Alder Grove — Architecture Flows

## Request Flow (Desktop → API)

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐     ┌────────────┐
│ Tauri Shell  │────▶│ React 19.2   │────▶│ Axum 0.8 API │────▶│ PostgreSQL │
│ (Desktop)    │     │ (Webview)    │     │ (Cloud)      │     │ 18         │
└─────────────┘     └──────────────┘     └──────────────┘     └────────────┘
                          │                     │
                          │                     ├── Clerk JWT validation
                          │                     ├── org_id extraction
                          │                     └── workspace_id scoping
                          │
                          └── ClerkProvider (auth context)
```

## Hexagonal Dependency Flow

```
                    ┌─────────────────────────┐
                    │        Domain           │
                    │  Types, Entities, Rules  │
                    │  (zero dependencies)     │
                    └────────▲────────────────┘
                             │
              ┌──────────────┴──────────────┐
              │        Application          │
              │  Hooks, Stores, Use Cases   │
              │  (depends on Domain only)   │
              └──────▲───────────────▲──────┘
                     │               │
           ┌─────────┴──┐     ┌─────┴──────────┐
           │     UI     │     │    Adapters     │
           │ Components │     │ API, Tauri, WS  │
           └────────────┘     └────────────────┘
```

## ACP Session Lifecycle

```
Client                    API                     Agent
  │                        │                        │
  │── Create Session ─────▶│                        │
  │                        │── WebSocket ──────────▶│
  │                        │                        │
  │                        │◀── Events ────────────│
  │◀── Stream Events ─────│                        │
  │                        │                        │
  │                        │◀── Gate Request ──────│
  │◀── Gate Approval ─────│                        │
  │── Approve/Deny ───────▶│                        │
  │                        │── Gate Decision ──────▶│
  │                        │                        │
  │                        │◀── Session Complete ──│
  │◀── Final State ───────│                        │
  │                        │                        │
```

## Multi-Tenant Data Flow

```
Request
  │
  ▼
┌──────────────────────────────┐
│ Clerk JWT Middleware         │
│ Extract org_id from claims   │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Workspace Middleware         │
│ Verify membership            │
│ Extract workspace_id         │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Route Handler                │
│ /orgs/{org_id}/workspaces/   │
│   {ws_id}/personas/{id}     │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Database Query               │
│ WHERE workspace_id = $1      │
│ (always scoped)              │
└──────────────────────────────┘
```

## Shell Extension Registration

```
bootstrapShell()
  │
  ├── HomeExtension.register()
  ├── WorkspaceExtension.register()
  ├── PersonasExtension.register()
  ├── JourneysExtension.register()
  ├── SpecsExtension.register()
  ├── ACPExtension.register()
  ├── SettingsExtension.register()
  └── SnapshotsExtension.register()   (v2)
```

Each extension provides:
- Navigation entry
- Route definitions
- Feature-scoped Zustand store
- Self-contained UI components

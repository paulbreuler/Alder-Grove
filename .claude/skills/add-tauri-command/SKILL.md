---
name: add-tauri-command
description: Scaffold a Tauri IPC command with full round-trip (Rust handler, TS types, React hook, tests)
user_invocable: true
---

# /add-tauri-command

Scaffold a new Tauri v2 IPC command with the full round-trip: Rust command
handler, generated TypeScript types, React hook adapter, and tests at both
layers — all driven by TDD.

## Architecture

```
React Hook (adapter)
    |  invoke("command_name", { ... })
    v
Tauri IPC Bridge (@tauri-apps/api/core)
    |
    v
Rust Command (crates/grove-tauri/src/commands/<module>.rs)
    |  may call API client, local DB, or filesystem
    v
Side Effects (HTTP to grove-api, local storage, etc.)
```

The React hook lives in `src/features/<feature>/adapters/` — never in a
top-level `src/hooks/` directory. The Rust command lives in
`crates/grove-tauri/src/commands/`. TypeScript types are generated from
domain types via `ts-rs`, never hand-written.

## Reference Files

| Purpose                        | Path                                                 |
| ------------------------------ | ---------------------------------------------------- |
| Tauri main (command registry)  | `crates/grove-tauri/src/main.rs`                     |
| Domain types (Rust)            | `crates/grove-domain/src/*.rs`                       |
| TypeScript type generation     | `crates/grove-ts-gen/src/main.rs`                    |
| Generated TS types output      | `src/generated/types/`                               |
| Frontend feature adapters      | `src/features/<feature>/adapters/`                   |
| Tauri Cargo.toml               | `crates/grove-tauri/Cargo.toml`                      |

> **Note:** `crates/grove-tauri/` is currently a stub — `main.rs` only prints
> a placeholder message. The `commands/` directory should be created when
> adding the first command. This skill is forward-looking and establishes the
> patterns for when Tauri integration begins.

---

## Step 0: Gather Requirements

Before writing any code, clarify:

- **Command name** (snake_case): e.g. `get_personas`, `create_workspace`
- **Description**: one-line summary of what the command does
- **Input parameters**: names, Rust types, required vs optional
- **Return type**: Rust type (must be `Serialize`)
- **State dependencies**: what managed state does it need (API client, local DB,
  config, etc.)?
- **Side effects**: does it call the cloud API, write to filesystem, access
  system resources?
- **Feature scope**: which Shell extension / feature does this belong to?
  (determines the React hook location)

---

## Step 1: Write Rust Test FIRST (RED)

**File:** `crates/grove-tauri/src/commands/<module>.rs`

Write the command function and its test in the same file. The test runs
without the full Tauri runtime — it validates the command logic in isolation.

```rust
use serde::{Deserialize, Serialize};

/// Input parameters for the `my_command` IPC command.
///
/// Preconditions:
/// - `name` must be non-empty after trimming
#[derive(Debug, Deserialize)]
pub struct MyCommandInput {
    pub name: String,
}

/// Output returned to the frontend from `my_command`.
#[derive(Debug, Serialize)]
pub struct MyCommandOutput {
    pub id: String,
    pub name: String,
}

/// Tauri IPC command: creates a new entity.
///
/// # Errors
/// Returns a `String` error if input validation fails or the operation
/// cannot be completed.
#[tauri::command]
pub async fn my_command(input: MyCommandInput) -> Result<MyCommandOutput, String> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("name cannot be empty".into());
    }

    // TODO: call API client or local storage
    Ok(MyCommandOutput {
        id: uuid::Uuid::now_v7().to_string(),
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn my_command_rejects_empty_name() {
        let input = MyCommandInput {
            name: "   ".into(),
        };
        let result = my_command(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name cannot be empty"));
    }

    #[tokio::test]
    async fn my_command_trims_name() {
        let input = MyCommandInput {
            name: "  hello  ".into(),
        };
        let result = my_command(input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "hello");
    }
}
```

### Rust Command Rules

- **Error type at IPC boundary is `String`.** Tauri serializes errors across
  the IPC bridge; custom error types require manual `Serialize` + `Into<InvokeError>`.
  Use `String` for simplicity; map internal errors with `.map_err(|e| e.to_string())`.

- **Clone `Arc` before `.await`.** If the command takes managed state
  (`tauri::State<'_, Arc<T>>`), clone the `Arc` into a local variable before
  any `.await` point. Holding a `State` borrow across an await is unsound.

  ```rust
  #[tauri::command]
  pub async fn my_command(
      state: tauri::State<'_, Arc<ApiClient>>,
      input: MyCommandInput,
  ) -> Result<MyCommandOutput, String> {
      let client = state.inner().clone(); // Clone Arc before await
      let result = client.get_something().await.map_err(|e| e.to_string())?;
      // ...
  }
  ```

- **Input sanitization.** Validate and sanitize all inputs. For filesystem
  paths, reject path traversal (`..`). For strings, trim whitespace and
  validate length. For IDs, parse as `Uuid` early.

- **No panics.** Commands must never panic — always return `Result`.

Create the commands module if it does not exist:

**File:** `crates/grove-tauri/src/commands/mod.rs`

```rust
pub mod my_module;
```

---

## Step 2: Write Vitest Test FIRST (RED)

**File:** `src/features/<feature>/adapters/use<CommandName>.test.ts`

Mock the Tauri `invoke` function to test the React hook in isolation.

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useMyCommand } from './useMyCommand';

// Mock Tauri's invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('useMyCommand', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('calls invoke with correct command name and args', async () => {
    const mockResult = { id: '123', name: 'test' };
    vi.mocked(invoke).mockResolvedValue(mockResult);

    const { result } = renderHook(() => useMyCommand());

    await act(async () => {
      const output = await result.current.execute({ name: 'test' });
      expect(output).toEqual(mockResult);
    });

    expect(invoke).toHaveBeenCalledWith('my_command', { input: { name: 'test' } });
  });

  it('surfaces errors from invoke', async () => {
    vi.mocked(invoke).mockRejectedValue('name cannot be empty');

    const { result } = renderHook(() => useMyCommand());

    await act(async () => {
      await expect(result.current.execute({ name: '' })).rejects.toBe(
        'name cannot be empty'
      );
    });
  });
});
```

---

## Step 3: Run Tests to Verify RED

```bash
cargo test -p grove-tauri 2>&1 | head -30
pnpm test -- --run src/features/<feature>/adapters/use<CommandName>.test.ts 2>&1 | head -30
```

Both must fail (compile error or assertion failure). This confirms the tests
are meaningful.

---

## Step 4: Implement Rust Command

If the test stub from Step 1 was minimal, fill in the real implementation now.

For commands that call the cloud API:

```rust
use std::sync::Arc;

pub struct ApiClient {
    base_url: String,
    http: reqwest::Client,
}

#[tauri::command]
pub async fn my_command(
    api: tauri::State<'_, Arc<ApiClient>>,
    input: MyCommandInput,
) -> Result<MyCommandOutput, String> {
    let client = api.inner().clone();
    let name = input.name.trim().to_string();

    if name.is_empty() {
        return Err("name cannot be empty".into());
    }

    let response = client
        .http
        .post(format!("{}/orgs/{}/workspaces/{}/my-entities",
            client.base_url, input.org_id, input.workspace_id))
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let entity: MyCommandOutput = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(entity)
}
```

---

## Step 5: Generate TypeScript Types

Types are generated from Rust domain structs annotated with `ts-rs`:

```bash
pnpm generate:types
```

This runs `grove-ts-gen` which exports all types marked with
`#[cfg_attr(feature = "ts", derive(ts_rs::TS))]` and `#[cfg_attr(feature = "ts", ts(export))]`
to `src/generated/types/`.

**Never hand-write TypeScript type definitions for domain types.** If a type
needs to be available in TypeScript, add the `ts-rs` derive to the Rust struct
in `crates/grove-domain/` and regenerate.

For command-specific input/output types that are NOT domain types (they live
in `grove-tauri`, not `grove-domain`), you may define TypeScript interfaces
directly in the hook file, mirroring the Rust structs.

---

## Step 6: Create React Hook

**File:** `src/features/<feature>/adapters/use<CommandName>.ts`

The hook wraps Tauri's `invoke` with typed parameters and return values.

```typescript
import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Import generated types if using domain types:
// import type { MyEntity } from '@/generated/types/MyEntity';

// Or define IPC-specific types inline:
interface MyCommandInput {
  name: string;
}

interface MyCommandOutput {
  id: string;
  name: string;
}

export function useMyCommand() {
  const execute = useCallback(async (input: MyCommandInput): Promise<MyCommandOutput> => {
    return invoke<MyCommandOutput>('my_command', { input });
  }, []);

  return { execute };
}
```

### Hook Conventions

- Hook name: `use<CommandName>` in PascalCase (e.g. `useGetPersonas`,
  `useCreateWorkspace`)
- File name: `use<CommandName>.ts` (e.g. `useGetPersonas.ts`)
- Location: `src/features/<feature>/adapters/`
- Use `useCallback` to stabilize the function reference
- Type the `invoke` call with the generic parameter: `invoke<ReturnType>(...)`
- For commands with loading/error state, consider returning
  `{ execute, isLoading, error }` using `useState`

---

## Step 7: Register Command

**File:** `crates/grove-tauri/src/main.rs`

Register the command in the Tauri builder. The current `main.rs` is a stub
and will need to be replaced with the Tauri app builder when desktop
integration begins.

The target structure for `main.rs`:

```rust
mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::my_module::my_command,
            // ... other commands ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Until the full Tauri setup is in place, ensure the command module compiles
and tests pass independently:

```bash
cargo test -p grove-tauri
```

---

## Step 8: Verify GREEN

```bash
# Rust tests
cargo test -p grove-tauri

# Frontend tests
pnpm test -- --run src/features/<feature>/adapters/use<CommandName>.test.ts

# Full workspace verification
cargo clippy --workspace -- -D warnings
pnpm check
pnpm test
```

All must pass.

---

## Patterns

### Command with Managed State

When a command needs access to shared state (API client, config, database):

```rust
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn my_command(
    client: State<'_, Arc<ApiClient>>,
    config: State<'_, Arc<AppConfig>>,
    input: MyCommandInput,
) -> Result<MyCommandOutput, String> {
    let client = client.inner().clone();
    let config = config.inner().clone();
    // ... use client and config after cloning ...
}
```

Register managed state in `main.rs`:

```rust
tauri::Builder::default()
    .manage(Arc::new(api_client))
    .manage(Arc::new(app_config))
    .invoke_handler(tauri::generate_handler![...])
```

### Command with Event Emission

When a command needs to notify the frontend of async updates:

```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn my_command(
    app: AppHandle,
    input: MyCommandInput,
) -> Result<MyCommandOutput, String> {
    // ... do work ...

    app.emit("my-event", &payload).map_err(|e| e.to_string())?;

    Ok(output)
}
```

### File System Commands

For commands that access the filesystem, always validate paths:

```rust
use std::path::PathBuf;

fn validate_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);

    // Reject path traversal
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("path traversal not allowed".into());
    }

    // Resolve to absolute and verify it exists
    let canonical = path.canonicalize().map_err(|e| e.to_string())?;

    Ok(canonical)
}
```

---

## Checklist

Before marking this command as done, verify:

- [ ] Command name follows `snake_case` convention
- [ ] Rust command in `crates/grove-tauri/src/commands/<module>.rs`
- [ ] Commands module registered in `crates/grove-tauri/src/commands/mod.rs`
- [ ] `#[tauri::command]` annotation on the function
- [ ] Error type is `String` at the IPC boundary
- [ ] `Arc` cloned before any `.await` (if using managed state)
- [ ] Input validated and sanitized (trim, empty check, path traversal)
- [ ] No panics — all paths return `Result`
- [ ] Rust unit tests in `#[cfg(test)] mod tests`
- [ ] Rust tests written BEFORE implementation (RED phase confirmed)
- [ ] TypeScript types generated via `pnpm generate:types` (domain types)
  or defined inline (IPC-specific types)
- [ ] React hook in `src/features/<feature>/adapters/use<CommandName>.ts`
- [ ] Vitest test in `src/features/<feature>/adapters/use<CommandName>.test.ts`
- [ ] Vitest test written BEFORE hook implementation (RED phase confirmed)
- [ ] `@tauri-apps/api/core` mocked in Vitest test
- [ ] Command registered in `generate_handler!` (when Tauri builder is set up)
- [ ] `cargo test -p grove-tauri` passes
- [ ] `pnpm test` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes

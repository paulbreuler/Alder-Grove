---
paths:
  - "src-tauri/**/*.rs"
  - "src-tauri/**/*.json"
---

# Desktop Rules (Tauri v2)

## Native-First Verification

- Always verify features work in the Tauri desktop context, not just the browser
- `cargo tauri dev` is the primary development command
- File system access, window management, and IPC go through Tauri APIs

## Tauri Commands

- Commands are the bridge between frontend and native Rust
- Every command must validate its inputs
- Commands that access the filesystem must validate paths (no traversal)
- Commands must not block the main thread — use async where needed

## Configuration

- `tauri.conf.json` defines window properties, permissions, and plugins
- Security: minimize allowed Tauri APIs to only what's needed
- CSP (Content Security Policy) must be configured for the webview

## IPC Security

- Tauri commands are callable from the webview — treat as an API boundary
- Validate all arguments as if they come from untrusted input
- Do not expose raw filesystem operations to the webview

## Build & Distribution

- Desktop app builds independently from the cloud API
- Auto-update configuration via Tauri updater plugin
- Code signing required for distribution
- Icons in `src-tauri/icons/`

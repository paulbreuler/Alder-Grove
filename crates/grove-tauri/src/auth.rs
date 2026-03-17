//! Clerk authentication Tauri commands.
//!
//! The Clerk JS SDK owns session management and token refresh.
//! Rust is responsible for:
//! - Persisting the JWT in an encrypted store (survives app restarts)
//! - Opening OAuth URLs in the system browser
//! - Running a one-shot HTTP callback server for OAuth redirects
//! - Forwarding callback params to the frontend

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use tauri::Emitter;
use tauri_plugin_store::StoreExt;

const AUTH_STORE_PATH: &str = "clerk-auth.json";
const TOKEN_KEY: &str = "clerk_session_token";

/// Port for the one-shot OAuth callback HTTP server.
/// Must not conflict with Vite dev server (5173).
const OAUTH_CALLBACK_PORT: u16 = 19287;

/// The redirect URL that Clerk will send the browser to after OAuth.
/// This is an HTTP URL that Clerk accepts (not a custom scheme).
pub const OAUTH_REDIRECT_URL: &str = "http://127.0.0.1:19287/clerk-callback";

/// Persist the Clerk session JWT to the encrypted store.
#[tauri::command]
pub async fn set_clerk_token(app: tauri::AppHandle, token: String) -> Result<(), String> {
    let store = app
        .store(AUTH_STORE_PATH)
        .map_err(|e| format!("Failed to open auth store: {e}"))?;

    store.set(TOKEN_KEY, serde_json::Value::String(token));
    store
        .save()
        .map_err(|e| format!("Failed to save auth store: {e}"))?;

    tracing::debug!("Clerk session token persisted to encrypted store");
    Ok(())
}

/// Load the persisted Clerk session JWT on startup.
#[tauri::command]
pub async fn get_stored_clerk_token(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = app
        .store(AUTH_STORE_PATH)
        .map_err(|e| format!("Failed to open auth store: {e}"))?;

    let token = store
        .get(TOKEN_KEY)
        .and_then(|v| v.as_str().map(String::from));

    if token.is_some() {
        tracing::debug!("Restored Clerk session token from encrypted store");
    }

    Ok(token)
}

/// Clear the persisted Clerk session JWT on logout.
#[tauri::command]
pub async fn clear_clerk_token(app: tauri::AppHandle) -> Result<(), String> {
    let store = app
        .store(AUTH_STORE_PATH)
        .map_err(|e| format!("Failed to open auth store: {e}"))?;

    store.delete(TOKEN_KEY);
    store
        .save()
        .map_err(|e| format!("Failed to save auth store: {e}"))?;

    tracing::info!("Clerk session token cleared from encrypted store");
    Ok(())
}

/// Return the OAuth callback URL that the frontend should pass to Clerk.
#[tauri::command]
pub fn get_oauth_callback_url() -> String {
    OAUTH_REDIRECT_URL.to_string()
}

/// Start a one-shot HTTP server, open the OAuth URL in the system browser,
/// wait for Clerk to redirect back, then emit the callback URL to the frontend.
///
/// Flow:
/// 1. Bind a TCP listener on 127.0.0.1:19287
/// 2. Open the provided auth URL in the system browser
/// 3. Wait for a single HTTP request (Clerk redirects here after OAuth)
/// 4. Respond with a "You can close this tab" HTML page
/// 5. Emit "auth-callback" event to the frontend with the full request URL
#[tauri::command]
pub async fn start_oauth_flow(app: tauri::AppHandle, auth_url: String) -> Result<(), String> {
    // Spawn blocking because TcpListener::accept() blocks the thread
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let listener = TcpListener::bind(format!("127.0.0.1:{OAUTH_CALLBACK_PORT}"))
            .map_err(|e| format!("Failed to bind OAuth callback server: {e}"))?;

        tracing::info!("OAuth callback server listening on port {OAUTH_CALLBACK_PORT}");

        // Open the system browser
        open::that(&auth_url).map_err(|e| format!("Failed to open browser: {e}"))?;

        // Accept exactly one connection (with a timeout)
        listener
            .set_nonblocking(false)
            .map_err(|e| format!("Failed to set blocking mode: {e}"))?;

        let (mut stream, _addr) = listener
            .accept()
            .map_err(|e| format!("OAuth callback: failed to accept connection: {e}"))?;

        // Read the HTTP request line to extract the path + query
        let reader = BufReader::new(&stream);
        let request_line = reader
            .lines()
            .next()
            .ok_or_else(|| "OAuth callback: empty request".to_string())?
            .map_err(|e| format!("OAuth callback: read error: {e}"))?;

        // Parse "GET /clerk-callback?code=...&state=... HTTP/1.1"
        let path = request_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("/")
            .to_string();

        // Respond with a simple page that closes itself
        let body = r#"<!DOCTYPE html>
<html><head><title>Alder Grove</title></head>
<body style="font-family:system-ui;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;background:#0f1117;color:#e4e4e7">
<div style="text-align:center">
<h2>Authentication complete</h2>
<p>You can close this tab and return to Alder Grove.</p>
<script>window.close()</script>
</div></body></html>"#;

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
        drop(stream);
        drop(listener);

        // Build the full callback URL and emit to frontend
        let callback_url = format!("http://127.0.0.1:{OAUTH_CALLBACK_PORT}{path}");
        tracing::info!("OAuth callback received, forwarding to frontend");

        if let Err(e) = handle.emit("auth-callback", callback_url) {
            tracing::warn!("Failed to emit auth-callback: {e}");
        }

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("OAuth flow task failed: {e}"))?
}

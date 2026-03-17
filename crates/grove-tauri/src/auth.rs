//! Clerk authentication Tauri commands.
//!
//! The Clerk JS SDK owns session management and token refresh.
//! Rust is responsible for:
//! - Persisting the JWT in an encrypted store (survives app restarts)
//! - Opening OAuth URLs in the system browser
//! - Forwarding deep-link callbacks to the frontend

use tauri_plugin_store::StoreExt;

const AUTH_STORE_PATH: &str = "clerk-auth.json";
const TOKEN_KEY: &str = "clerk_session_token";

/// Persist the Clerk session JWT to the encrypted store.
///
/// Called by the frontend's `useClerkSync` hook whenever the Clerk session
/// issues a new token (initial auth + periodic refresh).
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
///
/// Returns `None` if no token is stored. The token may be expired —
/// the Clerk JS SDK will refresh it automatically once loaded.
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

/// Open a URL in the system browser.
///
/// Used to launch Clerk's OAuth authorization flow in a real browser
/// (where cookies and OAuth redirects work correctly), rather than
/// the Tauri webview (where they don't).
#[tauri::command]
pub async fn open_auth_url(url: String) -> Result<(), String> {
    tracing::info!("Opening system browser for Clerk OAuth");
    open::that(&url).map_err(|e| format!("Failed to open browser: {e}"))
}

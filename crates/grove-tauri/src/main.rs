#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;

use tauri::{Emitter, Listener};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Forward deep-link callbacks to the frontend.
            // When the system browser redirects to `grove://callback?...`,
            // the deep-link plugin fires this handler.
            let handle = app.handle().clone();
            app.listen("deep-link://new-url", move |event| {
                tracing::info!("Deep link received");
                if let Err(e) = handle.emit("auth-callback", event.payload().to_string()) {
                    tracing::warn!("Failed to emit auth-callback event: {e}");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::set_clerk_token,
            auth::get_stored_clerk_token,
            auth::clear_clerk_token,
            auth::get_oauth_callback_url,
            auth::start_oauth_flow,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Alder Grove");
}

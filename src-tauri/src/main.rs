// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod p2p;
mod state;

use tokio::sync::mpsc;
use tauri::Manager;
use state::AppState;

#[tauri::command]
async fn send_message(
    msg: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.p2p_sender
        .send(msg)
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let (tx, rx) = mpsc::channel(32);

            // ✅ CLONAR el handle para mover al async task
            let app_handle = app.handle().clone();

            // Spawn P2P worker en background
            tauri::async_runtime::spawn(async move {
                if let Err(e) = p2p::start_p2p(rx, app_handle).await {
                    eprintln!("❌ P2P error: {}", e);
                }
            });

            // Manage state
            app.manage(AppState { p2p_sender: tx });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_message])
        .run(tauri::generate_context!())
        .expect("error running tauri");
}
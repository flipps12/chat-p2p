// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod p2p;
mod state;

use tokio::sync::mpsc;
use tauri::Manager;
use state::AppState;

/// Comando para enviar mensajes de chat
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

/// Comando para conectar manualmente a un peer
#[tauri::command]
async fn connect_to_peer(
    address: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Enviar comando especial CMD:CONNECT:
    state.p2p_sender
        .send(format!("CMD:CONNECT:{}", address))
        .await
        .map_err(|e| e.to_string())
}

/// Comando para obtener lista de peers conectados
#[tauri::command]
async fn get_connected_peers(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.p2p_sender
        .send("CMD:GET_PEERS".to_string())
        .await
        .map_err(|e| e.to_string())
}

/// Comando para obtener información del nodo local (peer_id y direcciones)
#[tauri::command]
async fn get_my_info(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.p2p_sender
        .send("CMD:GET_INFO".to_string())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_topic(
    topic: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.p2p_sender
        .send(format!("CMD:ADD_TOPIC:{}", topic))
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let (tx, rx) = mpsc::channel(32);

            // Clonar el handle para mover al async task
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
        .invoke_handler(tauri::generate_handler![
            send_message,
            connect_to_peer,
            get_connected_peers,
            get_my_info,
            add_topic,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri");
}
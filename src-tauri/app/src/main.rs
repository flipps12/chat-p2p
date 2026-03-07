// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use tauri::{Manager, State};

mod state;

use crate::state::AppState;

/// Comando para enviar mensajes de chat
// #[tauri::command]
// async fn send_message(
//     msg: Message,
//     state: tauri::State<'_, AppState>,
// ) -> Result<(), String> {
//     state.p2p_sender
//         .send(format!("CMD:SEND_MESSAGE:{}", serde_json::to_string(&msg).unwrap()))
//         .await
//         .map_err(|e| e.to_string())
// }

// /// Comando para conectar manualmente a un peer
// #[tauri::command]
// async fn connect_to_peer(
//     address: String,
//     state: tauri::State<'_, AppState>,
// ) -> Result<(), String> {
//     // Enviar comando especial CMD:CONNECT:
//     state.p2p_sender
//         .send(format!("CMD:CONNECT:{}", address))
//         .await
//         .map_err(|e| e.to_string())
// }

// /// Comando para obtener lista de peers conectados
// #[tauri::command]
// async fn get_connected_peers(
//     state: tauri::State<'_, AppState>,
// ) -> Result<(), String> {
//     state.p2p_sender
//         .send("CMD:GET_PEERS".to_string())
//         .await
//         .map_err(|e| e.to_string())
// }

// /// Comando para obtener información del nodo local (peer_id y direcciones)
// #[tauri::command]
// async fn get_my_info(
//     state: tauri::State<'_, AppState>,
// ) -> Result<(), String> {
//     state.p2p_sender
//         .send("CMD:GET_INFO".to_string())
//         .await
//         .map_err(|e| e.to_string())
// }

// #[tauri::command]
// async fn add_topic(
//     topic: String,
//     state: tauri::State<'_, AppState>,
// ) -> Result<(), String> {
//     state.p2p_sender
//         .send(format!("CMD:ADD_TOPIC:{}", topic))
//         .await
//         .map_err(|e| e.to_string())
// }

// // save and load peers and channels
// #[tauri::command]
// async fn get_peers(state: State<'_, AppState>) -> Result<Vec<PeerInfoToSave>, String> {
//     let fm = state.file_manager.lock().await;
//     fm.load_peers().map_err(|e| e.to_string())
// }

// #[tauri::command]
// async fn get_channels(state: State<'_, AppState>) -> Result<Vec<ChannelInfoToSave>, String> {
//     let fm = state.file_manager.lock().await;
//     fm.load_channels().map_err(|e| e.to_string())
// }

// #[tauri::command]
// async fn add_channel(
//     topic: String,
//     uuid: String,
//     state: State<'_, AppState>,
// ) -> Result<(), String> {
//     let channel = ChannelInfoToSave {
//         topic,
//         uuid,
//         last_message_uuid: None,
//     };
    
//     let fm = state.file_manager.lock().await;
//     fm.add_or_update_channel(channel).map_err(|e| e.to_string())
// }

// #[tauri::command]
// async fn remove_channel(uuid: String, state: State<'_, AppState>) -> Result<(), String> {
//     let fm = state.file_manager.lock().await;
//     fm.remove_channel(&uuid).map_err(|e| e.to_string())
// }

// #[tauri::command]
// async fn export_data(path: String, state: State<'_, AppState>) -> Result<(), String> {
//     let export_dir = std::path::PathBuf::from(path);
//     let fm = state.file_manager.lock().await;
//     fm.export_data(&export_dir).map_err(|e| e.to_string())
// }

// #[tauri::command]
// async fn import_data(path: String, state: State<'_, AppState>) -> Result<(), String> {
//     let import_dir = std::path::PathBuf::from(path);
//     let fm = state.file_manager.lock().await;
//     fm.import_data(&import_dir).map_err(|e| e.to_string())
// }

// #[tauri::command]
// async fn clear_all_data(state: State<'_, AppState>) -> Result<(), String> {
//     let fm = state.file_manager.lock().await;
//     fm.clear_all_data().map_err(|e| e.to_string())
// }

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let (tx, rx) = mpsc::channel(32);

            // Clonar el handle para mover al async task
            let app_handle = app.handle().clone();

            

            // Manage state
            app.manage(AppState { 
                p2p_sender: tx,
                // file_manager,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error running tauri");
}
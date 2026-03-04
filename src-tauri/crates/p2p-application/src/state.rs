use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use crate::fs::FileManager;

#[derive(Clone)]
pub struct AppState {
    pub p2p_sender: mpsc::Sender<String>,
    pub file_manager: Arc<Mutex<FileManager>>,
}